//! Pure mapping from durable kernel events to lossy theater events.

mod outcome;

use crate::stage::{ActorPose, AttentionTier, StageEvent};
use outcome::outcome_events;
use tsukumo_kernel::{
    redact_sensitive_text, KernelEvent, KernelEventPayload, PermissionDecision, RuntimePhase,
};

/// Optional presentation copy keyed by coarse situation.
#[derive(Debug, Clone, Default)]
pub struct LineBook {
    pub tool_start: Option<String>,
    pub tool_end_ok: Option<String>,
    pub tool_end_err: Option<String>,
    pub waiting: Option<String>,
    pub outcome: Option<String>,
    pub error: Option<String>,
}

/// Explicit presentation context consumed by the pure director.
#[derive(Debug, Clone, Default)]
pub struct DirectorContext {
    pub line_book: LineBook,
}

fn pick<'a>(custom: Option<&'a str>, fallback: &'a str) -> &'a str {
    custom.filter(|text| !text.is_empty()).unwrap_or(fallback)
}

fn actor_pose(event: &KernelEvent, pose: ActorPose) -> StageEvent {
    StageEvent::ActorPose {
        pose,
        spirit_id: Some(event.spirit_id.clone()),
    }
}

fn bubble(event: &KernelEvent, text: impl Into<String>) -> StageEvent {
    StageEvent::Bubble {
        text: safe_stage_text(text.into()),
        spirit_id: Some(event.spirit_id.clone()),
    }
}

fn log_line(event: &KernelEvent, text: impl Into<String>) -> StageEvent {
    StageEvent::LogLine {
        text: safe_stage_text(text.into()),
        spirit_id: Some(event.spirit_id.clone()),
    }
}

fn safe_stage_text(text: String) -> String {
    let redacted = redact_sensitive_text(&text);
    if redacted.chars().count() <= 512 {
        redacted
    } else {
        redacted.chars().take(511).collect::<String>() + "?"
    }
}

fn runtime_lifecycle_events(event: &KernelEvent, phase: RuntimePhase) -> Vec<StageEvent> {
    let (tier, pose) = match phase {
        RuntimePhase::Starting | RuntimePhase::Started => (AttentionTier::Focus, ActorPose::Work),
        RuntimePhase::Stopping => (AttentionTier::Focus, ActorPose::Wait),
        RuntimePhase::Completed => (AttentionTier::Ambient, ActorPose::Celebrate),
        RuntimePhase::Failed => (AttentionTier::Urgent, ActorPose::Upset),
        RuntimePhase::Cancelled => (AttentionTier::Ambient, ActorPose::Idle),
    };
    vec![
        StageEvent::AttentionTier { tier },
        actor_pose(event, pose),
        log_line(event, format!("runtime_lifecycle {phase:?}")),
    ]
}

/// Maps one durable kernel event into zero or more presentation events.
///
/// This function has no I/O, clocks, process calls, storage, or mutable globals.
pub fn direct(event: &KernelEvent, ctx: &DirectorContext) -> Vec<StageEvent> {
    let book = &ctx.line_book;
    match &event.payload {
        KernelEventPayload::UserInput { .. } => {
            // User text stays out of the theater log; Chronicle owns the source.
            vec![log_line(event, "user_input")]
        }
        KernelEventPayload::LegacyImported {
            source_id, kind, ..
        } => vec![log_line(
            event,
            format!("legacy_imported {kind} {source_id}"),
        )],
        KernelEventPayload::RuntimeLifecycle { phase } => runtime_lifecycle_events(event, *phase),
        KernelEventPayload::RuntimeSwitched { current, .. } => vec![
            StageEvent::AttentionTier {
                tier: AttentionTier::Focus,
            },
            actor_pose(event, ActorPose::Walk),
            bubble(event, "switching runtime…"),
            log_line(
                event,
                format!("runtime_switched {:?}/{:?}", current.kind, current.mode),
            ),
        ],
        KernelEventPayload::ToolStart {
            vendor_call, tool, ..
        } => {
            let fallback = format!("using {tool}…");
            vec![
                StageEvent::AttentionTier {
                    tier: AttentionTier::Focus,
                },
                actor_pose(event, ActorPose::Work),
                bubble(
                    event,
                    pick(book.tool_start.as_deref(), &fallback).to_owned(),
                ),
                log_line(event, format!("tool_start {tool} ({})", vendor_call.id)),
            ]
        }
        KernelEventPayload::ToolEnd {
            vendor_call,
            result,
            is_error,
            ..
        } => {
            let (pose, tier, custom) = if *is_error {
                (
                    ActorPose::Upset,
                    AttentionTier::Urgent,
                    book.tool_end_err.as_deref(),
                )
            } else {
                (
                    ActorPose::Work,
                    AttentionTier::Focus,
                    book.tool_end_ok.as_deref(),
                )
            };
            vec![
                StageEvent::AttentionTier { tier },
                actor_pose(event, pose),
                bubble(event, pick(custom, result.summary.as_str()).to_owned()),
                log_line(
                    event,
                    format!(
                        "tool_end {}{}: {}",
                        vendor_call.id,
                        if *is_error { " ERR" } else { "" },
                        result.summary
                    ),
                ),
            ]
        }
        KernelEventPayload::PermissionRequested {
            vendor_request,
            reason,
            ..
        } => vec![
            StageEvent::AttentionTier {
                tier: AttentionTier::Urgent,
            },
            actor_pose(event, ActorPose::Wait),
            bubble(
                event,
                pick(book.waiting.as_deref(), "need your OK…").to_owned(),
            ),
            log_line(
                event,
                format!("permission_requested {}: {reason}", vendor_request.id),
            ),
        ],
        KernelEventPayload::PermissionDecided {
            vendor_request,
            decision,
        } => {
            let (tier, pose, copy) = match decision {
                PermissionDecision::AllowOnce | PermissionDecision::AllowSession => {
                    (AttentionTier::Focus, ActorPose::Work, "permission granted")
                }
                PermissionDecision::Deny => (
                    AttentionTier::Ambient,
                    ActorPose::Upset,
                    "permission denied",
                ),
            };
            vec![
                StageEvent::AttentionTier { tier },
                actor_pose(event, pose),
                bubble(event, copy),
                log_line(
                    event,
                    format!("permission_decided {} {decision:?}", vendor_request.id),
                ),
            ]
        }
        KernelEventPayload::StateLifecycle {
            state_id, action, ..
        } => vec![log_line(
            event,
            format!("state_lifecycle {state_id} {action:?}"),
        )],
        KernelEventPayload::CheckpointCreated {
            checkpoint_id,
            version,
        } => vec![log_line(
            event,
            format!("checkpoint_created {checkpoint_id} v{version}"),
        )],
        KernelEventPayload::ProjectionCreated {
            projection_id,
            checkpoint_id,
        } => vec![log_line(
            event,
            format!("projection_created {projection_id} from {checkpoint_id}"),
        )],
        KernelEventPayload::Outcome {
            status, summary, ..
        } => outcome_events(
            event,
            *status,
            summary.as_ref().map(|text| text.as_str()),
            book.outcome.as_deref(),
        ),
        KernelEventPayload::Error {
            message,
            recoverable,
        } => vec![
            StageEvent::AttentionTier {
                tier: AttentionTier::Urgent,
            },
            actor_pose(event, ActorPose::Upset),
            bubble(
                event,
                pick(book.error.as_deref(), message.as_str()).to_owned(),
            ),
            log_line(
                event,
                format!(
                    "error{}: {message}",
                    if *recoverable { " (recoverable)" } else { "" }
                ),
            ),
        ],
    }
}
