//! Running-process loop and terminal reconciliation.

use crate::cleanup::cancel_after_host_failure;
use crate::envelope::EventBuilder;
use crate::host_api::{CancellationToken, RunningResources};
use crate::host_error::HostError;
use crate::orchestrator::RuntimeOrchestrator;
use crate::process::{ProcessTreeCapability, RuntimeHandle, RuntimeOutput};
use crate::report::{CleanupStatus, ExecutionFailure, ExecutionReport, FailureDetail};
use crate::terminal::{reconcile_exit, status_summary, Termination, VendorOutcome};
use std::time::Instant;
use tsukumo_adapters::{DecodeDisposition, RuntimeEventDecoder, RuntimeSafetyCapability};
use tsukumo_kernel::{KernelEventPayload, OutcomeStatus, PersistedText, RuntimePhase, Timestamp};
use tsukumo_soul::AppendOutcome;

pub(crate) struct RunningExecution<'run, 'host> {
    host: &'run mut RuntimeOrchestrator<'host>,
    builder: EventBuilder,
    handle: Box<dyn RuntimeHandle>,
    decoder: Box<dyn RuntimeEventDecoder>,
    cancellation: CancellationToken,
    safety_capability: RuntimeSafetyCapability,
    process_tree: ProcessTreeCapability,
    vendor_outcome: Option<VendorOutcome>,
    committed_events: usize,
    stderr_lines: usize,
    known_ignored_lines: usize,
    unknown_skipped_lines: usize,
    started_at: Timestamp,
    started: Instant,
}

impl<'run, 'host> RunningExecution<'run, 'host> {
    pub(crate) fn new(
        host: &'run mut RuntimeOrchestrator<'host>,
        builder: EventBuilder,
        handle: Box<dyn RuntimeHandle>,
        resources: RunningResources,
        started_at: Timestamp,
    ) -> Self {
        Self {
            host,
            builder,
            handle,
            decoder: resources.decoder,
            cancellation: resources.cancellation,
            safety_capability: resources.safety_capability,
            process_tree: resources.process_tree,
            vendor_outcome: None,
            committed_events: 2,
            stderr_lines: 0,
            known_ignored_lines: 0,
            unknown_skipped_lines: 0,
            started_at,
            started: Instant::now(),
        }
    }

    pub(crate) fn run(mut self) -> Result<ExecutionReport, HostError> {
        loop {
            if self.cancellation.is_cancelled() {
                return self.finish(Termination {
                    status: OutcomeStatus::Cancelled,
                    failure: Some(ExecutionFailure::Cancelled),
                    detail: None,
                    known_exit: None,
                    summary: None,
                });
            }
            let elapsed = self.started.elapsed();
            if elapsed >= self.host.policy.timeout() {
                return self.finish(Termination {
                    status: OutcomeStatus::TimedOut,
                    failure: Some(ExecutionFailure::TimedOut),
                    detail: None,
                    known_exit: None,
                    summary: None,
                });
            }
            let remaining = self.host.policy.timeout().saturating_sub(elapsed);
            let wait = remaining.min(self.host.policy.poll_interval());
            match self.handle.next(wait) {
                Ok(RuntimeOutput::StdoutLine(line)) => {
                    if let Some(termination) = self.decode_line(&line)? {
                        return self.finish(termination);
                    }
                }
                Ok(RuntimeOutput::StderrLine(_)) => {
                    self.stderr_lines = self.stderr_lines.saturating_add(1);
                }
                Ok(RuntimeOutput::Idle) => {}
                Ok(RuntimeOutput::Exited(exit)) => {
                    let termination =
                        reconcile_exit(self.decoder.as_ref(), &mut self.vendor_outcome, exit);
                    return self.finish(termination);
                }
                Err(error) => {
                    return self.finish(Termination {
                        status: OutcomeStatus::Degraded,
                        failure: Some(ExecutionFailure::ProcessFailure),
                        detail: Some(FailureDetail::Process(error)),
                        known_exit: None,
                        summary: None,
                    });
                }
            }
        }
    }

