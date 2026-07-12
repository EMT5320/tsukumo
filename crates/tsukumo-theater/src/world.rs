//! Stage world reduced from attributed presentation events.

mod snapshot;

pub use snapshot::{ActorSnapshot, StageSnapshot};

use crate::pack::PresentationActorId;
use crate::stage::{ActorPose, AttentionTier, StageAttribution, StageEvent};
use std::collections::VecDeque;
use tsukumo_kernel::SpiritId;

/// Coarse locomotion for terminal animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Motion {
    Idle,
    Walk,
    Work,
}

impl Motion {
    pub const fn from_pose(pose: ActorPose) -> Self {
        match pose {
            ActorPose::Walk => Motion::Walk,
            ActorPose::Work => Motion::Work,
            ActorPose::Idle | ActorPose::Wait | ActorPose::Celebrate | ActorPose::Upset => {
                Motion::Idle
            }
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Motion::Idle => "idle",
            Motion::Walk => "walk",
            Motion::Work => "work",
        }
    }
}

/// One visible presentation actor on the workshop floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorSlot {
    pub id: PresentationActorId,
    pub source_spirit_id: Option<SpiritId>,
    pub pose: ActorPose,
    pub motion: Motion,
    pub bubble: Option<String>,
    pub x: i32,
    pub y: i32,
}

impl ActorSlot {
    pub fn new(id: PresentationActorId) -> Self {
        Self {
            id,
            source_spirit_id: None,
            pose: ActorPose::Idle,
            motion: Motion::Idle,
            bubble: None,
            x: 8,
            y: 4,
        }
    }
}

/// One bounded factual log entry with source attribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageLogLine {
    pub text: String,
    pub attribution: StageAttribution,
}

const DEFAULT_LOG_CAP: usize = 32;
const WORKSHOP_WALK_MIN_X: i32 = 4;
const WORKSHOP_WALK_MAX_X: i32 = 28;

/// Mutable presentation state derived only from StageEvent values and ticks.
#[derive(Debug, Clone)]
pub struct StageWorld {
    pub actors: Vec<ActorSlot>,
    pub attention: AttentionTier,
    pub log: VecDeque<StageLogLine>,
    pub log_cap: usize,
    walk_dir: i32,
    walk_min_x: i32,
    walk_max_x: i32,
}

impl Default for StageWorld {
    fn default() -> Self {
        Self {
            actors: Vec::new(),
            attention: AttentionTier::Ambient,
            log: VecDeque::new(),
            log_cap: DEFAULT_LOG_CAP,
            walk_dir: 1,
            walk_min_x: WORKSHOP_WALK_MIN_X,
            walk_max_x: WORKSHOP_WALK_MAX_X,
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

    /// Configures the same walk corridor supplied by the validated scene pack.
    pub fn with_walk_bounds(mut self, minimum: i32, maximum: i32) -> Self {
        self.walk_min_x = minimum.min(maximum);
        self.walk_max_x = maximum.max(minimum);
        let midpoint = self.walk_midpoint();
        for actor in &mut self.actors {
            actor.x = actor.x.clamp(self.walk_min_x, self.walk_max_x);
            if actor.motion == Motion::Idle {
                actor.x = midpoint;
            }
        }
        self
    }

    pub fn ensure_placeholder(&mut self, actor_id: PresentationActorId) {
        if self.actors.is_empty() {
            let mut actor = ActorSlot::new(actor_id);
            actor.x = self.walk_midpoint();
            self.actors.push(actor);
        }
    }

    pub fn primary(&self) -> Option<&ActorSlot> {
        self.actors.first()
    }

    pub fn primary_mut(&mut self) -> Option<&mut ActorSlot> {
        self.actors.first_mut()
    }

    fn actor_mut(&mut self, attribution: &StageAttribution) -> &mut ActorSlot {
        let index = match self
            .actors
            .iter()
            .position(|actor| actor.id == attribution.actor_id)
        {
            Some(index) => index,
            None => {
                let mut actor = ActorSlot::new(attribution.actor_id.clone());
                actor.x = self.walk_midpoint();
                self.actors.push(actor);
                self.actors.len() - 1
            }
        };
        let actor = &mut self.actors[index];
        actor.source_spirit_id = Some(attribution.source_spirit_id.clone());
        actor
    }

    fn push_log(&mut self, line: StageLogLine) {
        if self.log.len() >= self.log_cap {
            self.log.pop_front();
        }
        self.log.push_back(line);
    }

    /// Applies one lossy presentation event without blocking execution.
    pub fn apply(&mut self, event: &StageEvent) {
        match event {
            StageEvent::ActorPose { pose, attribution } => {
                let actor = self.actor_mut(attribution);
                let previous = actor.motion;
                actor.pose = *pose;
                actor.motion = Motion::from_pose(*pose);
                if actor.motion == Motion::Work && previous != Motion::Work {
                    actor.motion = Motion::Walk;
                }
            }
            StageEvent::Bubble { text, attribution } => {
                let actor = self.actor_mut(attribution);
                actor.bubble = if text.is_empty() {
                    None
                } else {
                    Some(text.clone())
                };
            }
            StageEvent::LogLine { text, attribution } => {
                self.actor_mut(attribution);
                self.push_log(StageLogLine {
                    text: text.clone(),
                    attribution: attribution.clone(),
                });
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
        for event in events {
            self.apply(event);
        }
    }

    /// Advances the primary actor one deterministic walk step.
    pub fn tick(&mut self) -> bool {
        let Some(actor) = self.actors.first_mut() else {
            return false;
        };
        if actor.motion != Motion::Walk {
            return false;
        }
        let before = (actor.x, actor.motion);
        actor.x += self.walk_dir;
        if actor.x >= self.walk_max_x {
            actor.x = self.walk_max_x;
            self.walk_dir = -1;
            actor.motion = if actor.pose == ActorPose::Work {
                Motion::Work
            } else {
                Motion::Idle
            };
        } else if actor.x <= self.walk_min_x {
            actor.x = self.walk_min_x;
            self.walk_dir = 1;
            actor.motion = if actor.pose == ActorPose::Work {
                Motion::Work
            } else {
                Motion::Idle
            };
        }
        before != (actor.x, actor.motion)
    }

    const fn walk_midpoint(&self) -> i32 {
        self.walk_min_x + (self.walk_max_x - self.walk_min_x) / 2
    }

    pub fn snapshot(&self) -> StageSnapshot {
        StageSnapshot {
            attention: self.attention,
            log_len: self.log.len(),
            log_tail: self.log.back().map(|line| line.text.clone()),
            log_source: self
                .log
                .back()
                .map(|line| line.attribution.source_spirit_id.as_str().to_owned()),
            actors: self.actors.iter().map(ActorSnapshot::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests;
