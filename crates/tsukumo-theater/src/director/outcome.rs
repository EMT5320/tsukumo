//! Outcome-specific presentation mapping.

use super::{actor_pose, bubble, log_line, pick, DirectorContext};
use crate::stage::{ActorPose, AttentionTier, StageEvent};
use tsukumo_kernel::{KernelEvent, OutcomeStatus};

pub(super) fn outcome_events(
    event: &KernelEvent,
    ctx: &DirectorContext,
    status: OutcomeStatus,
    summary: Option<&str>,
    custom: Option<&str>,
) -> Vec<StageEvent> {
    let custom = if status == OutcomeStatus::Succeeded {
        custom
    } else {
        None
    };
    let default = summary
        .filter(|text| !text.is_empty())
        .unwrap_or(match status {
            OutcomeStatus::Succeeded => "quest complete",
            OutcomeStatus::Failed => "quest failed",
            OutcomeStatus::Cancelled => "quest cancelled",
            OutcomeStatus::PermissionDenied => "permission denied",
            OutcomeStatus::SafetyUnsupported => "runtime safety unsupported",
            OutcomeStatus::Degraded => "quest completed with degraded safety",
            OutcomeStatus::TimedOut => "runtime timed out",
            OutcomeStatus::MalformedOutput => "runtime output malformed",
            OutcomeStatus::NonZeroExit => "runtime exited unsuccessfully",
            OutcomeStatus::LaunchFailed => "runtime launch failed",
        });
    let (tier, pose) = match status {
        OutcomeStatus::Succeeded => (AttentionTier::Ambient, ActorPose::Celebrate),
        OutcomeStatus::Failed
        | OutcomeStatus::TimedOut
        | OutcomeStatus::MalformedOutput
        | OutcomeStatus::NonZeroExit
        | OutcomeStatus::LaunchFailed
        | OutcomeStatus::SafetyUnsupported => (AttentionTier::Urgent, ActorPose::Upset),
        OutcomeStatus::Cancelled | OutcomeStatus::PermissionDenied => {
            (AttentionTier::Ambient, ActorPose::Idle)
        }
        OutcomeStatus::Degraded => (AttentionTier::Urgent, ActorPose::Upset),
    };
    vec![
        StageEvent::AttentionTier { tier },
        actor_pose(event, ctx, pose),
        bubble(event, ctx, pick(custom, default)),
        log_line(event, ctx, format!("outcome {status:?}: {default}")),
    ]
}
