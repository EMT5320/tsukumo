//! Deterministic Host envelope assignment for normalized payloads.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tsukumo_kernel::{
    CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload, ProjectionId, QuestId,
    RuntimeBinding, SessionId, SpiritId, Timestamp, VendorEventRef, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::ProjectionReceipt;

/// Stable quest/session/spirit coordinates supplied by the composition root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContext {
    pub quest_id: QuestId,
    pub session_id: SessionId,
    pub spirit_id: SpiritId,
}

impl ExecutionContext {
    /// Creates envelope coordinates for one execution.
    pub fn new(
        quest_id: impl Into<QuestId>,
        session_id: impl Into<SessionId>,
        spirit_id: impl Into<SpiritId>,
    ) -> Self {
        Self {
            quest_id: quest_id.into(),
            session_id: session_id.into(),
            spirit_id: spirit_id.into(),
        }
    }
}

pub(crate) struct EventBuilder {
    context: ExecutionContext,
    execution_id: ExecutionId,
    runtime: RuntimeBinding,
    projection_id: ProjectionId,
    counter: u64,
    root_event_id: Option<EventId>,
    vendor_causes: HashMap<VendorEventRef, EventId>,
}

impl EventBuilder {
    pub(crate) fn new(context: ExecutionContext, receipt: &ProjectionReceipt) -> Self {
        Self {
            context,
            execution_id: receipt.execution_id.clone(),
            runtime: receipt.runtime.clone(),
            projection_id: receipt.id.clone(),
            counter: 0,
            root_event_id: None,
            vendor_causes: HashMap::new(),
        }
    }

    pub(crate) fn attach_projection(&self, payload: &mut KernelEventPayload) {
        match payload {
            KernelEventPayload::ToolStart { projection_id, .. }
            | KernelEventPayload::ToolEnd { projection_id, .. }
            | KernelEventPayload::Outcome { projection_id, .. } => {
                *projection_id = Some(self.projection_id.clone());
            }
            _ => {}
        }
    }

    pub(crate) fn next(
        &mut self,
        occurred_at: Timestamp,
        mut payload: KernelEventPayload,
    ) -> KernelEvent {
        self.attach_projection(&mut payload);
        self.counter = self.counter.saturating_add(1);
        let event_id = EventId::new(digest_identifier(
            "event",
            &self.execution_id,
            self.counter,
            None,
        ));
        let vendor = vendor_reference(&payload).cloned();
        let correlation_id = Some(CorrelationId::new(match vendor.as_ref() {
            Some(reference) => {
                digest_identifier("correlation", &self.execution_id, 0, Some(reference))
            }
            None => digest_identifier("correlation", &self.execution_id, 0, None),
        }));
        let causation_id = vendor
            .as_ref()
            .and_then(|reference| self.vendor_causes.get(reference).cloned())
            .or_else(|| self.root_event_id.clone());
        let event = KernelEvent {
            schema_version: KERNEL_EVENT_SCHEMA_VERSION,
            event_id: event_id.clone(),
            occurred_at,
            quest_id: self.context.quest_id.clone(),
            session_id: self.context.session_id.clone(),
            spirit_id: self.context.spirit_id.clone(),
            execution_id: Some(self.execution_id.clone()),
            runtime: Some(self.runtime.clone()),
            causation_id,
            correlation_id,
            payload,
        };
        if self.root_event_id.is_none()
            && matches!(
                event.payload,
                KernelEventPayload::RuntimeLifecycle {
                    phase: tsukumo_kernel::RuntimePhase::Starting
                }
            )
        {
            self.root_event_id = Some(event_id.clone());
        }
        if matches!(
            event.payload,
            KernelEventPayload::ToolStart { .. } | KernelEventPayload::PermissionRequested { .. }
        ) {
            if let Some(reference) = vendor {
                self.vendor_causes.insert(reference, event_id);
            }
        }
        event
    }
    pub(crate) fn permission_decision(
        &self,
        occurred_at: Timestamp,
        payload: KernelEventPayload,
    ) -> KernelEvent {
        let vendor = vendor_reference(&payload);
        let event_id = EventId::new(digest_identifier(
            "permission-decision",
            &self.execution_id,
            0,
            vendor,
        ));
        let correlation_id = Some(CorrelationId::new(digest_identifier(
            "correlation",
            &self.execution_id,
            0,
            vendor,
        )));
        KernelEvent {
            schema_version: KERNEL_EVENT_SCHEMA_VERSION,
            event_id,
            occurred_at,
            quest_id: self.context.quest_id.clone(),
            session_id: self.context.session_id.clone(),
            spirit_id: self.context.spirit_id.clone(),
            execution_id: Some(self.execution_id.clone()),
            runtime: Some(self.runtime.clone()),
            causation_id: None,
            correlation_id,
            payload,
        }
    }
}

fn vendor_reference(payload: &KernelEventPayload) -> Option<&VendorEventRef> {
    match payload {
        KernelEventPayload::ToolStart { vendor_call, .. }
        | KernelEventPayload::ToolEnd { vendor_call, .. } => Some(vendor_call),
        KernelEventPayload::PermissionRequested { vendor_request, .. }
        | KernelEventPayload::PermissionDecided { vendor_request, .. } => Some(vendor_request),
        _ => None,
    }
}

fn digest_identifier(
    prefix: &str,
    execution_id: &ExecutionId,
    counter: u64,
    vendor: Option<&VendorEventRef>,
) -> String {
    let mut digest = Sha256::new();
    digest.update(prefix.as_bytes());
    digest.update([0]);
    digest.update(execution_id.as_str().as_bytes());
    digest.update([0]);
    digest.update(counter.to_be_bytes());
    if let Some(reference) = vendor {
        digest.update([0]);
        digest.update(reference.namespace.as_bytes());
        digest.update([0]);
        digest.update(reference.id.as_bytes());
    }
    let bytes = digest.finalize();
    let mut value = String::with_capacity(prefix.len() + 1 + bytes.len() * 2);
    value.push_str(prefix);
    value.push('-');
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        value.push(char::from(HEX[usize::from(byte >> 4)]));
        value.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    value
}
