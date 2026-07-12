//! Immutable stage views used by contract tests and debug surfaces.

use super::{ActorSlot, Motion};
use crate::stage::{ActorPose, AttentionTier};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageSnapshot {
    pub attention: AttentionTier,
    pub log_len: usize,
    pub log_tail: Option<String>,
    pub log_source: Option<String>,
    pub actors: Vec<ActorSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSnapshot {
    pub id: String,
    pub source_spirit_id: Option<String>,
    pub pose: ActorPose,
    pub motion: Motion,
    pub bubble: Option<String>,
    pub x: i32,
    pub y: i32,
}

impl From<&ActorSlot> for ActorSnapshot {
    fn from(actor: &ActorSlot) -> Self {
        Self {
            id: actor.id.as_str().to_owned(),
            source_spirit_id: actor
                .source_spirit_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            pose: actor.pose,
            motion: actor.motion,
            bubble: actor.bubble.clone(),
            x: actor.x,
            y: actor.y,
        }
    }
}
