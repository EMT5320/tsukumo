//! Minimal [`StageEvent`] set for P0. Theater never sees vendor payloads.

use serde::{Deserialize, Serialize};
use tsukumo_kernel::ExecutorId;

/// Coarse actor body language for the pixel stage (S1 will animate these).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorPose {
    Idle,
    Walk,
    Work,
    Wait,
    Celebrate,
    Upset,
}

/// Attention ladder for the host UI (log highlight / stage urgency).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionTier {
    Ambient,
    Focus,
    Urgent,
}

/// What the theater (and log pane) consume. Same stream, different sinks later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StageEvent {
    ActorPose {
        pose: ActorPose,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        executor_id: Option<ExecutorId>,
    },
    Bubble {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        executor_id: Option<ExecutorId>,
    },
    LogLine {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        executor_id: Option<ExecutorId>,
    },
    AttentionTier {
        tier: AttentionTier,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_event_json_has_no_vendor_keys() {
        let ev = StageEvent::Bubble {
            text: "working…".into(),
            executor_id: Some(ExecutorId::new("gina")),
        };
        let json = serde_json::to_string(&ev).unwrap();
        assert!(json.contains("\"type\":\"bubble\""));
        assert!(!json.contains("claude"));
        assert!(!json.contains("acp"));
        assert!(!json.contains("stream_json"));
    }
}
