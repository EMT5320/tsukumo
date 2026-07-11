//! Chronicle test double with deterministic failure injection.

use super::process::{FakeMetrics, FakeRunner};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tsukumo_host::HostLedger;
use tsukumo_kernel::{ExecutionId, KernelEvent, KernelEventPayload, ProjectionId, VendorEventRef};
use tsukumo_soul::{AppendOutcome, ChronicleQuery, ProjectionReceipt, SoulError, SoulStore};

pub struct TestLedger {
    pub store: SoulStore,
    exit_signal: Arc<FakeMetrics>,
    append_calls: usize,
    fail_on_append: Option<usize>,
    pub append_before_exit: Vec<String>,
}

impl TestLedger {
    pub fn new(store: SoulStore, runner: &FakeRunner) -> Self {
        Self {
            store,
            exit_signal: runner.exit_signal(),
            append_calls: 0,
            fail_on_append: None,
            append_before_exit: Vec::new(),
        }
    }

    pub fn fail_on_append(mut self, call: usize) -> Self {
        self.fail_on_append = Some(call);
        self
    }

    pub fn execution_events(&self, execution_id: &ExecutionId) -> Vec<KernelEvent> {
        self.store
            .replay_events(
                ChronicleQuery::default()
                    .for_execution(execution_id.clone())
                    .limited_to(100),
            )
            .expect("replay host execution")
            .into_iter()
            .map(|persisted| persisted.event)
            .collect()
    }
}

impl HostLedger for TestLedger {
    fn projection_receipt(
        &self,
        projection_id: &ProjectionId,
    ) -> Result<Option<ProjectionReceipt>, SoulError> {
        self.store.projection_receipt(projection_id)
    }

    fn append_event(&mut self, event: &KernelEvent) -> Result<AppendOutcome, SoulError> {
        self.append_calls += 1;
        if self.fail_on_append == Some(self.append_calls) {
            return Err(SoulError::Io(std::io::Error::other(
                "injected Chronicle failure",
            )));
        }
        if !self.exit_signal.exit_emitted.load(Ordering::SeqCst) {
            self.append_before_exit
                .push(payload_name(&event.payload).into());
        }
        self.store.append_event(event)
    }

    fn permission_request_exists(
        &self,
        execution_id: &ExecutionId,
        vendor_request: &VendorEventRef,
    ) -> Result<bool, SoulError> {
        Ok(self.execution_events(execution_id).iter().any(|event| {
            matches!(
                &event.payload,
                KernelEventPayload::PermissionRequested {
                    vendor_request: stored,
                    ..
                } if stored == vendor_request
            )
        }))
    }
}

fn payload_name(payload: &KernelEventPayload) -> &'static str {
    match payload {
        KernelEventPayload::RuntimeLifecycle { .. } => "runtime_lifecycle",
        KernelEventPayload::ToolStart { .. } => "tool_start",
        KernelEventPayload::ToolEnd { .. } => "tool_end",
        KernelEventPayload::Outcome { .. } => "outcome",
        _ => "other",
    }
}
