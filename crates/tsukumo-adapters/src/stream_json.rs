//! Claude-like `stream-json` NDJSON → [`KernelEvent`].
//!
//! Documented subset (A1):
//! - `assistant` message `tool_use` blocks → [`KernelEvent::ToolStart`]
//! - `user` message `tool_result` blocks → [`KernelEvent::ToolEnd`]
//! - top-level `tool_use` / `tool_result` (some wrappers) → same
//! - `sdk_control_request` / `control_request` permission → [`KernelEvent::WaitingPermission`]
//! - `result` → [`KernelEvent::TurnOrQuestEnd`] or [`KernelEvent::Error`]
//!
//! Skipped: `system` init, text-only assistant, `stream_event` token deltas, unknown types.
//! Vendor fields never leave this module.

use serde_json::Value;
use std::io::{BufRead, BufReader, Read};
use thiserror::Error;
use tsukumo_kernel::{BackendKind, ExecutorId, KernelEvent, ToolResult};

/// Options for attributing soft identity on produced events.
#[derive(Debug, Clone)]
pub struct StreamJsonOptions {
    pub executor_id: Option<ExecutorId>,
    pub backend: BackendKind,
}

impl Default for StreamJsonOptions {
    fn default() -> Self {
        Self {
            executor_id: None,
            backend: BackendKind::StreamJson,
        }
    }
}

impl StreamJsonOptions {
    pub fn with_executor(mut self, id: impl Into<ExecutorId>) -> Self {
        self.executor_id = Some(id.into());
        self
    }
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error on line {line}: {source}")]
    Json {
        line: usize,
        #[source]
        source: serde_json::Error,
    },
}

/// Parse one NDJSON line. Blank / whitespace → empty vec. Unknown types → empty vec.
pub fn parse_stream_json_line(
    line: &str,
    opts: &StreamJsonOptions,
) -> Result<Vec<KernelEvent>, serde_json::Error> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let value: Value = serde_json::from_str(trimmed)?;
    Ok(map_value(&value, opts))
}

/// Parse a multi-line stream-json document (string body).
pub fn parse_stream_json_str(
    body: &str,
    opts: &StreamJsonOptions,
) -> Result<Vec<KernelEvent>, AdapterError> {
    let mut out = Vec::new();
    for (idx, line) in body.lines().enumerate() {
        let line_no = idx + 1;
        let mut events =
            parse_stream_json_line(line, opts).map_err(|source| AdapterError::Json {
                line: line_no,
                source,
            })?;
        out.append(&mut events);
    }
    Ok(out)
}

/// Parse stream-json from any [`Read`] (file / pipe).
pub fn parse_stream_json_reader<R: Read>(
    reader: R,
    opts: &StreamJsonOptions,
) -> Result<Vec<KernelEvent>, AdapterError> {
    let reader = BufReader::new(reader);
    let mut out = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line_no = idx + 1;
        let line = line?;
        let mut events =
            parse_stream_json_line(&line, opts).map_err(|source| AdapterError::Json {
                line: line_no,
                source,
            })?;
        out.append(&mut events);
    }
    Ok(out)
}

fn map_value(value: &Value, opts: &StreamJsonOptions) -> Vec<KernelEvent> {
    let Some(ty) = value.get("type").and_then(|t| t.as_str()) else {
        return Vec::new();
    };

    match ty {
        "assistant" | "user" => map_message_envelope(value, opts),
        "tool_use" => map_tool_use(value, opts).into_iter().collect(),
        "tool_result" => map_tool_result(value, opts).into_iter().collect(),
        "sdk_control_request" | "control_request" => {
            map_control_request(value, opts).into_iter().collect()
        }
        "result" => map_result(value, opts).into_iter().collect(),
        // system / stream_event / rate_limit_event / …
        _ => Vec::new(),
    }
}

fn map_message_envelope(value: &Value, opts: &StreamJsonOptions) -> Vec<KernelEvent> {
    let Some(content) = value
        .pointer("/message/content")
        .and_then(|c| c.as_array())
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for block in content {
        let Some(block_ty) = block.get("type").and_then(|t| t.as_str()) else {
            continue;
        };
        match block_ty {
            "tool_use" => {
                if let Some(ev) = map_tool_use(block, opts) {
                    out.push(ev);
                }
            }
            "tool_result" => {
                if let Some(ev) = map_tool_result(block, opts) {
                    out.push(ev);
                }
            }
            _ => {}
        }
    }
    out
}

fn map_tool_use(block: &Value, opts: &StreamJsonOptions) -> Option<KernelEvent> {
    let call_id = block
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let tool = block
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let args = block.get("input").cloned();
    Some(KernelEvent::ToolStart {
        call_id,
        tool,
        args,
        executor_id: opts.executor_id.clone(),
        backend: Some(opts.backend),
    })
}

fn map_tool_result(block: &Value, opts: &StreamJsonOptions) -> Option<KernelEvent> {
    let call_id = block
        .get("tool_use_id")
        .or_else(|| block.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let is_error = block
        .get("is_error")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let summary = summarize_tool_result_content(block.get("content"));
    Some(KernelEvent::ToolEnd {
        call_id,
        result: ToolResult::text(summary),
        is_error,
        executor_id: opts.executor_id.clone(),
        backend: Some(opts.backend),
    })
}

fn summarize_tool_result_content(content: Option<&Value>) -> String {
    match content {
        None => "ok".into(),
        Some(Value::String(s)) => truncate(s, 160),
        Some(Value::Array(items)) => {
            let mut parts = Vec::new();
            for item in items {
                if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                    parts.push(t);
                } else if let Some(s) = item.as_str() {
                    parts.push(s);
                }
            }
            if parts.is_empty() {
                "ok".into()
            } else {
                truncate(&parts.join(" "), 160)
            }
        }
        Some(other) => truncate(&other.to_string(), 160),
    }
}

