//! Receipt-first composition root for one owned runtime execution.

use crate::cleanup::{cancel_after_host_failure, CleanupAttempt};
use crate::config::ExecutionPolicy;
use crate::envelope::{EventBuilder, ExecutionContext};
use crate::host_api::{ExecutionRequest, HostServices, Presentation, RunningResources};
use crate::host_error::HostError;
use crate::process::{ProcessLaunch, ProcessTreeCapability};
use crate::report::{CleanupStatus, ExecutionFailure, ExecutionReport, FailureDetail};
use crate::safety::PermissionResolution;
use crate::session::RunningExecution;
use tsukumo_kernel::{
    ExecutionId, KernelEvent, KernelEventPayload, OutcomeStatus, PersistedText, RuntimeBinding,
    RuntimePhase,
};
use tsukumo_soul::{AppendOutcome, PreparedProjection, ProjectionReceipt, SoulError};
use tsukumo_theater::drive_kernel_event;

/// Host composition root for one ledger and one Theater world.
pub struct RuntimeOrchestrator<'a> {
    pub(crate) services: HostServices<'a>,
    presentation: Presentation<'a>,
    pub(crate) policy: ExecutionPolicy,
}

impl<'a> RuntimeOrchestrator<'a> {
    /// Creates a composition root whose mutable sinks cannot be bypassed.
    pub const fn new(
        services: HostServices<'a>,
        presentation: Presentation<'a>,
        policy: ExecutionPolicy,
    ) -> Self {
        Self {
            services,
            presentation,
            policy,
        }
    }

    /// Runs one prepared projection through receipt check, process, Chronicle, and Theater.
    pub fn execute(&mut self, request: ExecutionRequest<'_>) -> Result<ExecutionReport, HostError> {
        self.verify_receipt(request.prepared, request.runtime.profile.binding())?;
        let command = request.runtime.profile.command(request.runtime.launch)?;
        let execution_id = request.prepared.receipt.execution_id.clone();
        let resources = RunningResources {
            decoder: request.runtime.profile.decoder(),
            cancellation: request.cancellation,
            safety_capability: request.runtime.profile.safety_capability(),
            process_tree: self.services.runner.process_tree_capability(),
        };
        let mut builder = EventBuilder::new(request.context, &request.prepared.receipt);
        self.commit_starting(&mut builder, &execution_id)?;

        let launch = ProcessLaunch::new(
            command,
            Some(request.prepared.rendered_prompt().clone()),
            self.policy.process_limits(),
        );
        let mut handle = match self.services.runner.spawn(launch) {
            Ok(handle) => handle,
            Err(error) => {
                return self.report_launch_failure(&mut builder, error, resources.process_tree)
            }
        };
        if let Err(error) = self.commit_started(&mut builder) {
            let cleanup = cancel_after_host_failure(handle.as_mut());
            return Err(error.with_cleanup(cleanup));
        }

        RunningExecution::new(self, builder, handle, resources).run()
    }
    /// Persists one controller-owned human decision after its request evidence exists.
    pub fn record_permission_resolution(
        &mut self,
        receipt: &ProjectionReceipt,
        context: ExecutionContext,
        resolution: PermissionResolution,
    ) -> Result<AppendOutcome, HostError> {
        let scope = &resolution.request.scope;
        if scope.execution_id != receipt.execution_id
            || scope.runtime != receipt.runtime
            || scope.session_id != context.session_id
        {
            return Err(HostError::PermissionScopeMismatch);
        }
        self.verify_receipt_value(receipt)?;
        let vendor_request = resolution.request.vendor_request.clone();
        let exists = self
            .services
            .ledger
            .permission_request_exists(&receipt.execution_id, &vendor_request)
            .map_err(HostError::ChronicleBeforeSpawn)?;
        if !exists {
            return Err(HostError::MissingPermissionRequest);
        }
        let timestamp = self.services.clock.now()?;
        let builder = EventBuilder::new(context, receipt);
        let event = builder.permission_decision(timestamp, resolution.into_payload());
        self.commit_event(&event)
            .map_err(HostError::ChronicleBeforeSpawn)
    }

