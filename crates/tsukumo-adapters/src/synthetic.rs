//! Synthetic / demo stream-json producer for CI (no Claude install required).
//!
//! Emits a documented Claude-like NDJSON subset including `waiting_permission`
//! via `sdk_control_request`, so F1 can be engineered against a stable fixture.

use crate::stream_json::{parse_stream_json_str, StreamJsonOptions};
use tsukumo_kernel::{BackendKind, ExecutorId, KernelEvent};

/// Hard-coded Claude-like NDJSON used by demos and integration tests.
pub fn synthetic_demo_stream_jsonl() -> &'static str {
    r#"{"type":"system","subtype":"init","session_id":"synth-a1","tools":["Bash","Read"],"permissionMode":"default"}
{"type":"assistant","session_id":"synth-a1","message":{"id":"m1","type":"message","role":"assistant","content":[{"type":"text","text":"Checking the workshop."},{"type":"tool_use","id":"toolu_read","name":"Read","input":{"path":"DESIGN.md"}}]}}
{"type":"user","session_id":"synth-a1","message":{"id":"m2","type":"message","role":"user","content":[{"type":"tool_result","tool_use_id":"toolu_read","content":"north-star design, 797 lines"}]}}
{"type":"assistant","session_id":"synth-a1","message":{"id":"m3","type":"message","role":"assistant","content":[{"type":"tool_use","id":"toolu_bash","name":"Bash","input":{"command":"git status"}}]}}
{"type":"sdk_control_request","request":{"subtype":"permission","request_id":"perm_synth_1","tool_name":"Bash","tool_input":{"command":"git status"}}}
{"type":"user","session_id":"synth-a1","message":{"id":"m4","type":"message","role":"user","content":[{"type":"tool_result","tool_use_id":"toolu_bash","content":"clean working tree"}]}}
{"type":"result","subtype":"success","is_error":false,"session_id":"synth-a1","result":"synthetic A1 quest complete","num_turns":2}
"#
}

/// Parse the synthetic stream into [`KernelEvent`]s with soft identity attached.
pub fn synthetic_demo_events(executor_id: impl Into<ExecutorId>) -> Vec<KernelEvent> {
    let opts = StreamJsonOptions {
        executor_id: Some(executor_id.into()),
        backend: BackendKind::StreamJson,
    };
    parse_stream_json_str(synthetic_demo_stream_jsonl(), &opts)
        .expect("synthetic fixture must parse")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a1_synthetic_includes_waiting_permission() {
        let events = synthetic_demo_events("gina");
        assert!(
            events
                .iter()
                .any(|e| matches!(e, KernelEvent::WaitingPermission { .. })),
            "events={events:?}"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, KernelEvent::ToolStart { tool, .. } if tool == "Bash"))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, KernelEvent::TurnOrQuestEnd { .. }))
        );
        // system/init + text blocks skipped — still several playable events.
        assert!(events.len() >= 5, "got {}", events.len());
    }
}
