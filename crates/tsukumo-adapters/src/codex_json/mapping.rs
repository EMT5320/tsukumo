//! Codex item field validation and kernel payload mapping.

use crate::stream_json::DecodeError;
use crate::vendor_fields::{required_i64, required_label, required_string, truncate};
use serde_json::{json, Value};
use tsukumo_kernel::{
    redact_sensitive_text, KernelEventPayload, PersistedJson, PersistedText, ToolResult,
    VendorEventRef,
};

const COMMAND_EVENT: &str = "command_execution";

pub(super) struct ItemIdentity<'a> {
    pub id: String,
    pub kind: &'a str,
    pub value: &'a Value,
}

pub(super) fn item_identity<'a>(
    value: &'a Value,
    event_type: &'static str,
) -> Result<ItemIdentity<'a>, DecodeError> {
    let item = value
        .get("item")
        .ok_or_else(|| DecodeError::missing(event_type, "item"))?;
    if !item.is_object() {
        return Err(DecodeError::invalid(event_type, "item"));
    }
    let id = required_label(item, "id", event_type)?;
    let kind = required_string(item, "type", event_type)?;
    Ok(ItemIdentity {
        id,
        kind,
        value: item,
    })
}

pub(super) fn map_command_start(
    item: &Value,
    thread_id: &str,
    item_id: &str,
) -> Result<KernelEventPayload, DecodeError> {
    let status = required_string(item, "status", "item.started(command_execution)")?;
    if status != "in_progress" {
        return Err(DecodeError::unsupported(
            "item.started(command_execution)",
            "status",
            status,
        ));
    }
    let command = required_string(item, "command", "item.started(command_execution)")?;
    Ok(KernelEventPayload::ToolStart {
        vendor_call: vendor_call(thread_id, item_id),
        tool: COMMAND_EVENT.into(),
        args: Some(PersistedJson::from_untrusted(&json!({"command": command}))),
        projection_id: None,
    })
}

pub(super) fn map_command_end(
    item: &Value,
    thread_id: &str,
    item_id: &str,
) -> Result<KernelEventPayload, DecodeError> {
    let event = "item.completed(command_execution)";
    let status = required_string(item, "status", event)?;
    if !matches!(status, "completed" | "failed" | "declined") {
        return Err(DecodeError::unsupported(event, "status", status));
    }
    let exit_code = required_i64(item, "exit_code", event)?;
    let output = required_string(item, "aggregated_output", event)?;
    let _command = required_string(item, "command", event)?;
    let fallback = format!("command {status} with exit code {exit_code}");
    let summary = if output.trim().is_empty() {
        fallback
    } else {
        truncate(&redact_sensitive_text(output), 240)
    };
    Ok(KernelEventPayload::ToolEnd {
        vendor_call: vendor_call(thread_id, item_id),
        result: ToolResult {
            summary: PersistedText::from_redacted(summary),
            data: Some(PersistedJson::from_untrusted(&json!({
                "status": status,
                "exit_code": exit_code,
            }))),
        },
        is_error: status != "completed" || exit_code != 0,
        projection_id: None,
    })
}

pub(super) fn is_documented_ignored_item(kind: &str) -> bool {
    matches!(
        kind,
        "agent_message"
            | "reasoning"
            | "file_change"
            | "mcp_tool_call"
            | "web_search"
            | "plan_update"
    )
}

fn vendor_call(thread_id: &str, item_id: &str) -> VendorEventRef {
    VendorEventRef::new("codex_cli", format!("{thread_id}:{item_id}"))
}