fn map_control_request(value: &Value, opts: &StreamJsonOptions) -> Option<KernelEvent> {
    // Shapes seen in SDK docs:
    // { "type":"sdk_control_request", "request": { "subtype":"permission", ... } }
    // { "type":"control_request", "request_id":"…", "request": { "subtype":"can_use_tool", ... } }
    let request = value.get("request").unwrap_or(value);
    let subtype = request
        .get("subtype")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let is_permission = matches!(
        subtype,
        "permission" | "can_use_tool" | "request_permission" | ""
    ) || value.get("type").and_then(|t| t.as_str()) == Some("sdk_control_request");

    // If subtype is present and clearly not permission-related, skip.
    if !subtype.is_empty()
        && !matches!(
            subtype,
            "permission" | "can_use_tool" | "request_permission"
        )
    {
        return None;
    }
    if !is_permission && subtype.is_empty() {
        return None;
    }

    let request_id = request
        .get("request_id")
        .or_else(|| value.get("request_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("perm")
        .to_string();

    let tool_name = request
        .get("tool_name")
        .or_else(|| request.pointer("/tool_call/name"))
        .and_then(|v| v.as_str())
        .unwrap_or("tool");

    let reason = if let Some(input) = request.get("tool_input").or_else(|| request.get("input")) {
        format!("{tool_name}: {}", compact_json(input))
    } else {
        tool_name.to_string()
    };

    Some(KernelEvent::WaitingPermission {
        request_id,
        reason: truncate(&reason, 200),
        executor_id: opts.executor_id.clone(),
        backend: Some(opts.backend),
    })
}

fn map_result(value: &Value, opts: &StreamJsonOptions) -> Option<KernelEvent> {
    let is_error = value
        .get("is_error")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let subtype = value.get("subtype").and_then(|v| v.as_str()).unwrap_or("");
    let summary = value
        .get("result")
        .and_then(|v| v.as_str())
        .or_else(|| value.get("error").and_then(|v| v.as_str()))
        .unwrap_or(if is_error { "error" } else { "done" });

    let session_id = value
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if is_error || subtype == "error" {
        Some(KernelEvent::Error {
            message: truncate(summary, 240),
            recoverable: false,
            executor_id: opts.executor_id.clone(),
            backend: Some(opts.backend),
        })
    } else {
        Some(KernelEvent::TurnOrQuestEnd {
            quest_id: session_id,
            summary: Some(truncate(summary, 240)),
            executor_id: opts.executor_id.clone(),
            backend: Some(opts.backend),
        })
    }
}

fn compact_json(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Object(map) => {
            if let Some(cmd) = map.get("command").and_then(|v| v.as_str()) {
                return cmd.to_string();
            }
            if let Some(path) = map.get("path").or_else(|| map.get("file_path")).and_then(|v| v.as_str())
            {
                return path.to_string();
            }
            value.to_string()
        }
        other => other.to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> StreamJsonOptions {
        StreamJsonOptions::default().with_executor("gina")
    }

    #[test]
    fn a1_parses_assistant_tool_use() {
        let line = r#"{"type":"assistant","session_id":"s1","message":{"id":"m1","type":"message","role":"assistant","content":[{"type":"tool_use","id":"toolu_1","name":"Bash","input":{"command":"git status"}}]}}"#;
        let evs = parse_stream_json_line(line, &opts()).unwrap();
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            KernelEvent::ToolStart {
                call_id,
                tool,
                backend,
                ..
            } => {
                assert_eq!(call_id, "toolu_1");
                assert_eq!(tool, "Bash");
                assert_eq!(*backend, Some(BackendKind::StreamJson));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn a1_parses_user_tool_result() {
        let line = r#"{"type":"user","session_id":"s1","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"toolu_1","content":"clean"}]}}"#;
        let evs = parse_stream_json_line(line, &opts()).unwrap();
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            KernelEvent::ToolEnd {
                call_id,
                result,
                is_error,
                ..
            } => {
                assert_eq!(call_id, "toolu_1");
                assert_eq!(result.summary, "clean");
                assert!(!is_error);
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn a1_parses_permission_control_request() {
        let line = r#"{"type":"sdk_control_request","request":{"subtype":"permission","request_id":"perm_1","tool_name":"Bash","tool_input":{"command":"rm -rf /tmp/x"}}}"#;
        let evs = parse_stream_json_line(line, &opts()).unwrap();
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            KernelEvent::WaitingPermission {
                request_id,
                reason,
                ..
            } => {
                assert_eq!(request_id, "perm_1");
                assert!(reason.contains("Bash"));
                assert!(reason.contains("rm -rf"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn a1_parses_result_success() {
        let line = r#"{"type":"result","subtype":"success","is_error":false,"session_id":"s1","result":"ok"}"#;
        let evs = parse_stream_json_line(line, &opts()).unwrap();
        assert!(matches!(
            &evs[0],
            KernelEvent::TurnOrQuestEnd {
                quest_id: Some(qid),
                summary: Some(s),
                ..
            } if qid == "s1" && s == "ok"
        ));
    }

    #[test]
    fn a1_skips_system_init() {
        let line = r#"{"type":"system","subtype":"init","session_id":"s1","tools":["Bash"]}"#;
        let evs = parse_stream_json_line(line, &opts()).unwrap();
        assert!(evs.is_empty());
    }
}