    fn commit_starting(
        &mut self,
        builder: &mut EventBuilder,
        execution_id: &ExecutionId,
    ) -> Result<(), HostError> {
        let event = self.build_event(
            builder,
            KernelEventPayload::RuntimeLifecycle {
                phase: RuntimePhase::Starting,
            },
        )?;
        match self
            .commit_event(&event)
            .map_err(HostError::ChronicleBeforeSpawn)?
        {
            AppendOutcome::Inserted { .. } => Ok(()),
            AppendOutcome::Duplicate { .. } => Err(HostError::AlreadyExecuted {
                execution_id: execution_id.clone(),
            }),
        }
    }

    pub(crate) fn build_event(
        &self,
        builder: &mut EventBuilder,
        payload: KernelEventPayload,
    ) -> Result<KernelEvent, HostError> {
        let timestamp = self.services.clock.now()?;
        Ok(builder.next(timestamp, payload))
    }

    pub(crate) fn commit_event(&mut self, event: &KernelEvent) -> Result<AppendOutcome, SoulError> {
        let outcome = self.services.ledger.append_event(event)?;
        if matches!(outcome, AppendOutcome::Inserted { .. }) {
            drive_kernel_event(self.presentation.world, event, self.presentation.director);
        }
        Ok(outcome)
    }

    fn verify_receipt(
        &self,
        prepared: &PreparedProjection,
        selected_runtime: RuntimeBinding,
    ) -> Result<(), HostError> {
        self.verify_receipt_value(&prepared.receipt)?;
        if selected_runtime != prepared.receipt.runtime {
            return Err(HostError::RuntimeMismatch {
                receipt: prepared.receipt.runtime.clone(),
                selected: selected_runtime,
            });
        }
        Ok(())
    }

    fn verify_receipt_value(&self, receipt: &ProjectionReceipt) -> Result<(), HostError> {
        let Some(persisted) = self
            .services
            .ledger
            .projection_receipt(&receipt.id)
            .map_err(HostError::ChronicleBeforeSpawn)?
        else {
            return Err(HostError::MissingReceipt {
                projection_id: receipt.id.clone(),
            });
        };
        if persisted != *receipt {
            return Err(HostError::ReceiptMismatch {
                projection_id: receipt.id.clone(),
            });
        }
        Ok(())
    }

    fn commit_started(&mut self, builder: &mut EventBuilder) -> Result<(), PendingCommitError> {
        let event = self
            .build_event(
                builder,
                KernelEventPayload::RuntimeLifecycle {
                    phase: RuntimePhase::Started,
                },
            )
            .map_err(PendingCommitError::Host)?;
        self.commit_event(&event)
            .map_err(PendingCommitError::Chronicle)?;
        Ok(())
    }

    fn report_launch_failure(
        &mut self,
        builder: &mut EventBuilder,
        error: crate::process::ProcessError,
        process_tree: ProcessTreeCapability,
    ) -> Result<ExecutionReport, HostError> {
        for payload in [
            KernelEventPayload::RuntimeLifecycle {
                phase: RuntimePhase::Failed,
            },
            KernelEventPayload::Outcome {
                status: OutcomeStatus::LaunchFailed,
                summary: Some(PersistedText::from_reviewed("runtime launch failed")),
                projection_id: None,
            },
        ] {
            let event = self.build_event(builder, payload)?;
            self.commit_event(&event)
                .map_err(HostError::ChronicleBeforeSpawn)?;
        }
        Ok(ExecutionReport {
            status: OutcomeStatus::LaunchFailed,
            process_tree,
            failure: Some(ExecutionFailure::LaunchFailed),
            detail: Some(FailureDetail::Process(error)),
            cleanup: CleanupStatus::NotStarted,
            cleanup_error: None,
            exit: None,
            committed_events: 3,
            stderr_lines: 0,
            known_ignored_lines: 0,
            unknown_skipped_lines: 0,
        })
    }
}

enum PendingCommitError {
    Host(HostError),
    Chronicle(SoulError),
}

impl PendingCommitError {
    fn with_cleanup(self, cleanup: CleanupAttempt) -> HostError {
        match self {
            Self::Host(HostError::Clock(source)) => HostError::ClockDuringExecution {
                source,
                cleanup: cleanup.status,
                cleanup_error: cleanup.error,
            },
            Self::Host(other) => other,
            Self::Chronicle(source) => HostError::ChronicleDuringExecution {
                source,
                cleanup: cleanup.status,
                cleanup_error: cleanup.error,
            },
        }
    }
}
