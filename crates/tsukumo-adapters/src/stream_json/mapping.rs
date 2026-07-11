//! Claude-specific value mapping.
//!
//! Raw vendor keys stay in this module. Every returned value is bounded,
//! redacted, and provider-neutral.

mod fields;

use super::DecodeError;
use fields::{
    optional_bool, optional_string, optional_string_array, required_bool, required_label,
    required_string, reviewed_label, truncate,
};
use serde_json::Value;
use tsukumo_kernel::{
    redact_sensitive_text, KernelEventPayload, OutcomeStatus, PersistedJson, PersistedText,
    ToolResult, VendorEventRef,
};

const VENDOR_NAMESPACE: &str = "claude_cli";

pub(super) fn map_value(value: &Value) -> Result<Vec<KernelEventPayload>, DecodeError> {
    let Some(event_type) = value.get("type").and_then(Value::as_str) else {
        return Ok(Vec::new());
    };

    match event_type {
        "assistant" => map_message_envelope(value, "assistant"),
        "user" => map_message_envelope(value, "user"),
        "tool_use" => Ok(vec![map_tool_use(value)?]),
        "tool_result" => Ok(vec![map_tool_result(value)?]),
        "sdk_control_request" | "control_request" => {
            Ok(map_control_request(value)?.into_iter().collect())
        }
        "result" => Ok(vec![map_result(value)?]),
        // System initialization, token deltas, rate limits, and unknown future
        // events do not represent normalized product facts in the C1 contract.
        _ => Ok(Vec::new()),
    }
}

fn map_message_envelope(
    value: &Value,
    event_type: &'static str,
) -> Result<Vec<KernelEventPayload>, DecodeError> {
    let content = value
        .pointer("/message/content")
        .and_then(Value::as_array)
        .ok_or_else(|| DecodeError::missing(event_type, "message.content"))?;

    let mut payloads = Vec::new();
    for block in content {
        match block.get("type").and_then(Value::as_str) {
            Some("tool_use") => payloads.push(map_tool_use(block)?),
            Some("tool_result") => payloads.push(map_tool_result(block)?),
            // Text and unknown content blocks are forward-compatible noise for
            // the tool/outcome boundary owned by this decoder.
            Some(_) | None => {}
        }
    }
    Ok(payloads)
}

fn map_tool_use(block: &Value) -> Result<KernelEventPayload, DecodeError> {
    let call_id = required_label(block, "id", "tool_use")?;
    let tool = required_label(block, "name", "tool_use")?;

    Ok(KernelEventPayload::ToolStart {
        vendor_call: VendorEventRef::new(VENDOR_NAMESPACE, call_id),
        tool,
        args: block.get("input").map(PersistedJson::from_untrusted),
        projection_id: None,
    })
}

fn map_tool_result(block: &Value) -> Result<KernelEventPayload, DecodeError> {
    let raw_call_id = block
        .get("tool_use_id")
        .or_else(|| block.get("id"))
        .and_then(Value::as_str)
        .ok_or_else(|| DecodeError::missing("tool_result", "tool_use_id"))?;
    let call_id = reviewed_label(raw_call_id, "tool_result", "tool_use_id")?;
    let is_error = optional_bool(block, "is_error", "tool_result")?.unwrap_or(false);
    let summary = redact_sensitive_text(&summarize_tool_result_content(block.get("content")));

    Ok(KernelEventPayload::ToolEnd {
        vendor_call: VendorEventRef::new(VENDOR_NAMESPACE, call_id),
        result: ToolResult::reviewed_text(truncate(&summary, 160)),
        is_error,
        projection_id: None,
    })
}

fn map_control_request(value: &Value) -> Result<Option<KernelEventPayload>, DecodeError> {
    let request = value.get("request").unwrap_or(value);
    let subtype = optional_string(request, "subtype", "permission_request")?.unwrap_or("");
    if !subtype.is_empty()
        && !matches!(
            subtype,
            "permission" | "can_use_tool" | "request_permission"
        )
    {
        return Ok(None);
    }

    let request_id = request
        .get("request_id")
        .or_else(|| value.get("request_id"))
        .and_then(Value::as_str)
        .ok_or_else(|| DecodeError::missing("permission_request", "request_id"))
        .and_then(|value| reviewed_label(value, "permission_request", "request_id"))?;
    let tool = request
        .get("tool_name")
        .or_else(|| request.pointer("/tool_call/name"))
        .and_then(Value::as_str)
        .ok_or_else(|| DecodeError::missing("permission_request", "tool_name"))
        .and_then(|value| reviewed_label(value, "permission_request", "tool_name"))?;
    let input = request.get("tool_input").or_else(|| request.get("input"));
    let reason = input.map_or_else(
        || tool.clone(),
        |input| format!("{tool}: {}", compact_json(input)),
    );
    let cwd = optional_string(request, "cwd", "permission_request")?
        .map(|value| PersistedText::from_redacted(truncate(value, 512)));
    let risk_reasons = optional_string_array(request, "risk_reasons", "permission_request")?
        .into_iter()
        .map(|reason| PersistedText::from_redacted(truncate(reason, 240)))
        .collect();

    Ok(Some(KernelEventPayload::PermissionRequested {
        vendor_request: VendorEventRef::new(VENDOR_NAMESPACE, request_id),
        tool,
        arguments: input.map(PersistedJson::from_untrusted),
        cwd,
        risk_reasons,
        reason: PersistedText::from_redacted(truncate(&reason, 200)),
    }))
}

fn map_result(value: &Value) -> Result<KernelEventPayload, DecodeError> {
    let subtype = required_string(value, "subtype", "result")?;
    let is_error = required_bool(value, "is_error", "result")?;
    let status = match subtype {
        "success" if !is_error => OutcomeStatus::Succeeded,
        "error"
        | "error_during_execution"
        | "error_max_turns"
        | "error_max_budget_usd"
        | "error_max_structured_output_retries" => OutcomeStatus::Failed,
        "success" => return Err(DecodeError::invalid("result", "is_error")),
        other => return Err(DecodeError::unsupported("result", "subtype", other)),
    };
    let summary = optional_string(value, "result", "result")?
        .or(optional_string(value, "error", "result")?)
        .map(|text| PersistedText::from_redacted(truncate(text, 240)));

    Ok(KernelEventPayload::Outcome {
        status,
        summary,
        projection_id: None,
    })
}

fn summarize_tool_result_content(content: Option<&Value>) -> String {
    match content {
        None => "ok".into(),
        Some(Value::String(text)) => truncate(text, 160),
        Some(Value::Array(items)) => {
            let parts = items
                .iter()
                .filter_map(|item| {
                    item.get("text")
                        .and_then(Value::as_str)
                        .or_else(|| item.as_str())
                })
                .collect::<Vec<_>>();
            if parts.is_empty() {
                "ok".into()
            } else {
                truncate(&parts.join(" "), 160)
            }
        }
        Some(other) => truncate(&other.to_string(), 160),
    }
}

fn compact_json(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Object(map) => map
            .get("command")
            .or_else(|| map.get("path"))
            .or_else(|| map.get("file_path"))
            .and_then(Value::as_str)
            .map_or_else(|| value.to_string(), str::to_owned),
        other => other.to_string(),
    }
}
