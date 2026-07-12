use super::*;

fn actor_id() -> PresentationActorId {
    PresentationActorId::try_from("companion").expect("valid actor id")
}

fn attribution(source: &str) -> StageAttribution {
    StageAttribution {
        actor_id: actor_id(),
        source_spirit_id: SpiritId::new(source),
    }
}

#[test]
fn apply_pose_bubble_and_lossy_log() {
    // Given: attributed actor, bubble, and three log events with capacity two.
    let mut world = StageWorld::new().with_log_cap(2);
    world.apply(&StageEvent::ActorPose {
        pose: ActorPose::Work,
        attribution: attribution("gina"),
    });
    world.apply(&StageEvent::Bubble {
        text: "using read...".into(),
        attribution: attribution("gina"),
    });
    for text in ["line-1", "line-2", "line-3"] {
        world.apply(&StageEvent::LogLine {
            text: text.into(),
            attribution: attribution("gina"),
        });
    }

    // When: the stage is snapshotted.
    let snapshot = world.snapshot();

    // Then: actor/source facts remain separate and the log stays bounded.
    assert_eq!(snapshot.actors.len(), 1);
    assert_eq!(snapshot.actors[0].id, "companion");
    assert_eq!(snapshot.actors[0].source_spirit_id.as_deref(), Some("gina"));
    assert_eq!(snapshot.actors[0].pose, ActorPose::Work);
    assert_eq!(snapshot.actors[0].motion, Motion::Walk);
    assert_eq!(snapshot.actors[0].bubble.as_deref(), Some("using read..."));
    assert_eq!(snapshot.log_len, 2);
    assert_eq!(snapshot.log_tail.as_deref(), Some("line-3"));
    assert_eq!(snapshot.log_source.as_deref(), Some("gina"));
}

#[test]
fn wait_pose_when_applied_maps_to_idle_motion() {
    // Given: a new stage world.
    let mut world = StageWorld::new();

    // When: the companion receives a Wait pose.
    world.apply(&StageEvent::ActorPose {
        pose: ActorPose::Wait,
        attribution: attribution("gina"),
    });

    // Then: the actor remains stationary in the wait pose.
    let actor = world.primary().expect("primary actor");
    assert_eq!(actor.motion, Motion::Idle);
    assert_eq!(actor.pose, ActorPose::Wait);
}

#[test]
fn tick_when_walk_reaches_bound_settles_to_work() {
    // Given: a working companion one step before the walk bound.
    let mut world = StageWorld::new();
    world.ensure_placeholder(actor_id());
    if let Some(actor) = world.primary_mut() {
        actor.pose = ActorPose::Work;
        actor.motion = Motion::Walk;
        actor.x = WORKSHOP_WALK_MAX_X - 1;
    }

    // When: one deterministic tick is applied.
    world.tick();

    // Then: the actor settles at the work position.
    let actor = world.primary().expect("primary actor");
    assert_eq!(actor.x, WORKSHOP_WALK_MAX_X);
    assert_eq!(actor.motion, Motion::Work);
}

#[test]
fn configured_walk_bounds_when_ticked_settle_at_pack_destination() {
    // Given: an actor starts in a pack-defined corridor and receives a work pose.
    let attribution = StageAttribution {
        actor_id: actor_id(),
        source_spirit_id: SpiritId::new("runtime"),
    };
    let mut world = StageWorld::new().with_walk_bounds(12, 60);
    world.apply(&StageEvent::ActorPose {
        pose: ActorPose::Work,
        attribution,
    });

    // When: deterministic ticks exhaust the corridor.
    while world.tick() {}

    // Then: the same world position rendered by the workshop settles without a jump.
    let actor = world.primary().expect("primary actor");
    assert_eq!(actor.x, 60);
    assert_eq!(actor.motion, Motion::Work);
}
