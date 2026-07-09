//! A1 / F1: structured stream-json → KernelEvent → Director → StageWorld.
//!
//! CI uses the synthetic producer (no Claude install). Live Claude stream-json
//! is the same parser path — see `notes-a1-channel.md`.

use tsukumo_adapters::{
    assemble_prompt, parse_stream_json_str, synthetic_demo_events, synthetic_demo_stream_jsonl,
    BriefingSource, NullBriefing, PromptAssemblyContext, StreamJsonOptions,
};
use tsukumo_kernel::{BackendKind, KernelEvent};
use tsukumo_theater::{
    drive_kernel_events, ActorPose, AttentionTier, DirectorContext, Motion, StageWorld,
};

#[test]
fn a1_stream_json_waiting_permission_raises_urgent() {
    let opts = StreamJsonOptions::default().with_executor("gina");
    let events = parse_stream_json_str(synthetic_demo_stream_jsonl(), &opts).unwrap();

    let ctx = DirectorContext::default();
    let mut world = StageWorld::new().with_log_cap(64);
    world.ensure_placeholder("gina");

    // Drive only through the permission event — attention must go Urgent / Wait.
    let wait_idx = events
        .iter()
        .position(|e| matches!(e, KernelEvent::WaitingPermission { .. }))
        .expect("synthetic stream must include waiting_permission");

    drive_kernel_events(&mut world, &events[..=wait_idx], &ctx);

    assert_eq!(world.attention, AttentionTier::Urgent);
    let actor = world.primary().expect("actor");
    assert_eq!(actor.pose, ActorPose::Wait);
    assert_eq!(actor.motion, Motion::Idle);
    assert!(
        world
            .log
            .iter()
            .any(|l| l.contains("waiting_permission") && l.contains("perm_synth_1")),
        "log={:?}",
        world.log
    );
}

#[test]
fn a1_full_synthetic_quest_drives_stage_to_ambient() {
    let events = synthetic_demo_events("gina");
    let ctx = DirectorContext::default();
    let mut world = StageWorld::new().with_log_cap(64);
    world.ensure_placeholder("gina");

    drive_kernel_events(&mut world, &events, &ctx);

    let snap = world.snapshot();
    assert_eq!(snap.attention, AttentionTier::Ambient);
    assert_eq!(snap.actors[0].pose, ActorPose::Celebrate);
    assert!(
        snap.log_tail
            .as_deref()
            .is_some_and(|t| t.contains("turn_or_quest_end")),
        "tail={:?}",
        snap.log_tail
    );
    // Soft identity + stream_json backend attribution survived the adapter.
    assert!(events.iter().any(|e| match e {
        KernelEvent::ToolStart {
            executor_id,
            backend,
            ..
        } => {
            executor_id.as_ref().map(|id| id.as_str()) == Some("gina")
                && *backend == Some(BackendKind::StreamJson)
        }
        _ => false,
    }));
}

#[test]
fn a1_briefing_assembly_hook_exists() {
    let src = NullBriefing;
    let ctx = PromptAssemblyContext {
        executor_id: Some("gina".into()),
        quest_id: Some("synth-a1".into()),
    };
    // Hook site: host would call briefing_for then assemble_prompt before spawn.
    let brief = src.briefing_for(&ctx);
    let prompt = assemble_prompt("run the workshop check", brief.as_deref());
    assert_eq!(prompt, "run the workshop check");

    // Fixture briefing path (Phase R will fill real content).
    let with_brief = assemble_prompt("run the workshop check", Some("owner prefers tea"));
    assert!(with_brief.contains("<!-- tsukumo-briefing -->"));
}
