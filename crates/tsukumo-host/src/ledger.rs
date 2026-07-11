//! Durable Host ledger port over Soul Chronicle and projection receipts.

use tsukumo_kernel::{ExecutionId, KernelEvent, KernelEventPayload, ProjectionId, VendorEventRef};
use tsukumo_soul::{AppendOutcome, ChronicleQuery, ProjectionReceipt, SoulError, SoulStore};

const PERMISSION_SCAN_PAGE: usize = 1_000;

/// Minimum durable authority required by runtime orchestration.
pub trait HostLedger {
    fn projection_receipt(
        &self,
        projection_id: &ProjectionId,
    ) -> Result<Option<ProjectionReceipt>, SoulError>;

    fn append_event(&mut self, event: &KernelEvent) -> Result<AppendOutcome, SoulError>;

    fn permission_request_exists(
        &self,
        execution_id: &ExecutionId,
        vendor_request: &VendorEventRef,
    ) -> Result<bool, SoulError>;
}

impl HostLedger for SoulStore {
    fn projection_receipt(
        &self,
        projection_id: &ProjectionId,
    ) -> Result<Option<ProjectionReceipt>, SoulError> {
        SoulStore::projection_receipt(self, projection_id)
    }

    fn append_event(&mut self, event: &KernelEvent) -> Result<AppendOutcome, SoulError> {
        SoulStore::append_event(self, event)
    }

    fn permission_request_exists(
        &self,
        execution_id: &ExecutionId,
        vendor_request: &VendorEventRef,
    ) -> Result<bool, SoulError> {
        let mut after = 0;
        loop {
            let page = SoulStore::replay_events(
                self,
                ChronicleQuery::default()
                    .for_execution(execution_id.clone())
                    .after(after)
                    .limited_to(PERMISSION_SCAN_PAGE),
            )?;
            if page.iter().any(|persisted| {
                matches!(
                    &persisted.event.payload,
                    KernelEventPayload::PermissionRequested {
                        vendor_request: stored,
                        ..
                    } if stored == vendor_request
                )
            }) {
                return Ok(true);
            }
            let Some(last) = page.last() else {
                return Ok(false);
            };
            after = last.sequence;
            if page.len() < PERMISSION_SCAN_PAGE {
                return Ok(false);
            }
        }
    }
}
