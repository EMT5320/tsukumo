//! Projection receipt selection and bounded inspector view assembly.

use tsukumo_kernel::{ExecutionId, KernelEventPayload, ProjectionId};
use tsukumo_soul::{ProjectionOmissionReason, ProjectionReceipt, SoulStore};
use tsukumo_theater::{DisplayText, ProductView, ProjectionStateRefView, ProjectionView};

use crate::ProductControllerError;

pub(super) fn latest_projection(
    store: &SoulStore,
) -> Result<Option<ProjectionReceipt>, ProductControllerError> {
    let projection_id = store
        .latest_projection_event(None)?
        .and_then(|item| projection_id(&item.event.payload));
    projection_id
        .map(|id| store.projection_receipt(&id))
        .transpose()
        .map(Option::flatten)
        .map_err(Into::into)
}
pub(super) fn projection_id(payload: &KernelEventPayload) -> Option<ProjectionId> {
    match payload {
        KernelEventPayload::ProjectionCreated { projection_id, .. } => Some(projection_id.clone()),
        KernelEventPayload::ToolStart { projection_id, .. }
        | KernelEventPayload::ToolEnd { projection_id, .. }
        | KernelEventPayload::Outcome { projection_id, .. } => projection_id.clone(),
        _ => None,
    }
}

pub(super) fn receipt_for_execution(
    store: &SoulStore,
    execution_id: &ExecutionId,
) -> Result<Option<ProjectionReceipt>, ProductControllerError> {
    let projection = store
        .latest_projection_event(Some(execution_id))?
        .and_then(|item| projection_id(&item.event.payload));
    projection
        .map(|id| store.projection_receipt(&id))
        .transpose()
        .map(Option::flatten)
        .map_err(Into::into)
}
const PROJECTION_SELECTED_CAP: usize = 256;
const PROJECTION_OMISSION_CAP: usize = 256;

pub(super) fn apply_projection(
    view: &mut ProductView,
    receipt: Option<&ProjectionReceipt>,
    checkpoint_version: Option<u64>,
) {
    let Some(receipt) = receipt else {
        return;
    };
    if view.handoff.checkpoint_id.is_none() {
        view.handoff.checkpoint_id = Some(receipt.checkpoint_id.clone());
        view.handoff.version = checkpoint_version;
    }
    view.handoff.projection_id = Some(receipt.id.clone());
    view.handoff.selected_count = receipt.selected_state_refs.len();
    view.handoff.omitted_count = receipt.omissions.len();
    view.handoff.budget_used = receipt.budget.used;
    view.handoff.budget_limit = receipt.budget.limit;
    view.projection = Some(ProjectionView {
        projection_id: receipt.id.clone(),
        checkpoint_id: receipt.checkpoint_id.clone(),
        projection_version: u64::from(receipt.projection_version),
        renderer_version: u64::from(receipt.renderer_version),
        checkpoint_version,
        selected_total: receipt.selected_state_refs.len(),
        omissions_total: receipt.omissions.len(),
        selected_refs: receipt
            .selected_state_refs
            .iter()
            .take(PROJECTION_SELECTED_CAP)
            .map(|state| ProjectionStateRefView {
                state_id: state.state_id.clone(),
                version: state.version,
            })
            .collect(),
        omissions: receipt
            .omissions
            .iter()
            .take(PROJECTION_OMISSION_CAP)
            .map(|omission| {
                DisplayText::from_untrusted(&format!(
                    "{}: {}",
                    omission.state_id,
                    omission_label(omission.reason)
                ))
            })
            .collect(),
        budget_used: receipt.budget.used,
        budget_limit: receipt.budget.limit,
    });
}

const fn omission_label(reason: ProjectionOmissionReason) -> &'static str {
    match reason {
        ProjectionOmissionReason::ExcludedByComparison => "comparison excluded",
        ProjectionOmissionReason::ScopeMismatch => "scope mismatch",
        ProjectionOmissionReason::Inactive => "inactive",
        ProjectionOmissionReason::BudgetExceeded => "budget exceeded",
    }
}
