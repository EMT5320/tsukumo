//! Shared constructors for bounded, attributed stage events.

use super::DirectorContext;
use crate::stage::{ActorPose, AttentionTier, StageAttribution, StageEvent};
use tsukumo_kernel::{redact_sensitive_text, KernelEvent, RuntimePhase};

pub(super) fn pick<'a>(custom: Option<&'a str>, fallback: &'a str) -> &'a str {
    match custom.filter(|text| !text.is_empty()) {
        Some(text) => text,
        None => fallback,
    }
}

fn attribution(event: &KernelEvent, ctx: &DirectorContext) -> StageAttribution {
    StageAttribution {
        actor_id: ctx.actor_id.clone(),
        source_spirit_id: event.spirit_id.clone(),
    }
}

pub(super) fn actor_pose(
    event: &KernelEvent,
    ctx: &DirectorContext,
    pose: ActorPose,
) -> StageEvent {
    StageEvent::ActorPose {
        pose,
        attribution: attribution(event, ctx),
    }
}

pub(super) fn bubble(
    event: &KernelEvent,
    ctx: &DirectorContext,
    text: impl Into<String>,
) -> StageEvent {
    StageEvent::Bubble {
        text: safe_stage_text(text.into()),
        attribution: attribution(event, ctx),
    }
}

pub(super) fn log_line(
    event: &KernelEvent,
    ctx: &DirectorContext,
    text: impl Into<String>,
) -> StageEvent {
    StageEvent::LogLine {
        text: safe_stage_text(text.into()),
        attribution: attribution(event, ctx),
    }
}

fn safe_stage_text(text: String) -> String {
    let redacted = redact_sensitive_text(&text);
    if redacted.chars().count() <= 512 {
        redacted
    } else {
        redacted.chars().take(509).collect::<String>() + "..."
    }
}

pub(super) fn runtime_lifecycle_events(
    event: &KernelEvent,
    ctx: &DirectorContext,
    phase: RuntimePhase,
) -> Vec<StageEvent> {
    let (tier, pose) = match phase {
        RuntimePhase::Starting | RuntimePhase::Started => (AttentionTier::Focus, ActorPose::Work),
        RuntimePhase::Stopping => (AttentionTier::Focus, ActorPose::Wait),
        RuntimePhase::Completed => (AttentionTier::Ambient, ActorPose::Celebrate),
        RuntimePhase::Failed => (AttentionTier::Urgent, ActorPose::Upset),
        RuntimePhase::Cancelled => (AttentionTier::Ambient, ActorPose::Idle),
    };
    vec![
        StageEvent::AttentionTier { tier },
        actor_pose(event, ctx, pose),
        log_line(event, ctx, format!("runtime_lifecycle {phase:?}")),
    ]
}
