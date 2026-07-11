//! Stage world: consumes [`StageEvent`] into actor + log state.
//!
//! Director stays pure; this reducer is the theater-side sink (lossy OK).

use crate::stage::{ActorPose, AttentionTier, StageEvent};
use std::collections::VecDeque;
use tsukumo_kernel::SpiritId;

/// Coarse locomotion for S1 animation (Idle / Walk / Work).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Motion {
    Idle,
    Walk,
    Work,
}

impl Motion {
    /// Map a director pose onto the thin locomotion FSM.
    pub fn from_pose(pose: ActorPose) -> Self {
        match pose {
            ActorPose::Walk => Motion::Walk,
            ActorPose::Work => Motion::Work,
            ActorPose::Idle | ActorPose::Wait | ActorPose::Celebrate | ActorPose::Upset => {
                Motion::Idle
            }
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Motion::Idle => "idle",
            Motion::Walk => "walk",
            Motion::Work => "work",
        }
    }
}

/// One spirit ID actor on the workshop floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSlot {
    pub id: SpiritId,
    pub pose: ActorPose,
    pub motion: Motion,
    pub bubble: Option<String>,
    /// Workshop-local x (cells). Walk ticks nudge this.
    pub x: i32,
    pub y: i32,
}

impl ActorSlot {
    pub fn new(id: impl Into<SpiritId>) -> Self {
        Self {
            id: id.into(),
            pose: ActorPose::Idle,
            motion: Motion::Idle,
            bubble: None,
            x: 8,
            y: 4,
        }
    }
}

const DEFAULT_LOG_CAP: usize = 32;
const WORKSHOP_WALK_MIN_X: i32 = 4;
const WORKSHOP_WALK_MAX_X: i32 = 28;

/// Mutable theater state derived only from [`StageEvent`] (+ optional ticks).
#[derive(Debug, Clone)]
pub struct StageWorld {
    pub actors: Vec<ActorSlot>,
    pub attention: AttentionTier,
    pub log: VecDeque<String>,
    pub log_cap: usize,
    /// Walk direction for the primary actor (+1 / -1).
    walk_dir: i32,
}

impl Default for StageWorld {
    fn default() -> Self {
        Self {
            actors: Vec::new(),
            attention: AttentionTier::Ambient,
            log: VecDeque::new(),
            log_cap: DEFAULT_LOG_CAP,
            walk_dir: 1,
        }
    }
}

impl StageWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_log_cap(mut self, cap: usize) -> Self {
        self.log_cap = cap.max(1);
        self
    }

    /// Ensure a default placeholder actor exists (S0 single-sprite workshop).
    pub fn ensure_placeholder(&mut self, id: impl Into<SpiritId>) {
        let id = id.into();
        if self.actors.is_empty() {
            self.actors.push(ActorSlot::new(id));
        }
    }

    pub fn primary(&self) -> Option<&ActorSlot> {
        self.actors.first()
    }

    pub fn primary_mut(&mut self) -> Option<&mut ActorSlot> {
        self.actors.first_mut()
    }

    fn resolve_id(&self, id: Option<&SpiritId>) -> SpiritId {
        id.cloned()
            .or_else(|| self.actors.first().map(|a| a.id.clone()))
            .unwrap_or_else(|| SpiritId::new("anon"))
    }

    fn actor_mut(&mut self, id: &SpiritId) -> &mut ActorSlot {
        if let Some(idx) = self.actors.iter().position(|a| &a.id == id) {
            return &mut self.actors[idx];
        }
        self.actors.push(ActorSlot::new(id.clone()));
        self.actors.last_mut().expect("just pushed")
    }

    fn push_log(&mut self, line: String) {
        if self.log.len() >= self.log_cap {
            self.log.pop_front();
        }
        self.log.push_back(line);
    }

    /// Apply one stage event (lossy: log capped; no blocking).
    pub fn apply(&mut self, event: &StageEvent) {
        match event {
            StageEvent::ActorPose { pose, spirit_id } => {
                let id = self.resolve_id(spirit_id.as_ref());
                let actor = self.actor_mut(&id);
                let prev = actor.motion;
                actor.pose = *pose;
                actor.motion = Motion::from_pose(*pose);
                // Brief walk when entering Work from a non-work pose (S1 spice).
                if actor.motion == Motion::Work && prev != Motion::Work {
                    actor.motion = Motion::Walk;
                }
            }
            StageEvent::Bubble { text, spirit_id } => {
                let id = self.resolve_id(spirit_id.as_ref());
                let actor = self.actor_mut(&id);
                actor.bubble = if text.is_empty() {
                    None
                } else {
                    Some(text.clone())
                };
            }
            StageEvent::LogLine { text, .. } => {
                self.push_log(text.clone());
            }
            StageEvent::AttentionTier { tier } => {
                self.attention = *tier;
            }
        }
    }

    pub fn apply_all<'a, I>(&mut self, events: I)
    where
        I: IntoIterator<Item = &'a StageEvent>,
    {
        for ev in events {
            self.apply(ev);
        }
    }

    /// Advance locomotion one step (walk bounce inside the workshop).
    pub fn tick(&mut self) {
        let Some(actor) = self.actors.first_mut() else {
            return;
        };
        if actor.motion != Motion::Walk {
            return;
        }
        actor.x += self.walk_dir;
        if actor.x >= WORKSHOP_WALK_MAX_X {
            actor.x = WORKSHOP_WALK_MAX_X;
            self.walk_dir = -1;
            // Settled at the workbench → Work.
            if actor.pose == ActorPose::Work {
                actor.motion = Motion::Work;
            } else {
                actor.motion = Motion::Idle;
            }
        } else if actor.x <= WORKSHOP_WALK_MIN_X {
            actor.x = WORKSHOP_WALK_MIN_X;
            self.walk_dir = 1;
            if actor.pose == ActorPose::Work {
                actor.motion = Motion::Work;
            } else {
                actor.motion = Motion::Idle;
            }
        }
    }

    /// Snapshot suitable for golden / assert tests.
    pub fn snapshot(&self) -> StageSnapshot {
        StageSnapshot {
            attention: self.attention,
            log_len: self.log.len(),
            log_tail: self.log.back().cloned(),
            actors: self
                .actors
                .iter()
                .map(|a| ActorSnapshot {
                    id: a.id.as_str().to_string(),
                    pose: a.pose,
                    motion: a.motion,
                    bubble: a.bubble.clone(),
                    x: a.x,
                    y: a.y,
                })
                .collect(),
        }
    }
}

/// Immutable view for tests / demo assertions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageSnapshot {
    pub attention: AttentionTier,
    pub log_len: usize,
    pub log_tail: Option<String>,
    pub actors: Vec<ActorSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSnapshot {
    pub id: String,
    pub pose: ActorPose,
    pub motion: Motion,
    pub bubble: Option<String>,
    pub x: i32,
    pub y: i32,
}

#[cfg(test)]
mod tests;
