//! Durable state correction and permission-decision actions.

use sha2::{Digest, Sha256};
use tsukumo_kernel::{
    CorrelationId, EventId, KernelEvent, KernelEventPayload, PermissionDecision, PersistedText,
    SpiritId, StateId, StateLifecycleAction, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{StateTransition, StateWriteRequest};
use tsukumo_theater::StageWorld;

use super::read_model::PendingPermission;
use super::{HostProductController, ProductControllerError};
use crate::{
    ExecutionContext, ExecutionPolicy, HostClock, HostServices, Presentation, RuntimeOrchestrator,
    StandardProcessRunner,
};

pub(super) fn revoke_state(
    controller: &mut HostProductController,
    state_id: &StateId,
) -> Result<(), ProductControllerError> {
    let source_spirit_id = controller
        .coordinates
        .spirit_id
        .clone()
        .ok_or(ProductControllerError::MissingSourceSpirit)?;
    let timestamp = controller.clock.now()?;
    controller.event_counter = controller.event_counter.saturating_add(1);
    let source_id = event_id(
        "revoke-source",
        state_id,
        timestamp.as_unix_millis(),
        controller.event_counter,
    );
    let lifecycle_id = event_id(
        "revoke-lifecycle",
        state_id,
        timestamp.as_unix_millis(),
        controller.event_counter,
    );
    let source = event(
        controller,
        &source_spirit_id,
        source_id,
        timestamp,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed(format!(
                "Operator revoked state {state_id} through the TUI"
            )),
        },
        None,
    );
    let lifecycle = event(
        controller,
        &source_spirit_id,
        lifecycle_id,
        timestamp,
        KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Revoked,
            prior_state_id: None,
            reason: Some(PersistedText::from_reviewed(
                "operator revoked state in TUI",
            )),
        },
        Some(source.event_id.clone()),
    );
    controller.store.apply_state(
        StateWriteRequest::new(
            StateTransition::Revoke {
                prior: state_id.clone(),
                evidence: source.event_id.clone(),
                revoked_at: timestamp,
            },
            lifecycle,
        )
        .with_source_event(source),
    )?;
    Ok(())
}

pub(super) fn record_permission(
    controller: &mut HostProductController,
    pending: &PendingPermission,
    decision: PermissionDecision,
) -> Result<(), ProductControllerError> {
    let receipt = pending
        .receipt
        .as_ref()
        .ok_or(ProductControllerError::MissingPermissionReceipt)?;
    let resolution = controller.permissions.decide_scoped(
        &pending.request.scope,
        &pending.request.vendor_request,
        decision,
    )?;
    let runner = StandardProcessRunner;
    let mut world =
        StageWorld::new().with_walk_bounds(controller.walk_bounds.0, controller.walk_bounds.1);
    let mut host = RuntimeOrchestrator::new(
        HostServices::new(&mut controller.store, &runner, &controller.clock),
        Presentation::new(&mut world, &controller.director),
        ExecutionPolicy::default(),
    );
    host.record_permission_resolution(
        receipt,
        ExecutionContext {
            quest_id: pending.coordinates.quest_id.clone(),
            session_id: pending.coordinates.session_id.clone(),
            spirit_id: pending
                .coordinates
                .spirit_id
                .clone()
                .ok_or(ProductControllerError::MissingSourceSpirit)?,
        },
        resolution,
    )?;
    Ok(())
}

fn event(
    controller: &HostProductController,
    source_spirit_id: &SpiritId,
    event_id: EventId,
    occurred_at: tsukumo_kernel::Timestamp,
    payload: KernelEventPayload,
    causation_id: Option<EventId>,
) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id,
        occurred_at,
        quest_id: controller.coordinates.quest_id.clone(),
        session_id: controller.coordinates.session_id.clone(),
        spirit_id: source_spirit_id.clone(),
        execution_id: None,
        runtime: None,
        causation_id,
        correlation_id: Some(CorrelationId::new(format!(
            "tui-state-{}",
            controller.event_counter
        ))),
        payload,
    }
}

fn event_id(prefix: &str, state_id: &StateId, timestamp: i64, counter: u64) -> EventId {
    let mut digest = Sha256::new();
    digest.update(prefix.as_bytes());
    digest.update([0]);
    digest.update(state_id.as_str().as_bytes());
    digest.update(timestamp.to_be_bytes());
    digest.update(counter.to_be_bytes());
    let bytes = digest.finalize();
    let mut value = format!("{prefix}-");
    for byte in bytes.iter().take(12) {
        use std::fmt::Write;
        let _ = write!(&mut value, "{byte:02x}");
    }
    EventId::new(value)
}
