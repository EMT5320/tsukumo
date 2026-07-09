//! Fixture-driven Director replay (P0.4).

use std::path::PathBuf;
use tsukumo_kernel::read_jsonl_events;
use tsukumo_theater::{direct, ActorPose, AttentionTier, DirectorContext, StageEvent};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/minimal_quest.jsonl")
}

#[test]
fn fixture_replays_without_panic() {
    let events = read_jsonl_events(fixture_path()).expect("load fixture");
    assert_eq!(events.len(), 9);

    let ctx = DirectorContext::default();
    let mut stage: Vec<StageEvent> = Vec::new();
    for ev in &events {
        stage.extend(direct(ev, &ctx));
    }

    assert!(!stage.is_empty());
    assert!(
        stage.iter().any(|e| matches!(
            e,
            StageEvent::ActorPose {
                pose: ActorPose::Wait,
                ..
            }
        )),
        "waiting_permission should produce Wait pose"
    );
    assert!(
        stage.iter().any(|e| matches!(
            e,
            StageEvent::AttentionTier {
                tier: AttentionTier::Urgent
            }
        )),
        "permission / error should raise Urgent"
    );
    assert!(
        stage.iter().any(|e| matches!(
            e,
            StageEvent::ActorPose {
                pose: ActorPose::Celebrate,
                ..
            }
        )),
        "quest end should celebrate"
    );
    assert!(
        stage.iter().any(|e| matches!(e, StageEvent::LogLine { .. })),
        "every mapped event should leave a log line"
    );

    // Theater types must stay vendor-clean even after fixture replay.
    for ev in &stage {
        let json = serde_json::to_string(ev).unwrap();
        assert!(!json.contains("claude"), "{json}");
        assert!(!json.contains("\"acp\""), "{json}");
    }
}

#[test]
fn fixture_preserves_executor_id_on_poses() {
    let events = read_jsonl_events(fixture_path()).unwrap();
    let ctx = DirectorContext::default();
    let first = direct(&events[0], &ctx);
    let pose = first.iter().find_map(|e| match e {
        StageEvent::ActorPose { executor_id, .. } => executor_id.as_ref(),
        _ => None,
    });
    assert_eq!(pose.map(|id| id.as_str()), Some("gina"));
}
