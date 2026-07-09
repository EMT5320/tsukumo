//! Minimal playable [`KernelEvent`] set for Path B / P0.
//!
//! Vendor / ACP / stream-json details stay in adapters — never on these variants.

use crate::identity::{BackendKind, ExecutorId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Normalized tool outcome. Keep this vendor-agnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    /// Short human-readable summary suitable for logs / later stage copy.
    pub summary: String,
    /// Optional structured payload (already normalized by the adapter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl ToolResult {
    pub fn text(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            data: None,
        }
    }
}

/// Upper-layer contract: adapters produce this; theater never sees vendor types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KernelEvent {
    /// A tool / capability invocation began.
    ToolStart {
        call_id: String,
        tool: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        args: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        executor_id: Option<ExecutorId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        backend: Option<BackendKind>,
    },
    /// A tool / capability invocation finished.
    ToolEnd {
        call_id: String,
        result: ToolResult,
        is_error: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        executor_id: Option<ExecutorId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        backend: Option<BackendKind>,
    },
    /// Runtime is blocked on human permission / approval.
    WaitingPermission {
        request_id: String,
        /// Short reason already normalized (e.g. "shell: rm -rf").
        reason: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        executor_id: Option<ExecutorId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        backend: Option<BackendKind>,
    },
    /// Turn or quest completed — enough for settlement / attention reset.
    TurnOrQuestEnd {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        quest_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        summary: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        executor_id: Option<ExecutorId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        backend: Option<BackendKind>,
    },
    /// Recoverable or fatal error surfaced to the host.
    Error {
        message: String,
        recoverable: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        executor_id: Option<ExecutorId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        backend: Option<BackendKind>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{BackendKind, ExecutorId};

    #[test]
    fn tool_start_roundtrips() {
        let ev = KernelEvent::ToolStart {
            call_id: "c1".into(),
            tool: "read".into(),
            args: Some(serde_json::json!({"path": "README.md"})),
            executor_id: Some(ExecutorId::new("gina")),
            backend: Some(BackendKind::Fixture),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: KernelEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ev);
        assert!(json.contains("\"type\":\"tool_start\""));
        assert!(!json.contains("claude"));
        assert!(!json.contains("acp"));
    }

    #[test]
    fn waiting_permission_roundtrips() {
        let ev = KernelEvent::WaitingPermission {
            request_id: "p1".into(),
            reason: "shell: git push".into(),
            executor_id: None,
            backend: None,
        };
        let back: KernelEvent = serde_json::from_str(&serde_json::to_string(&ev).unwrap()).unwrap();
        assert_eq!(back, ev);
    }
}
