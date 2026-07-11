use super::*;

#[test]
fn apply_pose_bubble_and_lossy_log() {
    let mut world = StageWorld::new().with_log_cap(2);
    world.apply(&StageEvent::ActorPose {
        pose: ActorPose::Work,
        spirit_id: Some(SpiritId::new("gina")),
    });
    world.apply(&StageEvent::Bubble {
        text: "using read…".into(),
        spirit_id: Some(SpiritId::new("gina")),
    });
    world.apply(&StageEvent::LogLine {
        text: "line-1".into(),
        spirit_id: None,
    });
    world.apply(&StageEvent::LogLine {
        text: "line-2".into(),
        spirit_id: None,
    });
    world.apply(&StageEvent::LogLine {
        text: "line-3".into(),
        spirit_id: None,
    });

    let snap = world.snapshot();
    assert_eq!(snap.actors.len(), 1);
    assert_eq!(snap.actors[0].id, "gina");
    assert_eq!(snap.actors[0].pose, ActorPose::Work);
    // Entering Work from Idle briefly becomes Walk.
    assert_eq!(snap.actors[0].motion, Motion::Walk);
    assert_eq!(snap.actors[0].bubble.as_deref(), Some("using read…"));
    assert_eq!(snap.log_len, 2);
    assert_eq!(snap.log_tail.as_deref(), Some("line-3"));
}

#[test]
fn wait_pose_maps_to_idle_motion() {
    let mut world = StageWorld::new();
    world.apply(&StageEvent::ActorPose {
        pose: ActorPose::Wait,
        spirit_id: Some(SpiritId::new("gina")),
    });
    assert_eq!(world.primary().unwrap().motion, Motion::Idle);
    assert_eq!(world.primary().unwrap().pose, ActorPose::Wait);
}

#[test]
fn tick_advances_walk_then_settles() {
    let mut world = StageWorld::new();
    world.ensure_placeholder("gina");
    {
        let a = world.primary_mut().unwrap();
        a.pose = ActorPose::Work;
        a.motion = Motion::Walk;
        a.x = WORKSHOP_WALK_MAX_X - 1;
    }
    world.tick();
    let a = world.primary().unwrap();
    assert_eq!(a.x, WORKSHOP_WALK_MAX_X);
    assert_eq!(a.motion, Motion::Work);
}