    fn decode_line(&mut self, line: &str) -> Result<Option<Termination>, HostError> {
        let decoded = match self.decoder.decode_line(line) {
            Ok(decoded) => decoded,
            Err(error) => {
                return Ok(Some(Termination {
                    status: OutcomeStatus::MalformedOutput,
                    failure: Some(ExecutionFailure::MalformedOutput),
                    detail: Some(FailureDetail::Adapter(error)),
                    known_exit: None,
                    summary: None,
                }));
            }
        };
        match decoded.disposition {
            DecodeDisposition::Emitted => {}
            DecodeDisposition::KnownIgnored => {
                self.known_ignored_lines = self.known_ignored_lines.saturating_add(1);
            }
            DecodeDisposition::UnknownSkipped => {
                self.unknown_skipped_lines = self.unknown_skipped_lines.saturating_add(1);
            }
        }
        for mut payload in decoded.payloads {
            self.builder.attach_projection(&mut payload);
            match payload {
                KernelEventPayload::Outcome {
                    status, summary, ..
                } => {
                    self.vendor_outcome = Some(VendorOutcome { status, summary });
                }
                KernelEventPayload::PermissionRequested { .. } => {
                    self.commit_payload(payload)?;
                    let summary = match self.safety_capability {
                        RuntimeSafetyCapability::DenyUnapproved
                        | RuntimeSafetyCapability::PermissionPromptTool => {
                            PersistedText::from_reviewed(
                                "runtime permission bridge is not proven live",
                            )
                        }
                    };
                    return Ok(Some(Termination {
                        status: OutcomeStatus::SafetyUnsupported,
                        failure: Some(ExecutionFailure::SafetyUnsupported),
                        detail: None,
                        known_exit: None,
                        summary: Some(summary),
                    }));
                }
                _ => self.commit_payload(payload)?,
            }
        }
        Ok(None)
    }

    fn commit_payload(&mut self, payload: KernelEventPayload) -> Result<(), HostError> {
        let event = match self.host.build_event(&mut self.builder, payload) {
            Ok(event) => event,
            Err(HostError::Clock(source)) => {
                let cleanup = cancel_after_host_failure(self.handle.as_mut());
                return Err(HostError::ClockDuringExecution {
                    source,
                    cleanup: cleanup.status,
                    cleanup_error: cleanup.error,
                });
            }
            Err(other) => return Err(other),
        };
        match self.host.commit_event(&event) {
            Ok(AppendOutcome::Inserted { .. }) => {
                self.committed_events = self.committed_events.saturating_add(1);
                Ok(())
            }
            Ok(AppendOutcome::Duplicate { .. }) => Ok(()),
            Err(source) => {
                let cleanup = cancel_after_host_failure(self.handle.as_mut());
                Err(HostError::ChronicleDuringExecution {
                    source,
                    cleanup: cleanup.status,
                    cleanup_error: cleanup.error,
                })
            }
        }
    }

    fn finish(mut self, termination: Termination) -> Result<ExecutionReport, HostError> {
        let (cleanup, cleanup_error, exit) = match termination.known_exit {
            Some(exit) => (CleanupStatus::Natural(exit), None, Some(exit)),
            None => match self.handle.cancel_and_reap() {
                Ok(exit) => (CleanupStatus::Cancelled(exit), None, Some(exit)),
                Err(error) => (CleanupStatus::Failed, Some(error), None),
            },
        };
        let phase = match termination.status {
            OutcomeStatus::Succeeded => RuntimePhase::Completed,
            OutcomeStatus::Cancelled => RuntimePhase::Cancelled,
            _ => RuntimePhase::Failed,
        };
        self.commit_payload(KernelEventPayload::RuntimeLifecycle { phase })?;
        self.commit_payload(KernelEventPayload::Outcome {
            status: termination.status,
            summary: termination.summary.or_else(|| {
                Some(PersistedText::from_reviewed(status_summary(
                    termination.status,
                )))
            }),
            projection_id: None,
        })?;
        Ok(ExecutionReport {
            started_at: self.started_at,
            status: termination.status,
            process_tree: self.process_tree,
            failure: termination.failure,
            detail: termination.detail,
            cleanup,
            cleanup_error,
            exit,
            committed_events: self.committed_events,
            stderr_lines: self.stderr_lines,
            known_ignored_lines: self.known_ignored_lines,
            unknown_skipped_lines: self.unknown_skipped_lines,
        })
    }
}
