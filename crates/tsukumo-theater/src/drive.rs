//! Drive a [`StageWorld`] from KernelEvent streams via the pure Director.
//!
//! Theater-only wiring — adapters stay out of this path (S1b fixture proof).

use crate::director::{direct, DirectorContext};
use crate::world::StageWorld;
use tsukumo_kernel::KernelEvent;

/// Apply one kernel event through Director into the stage world.
pub fn drive_kernel_event(world: &mut StageWorld, event: &KernelEvent, ctx: &DirectorContext) {
    let stage = direct(event, ctx);
    world.apply_all(&stage);
}

/// Replay a kernel event list (fixture / recorded session).
pub fn drive_kernel_events(world: &mut StageWorld, events: &[KernelEvent], ctx: &DirectorContext) {
    for ev in events {
        drive_kernel_event(world, ev, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::{ActorPose, AttentionTier};
    use crate::world::Motion;
    use std::path::PathBuf;
    use tsukumo_kernel::read_jsonl_events;

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/minimal_quest.jsonl")
    }

    #[test]
    fn fixture_drives_stage_world_acting() {
        let events = read_jsonl_events(fixture_path()).expect("load fixture");
        let ctx = DirectorContext::default();
        let mut world = StageWorld::new().with_log_cap(64);
        world.ensure_placeholder("gina");

        drive_kernel_events(&mut world, &events, &ctx);

        let snap = world.snapshot();
        assert_eq!(snap.actors.len(), 1);
        assert_eq!(snap.actors[0].id, "gina");
        // Quest end → Celebrate pose (motion Idle).
        assert_eq!(snap.actors[0].pose, ActorPose::Celebrate);
        assert_eq!(snap.actors[0].motion, Motion::Idle);
        assert_eq!(snap.attention, AttentionTier::Ambient);
        assert!(
            snap.log_len >= 5,
            "expected several log lines, got {}",
            snap.log_len
        );
        assert!(
            snap.log_tail
                .as_deref()
                .is_some_and(|t| t.contains("outcome")),
            "tail={:?}",
            snap.log_tail
        );
        assert!(
            snap.actors[0]
                .bubble
                .as_deref()
                .is_some_and(|b| b.contains("fixture quest") || b.contains("complete")),
            "bubble={:?}",
            snap.actors[0].bubble
        );
    }

    #[test]
    fn mid_fixture_shows_wait_then_work_path() {
        let events = read_jsonl_events(fixture_path()).unwrap();
        let ctx = DirectorContext::default();
        let mut world = StageWorld::new();

        // Through waiting_permission (index 3).
        drive_kernel_events(&mut world, &events[..=3], &ctx);
        let a = world.primary().expect("actor");
        assert_eq!(a.pose, ActorPose::Wait);
        assert_eq!(a.motion, Motion::Idle);
        assert_eq!(world.attention, AttentionTier::Urgent);

        // Next successful tool_end → Work (briefly Walk).
        drive_kernel_event(&mut world, &events[4], &ctx);
        let a = world.primary().unwrap();
        assert_eq!(a.pose, ActorPose::Work);
        assert_eq!(a.motion, Motion::Walk);
    }
}
