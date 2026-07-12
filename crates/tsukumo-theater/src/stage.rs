//! Presentation events emitted by the pure Director.

use crate::pack::PresentationActorId;
use serde::{Deserialize, Serialize};
use tsukumo_kernel::SpiritId;

/// Coarse actor body language for the pixel stage.
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

/// Attention ladder for the terminal product surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttentionTier {
    Ambient,
    Focus,
    Urgent,
}

/// Keeps the visible presentation actor separate from the factual executor.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StageAttribution {
    pub actor_id: PresentationActorId,
    pub source_spirit_id: SpiritId,
}

/// Lossy presentation events consumed by the stage and factual log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StageEvent {
    ActorPose {
        pose: ActorPose,
        attribution: StageAttribution,
    },
    Bubble {
        text: String,
        attribution: StageAttribution,
    },
    LogLine {
        text: String,
        attribution: StageAttribution,
    },
    AttentionTier {
        tier: AttentionTier,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attribution() -> StageAttribution {
        StageAttribution {
            actor_id: PresentationActorId::try_from("companion").expect("valid actor id"),
            source_spirit_id: SpiritId::new("gina"),
        }
    }

    #[test]
    fn stage_event_json_when_attributed_has_no_vendor_keys() {
        // Given: a neutral attributed bubble.
        let event = StageEvent::Bubble {
            text: "working...".into(),
            attribution: attribution(),
        };

        // When: theater output is serialized.
        let json = serde_json::to_string(&event).expect("serialize stage event");

        // Then: actor and source facts remain without vendor protocol names.
        assert!(json.contains("\"type\":\"bubble\""));
        assert!(json.contains("\"actor_id\":\"companion\""));
        assert!(json.contains("\"source_spirit_id\":\"gina\""));
        assert!(!json.contains("claude"));
        assert!(!json.contains("stream_json"));
    }
}
