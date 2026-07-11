//! Payload-specific durable event validation.

use super::{
    require, require_execution_runtime, require_execution_runtime_correlation, validate_json,
    validate_label, validate_text, validate_vendor_ref, EventContractError,
    MAX_PERSISTED_TEXT_ITEMS,
};
use crate::event::{KernelEvent, KernelEventPayload};

pub(super) fn validate_payload(event: &KernelEvent) -> Result<(), EventContractError> {
    match &event.payload {
        KernelEventPayload::RuntimeLifecycle { .. } => {
            require_execution_runtime(event)?;
        }
        KernelEventPayload::ToolStart {
            vendor_call,
            tool,
            args,
            projection_id,
        } => {
            require_execution_runtime_correlation(event)?;
            require(event, "projection_id", projection_id.is_some())?;
            validate_label(
                event,
                "projection_id",
                projection_id.as_ref().map_or("", |id| id.as_str()),
            )?;
            validate_vendor_ref(event, vendor_call)?;
            validate_label(event, "tool", tool)?;
            validate_json(event, args.as_ref().map(|value| value.as_value()))?;
        }
        KernelEventPayload::ToolEnd {
            vendor_call,
            result,
            projection_id,
            ..
        } => {
            require_execution_runtime_correlation(event)?;
            require(event, "projection_id", projection_id.is_some())?;
            validate_label(
                event,
                "projection_id",
                projection_id.as_ref().map_or("", |id| id.as_str()),
            )?;
            validate_vendor_ref(event, vendor_call)?;
            validate_text(event, &result.summary)?;
            validate_json(event, result.data.as_ref().map(|value| value.as_value()))?;
        }
        KernelEventPayload::PermissionRequested {
            vendor_request,
            tool,
            arguments,
            cwd,
            risk_reasons,
            reason,
        } => {
            require_execution_runtime_correlation(event)?;
            validate_vendor_ref(event, vendor_request)?;
            validate_label(event, "permission.tool", tool)?;
            validate_json(event, arguments.as_ref().map(|value| value.as_value()))?;
            if let Some(cwd) = cwd {
                validate_text(event, cwd)?;
            }
            if risk_reasons.len() > MAX_PERSISTED_TEXT_ITEMS {
                return Err(EventContractError::InvalidField {
                    event_id: event.event_id.clone(),
                    field: "permission.risk_reasons",
                });
            }
            for risk in risk_reasons {
                validate_text(event, risk)?;
            }
            validate_text(event, reason)?;
        }
        KernelEventPayload::PermissionDecided { vendor_request, .. } => {
            require_execution_runtime_correlation(event)?;
            validate_vendor_ref(event, vendor_request)?;
        }
        KernelEventPayload::ProjectionCreated {
            projection_id,
            checkpoint_id,
        } => {
            require_execution_runtime_correlation(event)?;
            validate_label(event, "projection_id", projection_id.as_str())?;
            validate_label(event, "checkpoint_id", checkpoint_id.as_str())?;
        }
        KernelEventPayload::Outcome {
            summary,
            projection_id,
            ..
        } => {
            if let Some(projection_id) = projection_id {
                require_execution_runtime_correlation(event)?;
                validate_label(event, "projection_id", projection_id.as_str())?;
            }
            if let Some(summary) = summary {
                validate_text(event, summary)?;
            }
        }
        KernelEventPayload::UserInput { content } => validate_text(event, content)?,
        KernelEventPayload::LegacyImported {
            source_id,
            kind,
            content,
        } => {
            validate_label(event, "legacy.source_id", source_id)?;
            validate_label(event, "legacy.kind", kind)?;
            validate_text(event, content)?;
        }
        KernelEventPayload::StateLifecycle {
            state_id,
            prior_state_id,
            reason,
            ..
        } => {
            require(event, "causation_id", event.causation_id.is_some())?;
            validate_label(event, "state_id", state_id.as_str())?;
            if let Some(prior_state_id) = prior_state_id {
                validate_label(event, "prior_state_id", prior_state_id.as_str())?;
            }
            if let Some(reason) = reason {
                validate_text(event, reason)?;
            }
        }
        KernelEventPayload::Error { message, .. } => validate_text(event, message)?,
        KernelEventPayload::CheckpointCreated { checkpoint_id, .. } => {
            validate_label(event, "checkpoint_id", checkpoint_id.as_str())?;
        }
        KernelEventPayload::RuntimeSwitched { .. } => {}
    };
    Ok(())
}
