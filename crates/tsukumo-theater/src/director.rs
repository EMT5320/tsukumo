//! Pure mapping from durable kernel events to lossy theater events.

mod context;
mod events;
mod outcome;

pub use context::{DirectorContext, LineBook};

use events::{actor_pose, bubble, log_line, pick, runtime_lifecycle_events};
use outcome::outcome_events;
use tsukumo_kernel::{KernelEvent, KernelEventPayload, PermissionDecision};

/// Maps one durable kernel event into zero or more presentation events.
///
/// This function has no I/O, clocks, process calls, storage, or mutable globals.
pub fn direct(event: &KernelEvent, ctx: &DirectorContext) -> Vec<crate::stage::StageEvent> {
    let book = &ctx.line_book;
    match &event.payload {
        KernelEventPayload::UserInput { .. } => {
            // User text stays out of the theater log; Chronicle owns the source.
            vec![log_line(event, ctx, "user_input")]
        }
        KernelEventPayload::LegacyImported {
            source_id, kind, ..
        } => vec![log_line(
            event,
            ctx,
            format!("legacy_imported {kind} {source_id}"),
        )],
        KernelEventPayload::RuntimeLifecycle { phase } => {
            runtime_lifecycle_events(event, ctx, *phase)
        }
        KernelEventPayload::RuntimeSwitched { current, .. } => vec![
            crate::stage::StageEvent::AttentionTier {
                tier: crate::stage::AttentionTier::Focus,
            },
            actor_pose(event, ctx, crate::stage::ActorPose::Walk),
            bubble(event, ctx, "switching runtime..."),
            log_line(
                event,
                ctx,
                format!("runtime_switched {:?}/{:?}", current.kind, current.mode),
            ),
        ],
        KernelEventPayload::ToolStart {
            vendor_call, tool, ..
        } => {
            let fallback = format!("using {tool}...");
            vec![
                crate::stage::StageEvent::AttentionTier {
                    tier: crate::stage::AttentionTier::Focus,
                },
                actor_pose(event, ctx, crate::stage::ActorPose::Work),
                bubble(
                    event,
                    ctx,
                    pick(book.tool_start.as_deref(), &fallback).to_owned(),
                ),
                log_line(
                    event,
                    ctx,
                    format!("tool_start {tool} ({})", vendor_call.id),
                ),
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
                    crate::stage::ActorPose::Upset,
                    crate::stage::AttentionTier::Urgent,
                    book.tool_end_err.as_deref(),
                )
            } else {
                (
                    crate::stage::ActorPose::Work,
                    crate::stage::AttentionTier::Focus,
                    book.tool_end_ok.as_deref(),
                )
            };
            vec![
                crate::stage::StageEvent::AttentionTier { tier },
                actor_pose(event, ctx, pose),
                bubble(event, ctx, pick(custom, result.summary.as_str()).to_owned()),
                log_line(
                    event,
                    ctx,
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
            crate::stage::StageEvent::AttentionTier {
                tier: crate::stage::AttentionTier::Urgent,
            },
            actor_pose(event, ctx, crate::stage::ActorPose::Upset),
            bubble(
                event,
                ctx,
                pick(book.waiting.as_deref(), "need your approval...").to_owned(),
            ),
            log_line(
                event,
                ctx,
                format!("permission_requested {}: {reason}", vendor_request.id),
            ),
        ],
        KernelEventPayload::PermissionDecided {
            vendor_request,
            decision,
        } => {
            let (tier, pose, copy) = match decision {
                PermissionDecision::AllowOnce | PermissionDecision::AllowSession => (
                    crate::stage::AttentionTier::Focus,
                    crate::stage::ActorPose::Work,
                    "permission granted",
                ),
                PermissionDecision::Deny => (
                    crate::stage::AttentionTier::Ambient,
                    crate::stage::ActorPose::Upset,
                    "permission denied",
                ),
            };
            vec![
                crate::stage::StageEvent::AttentionTier { tier },
                actor_pose(event, ctx, pose),
                bubble(event, ctx, copy),
                log_line(
                    event,
                    ctx,
                    format!("permission_decided {} {decision:?}", vendor_request.id),
                ),
            ]
        }
        KernelEventPayload::StateLifecycle {
            state_id, action, ..
        } => vec![log_line(
            event,
            ctx,
            format!("state_lifecycle {state_id} {action:?}"),
        )],
        KernelEventPayload::CheckpointCreated {
            checkpoint_id,
            version,
        } => vec![log_line(
            event,
            ctx,
            format!("checkpoint_created {checkpoint_id} v{version}"),
        )],
        KernelEventPayload::ProjectionCreated {
            projection_id,
            checkpoint_id,
        } => vec![log_line(
            event,
            ctx,
            format!("projection_created {projection_id} from {checkpoint_id}"),
        )],
        KernelEventPayload::Outcome {
            status, summary, ..
        } => outcome_events(
            event,
            ctx,
            *status,
            summary.as_ref().map(|text| text.as_str()),
            book.outcome.as_deref(),
        ),
        KernelEventPayload::Error {
            message,
            recoverable,
        } => vec![
            crate::stage::StageEvent::AttentionTier {
                tier: crate::stage::AttentionTier::Urgent,
            },
            actor_pose(event, ctx, crate::stage::ActorPose::Upset),
            bubble(
                event,
                ctx,
                pick(book.error.as_deref(), message.as_str()).to_owned(),
            ),
            log_line(
                event,
                ctx,
                format!(
                    "error{}: {message}",
                    if *recoverable { " (recoverable)" } else { "" }
                ),
            ),
        ],
    }
}
