//! Fixture adapter -> envelope -> Director -> StageWorld integration.

use tsukumo_adapters::{
    assemble_prompt, synthetic_demo_payloads, BriefingSource, NullBriefing, PromptAssemblyContext,
};
use tsukumo_kernel::{
    validate_kernel_event, CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload,
    ProjectionId, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SessionId, SpiritId,
    Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_theater::{
    drive_kernel_events, ActorPose, AttentionTier, DirectorContext, Motion, StageWorld,
};

fn correlation_for(payload: &KernelEventPayload) -> Option<CorrelationId> {
    match payload {
        KernelEventPayload::ToolStart { vendor_call, .. }
        | KernelEventPayload::ToolEnd { vendor_call, .. } => {
            Some(CorrelationId::new(vendor_call.id.clone()))
        }
        KernelEventPayload::PermissionRequested { vendor_request, .. }
        | KernelEventPayload::PermissionDecided { vendor_request, .. } => {
            Some(CorrelationId::new(vendor_request.id.clone()))
        }
        KernelEventPayload::ProjectionCreated { .. }
        | KernelEventPayload::Outcome {
            projection_id: Some(_),
            ..
        } => Some(CorrelationId::new("synthetic-projection-chain")),
        KernelEventPayload::UserInput { .. }
        | KernelEventPayload::LegacyImported { .. }
        | KernelEventPayload::RuntimeLifecycle { .. }
        | KernelEventPayload::RuntimeSwitched { .. }
        | KernelEventPayload::StateLifecycle { .. }
        | KernelEventPayload::CheckpointCreated { .. }
        | KernelEventPayload::Outcome {
            projection_id: None,
            ..
        }
        | KernelEventPayload::Error { .. } => None,
    }
}

fn bind_fixture_projection(payload: &mut KernelEventPayload) {
    let projection = ProjectionId::new("synthetic-projection");
    match payload {
        KernelEventPayload::ToolStart { projection_id, .. }
        | KernelEventPayload::ToolEnd { projection_id, .. }
        | KernelEventPayload::Outcome { projection_id, .. } => {
            *projection_id = Some(projection);
        }
        KernelEventPayload::UserInput { .. }
        | KernelEventPayload::LegacyImported { .. }
        | KernelEventPayload::RuntimeLifecycle { .. }
        | KernelEventPayload::RuntimeSwitched { .. }
        | KernelEventPayload::PermissionRequested { .. }
        | KernelEventPayload::PermissionDecided { .. }
        | KernelEventPayload::StateLifecycle { .. }
        | KernelEventPayload::CheckpointCreated { .. }
        | KernelEventPayload::ProjectionCreated { .. }
        | KernelEventPayload::Error { .. } => {}
    }
}
fn fixture_events(payloads: Vec<KernelEventPayload>) -> Vec<KernelEvent> {
    payloads
        .into_iter()
        .enumerate()
        .map(|(index, mut payload)| {
            bind_fixture_projection(&mut payload);
            let timestamp_offset = i64::try_from(index).expect("fixture index fits i64");
            let event = KernelEvent {
                schema_version: KERNEL_EVENT_SCHEMA_VERSION,
                event_id: EventId::new(format!("synthetic-event-{:02}", index + 1)),
                occurred_at: Timestamp::from_unix_millis(1_750_000_100_000 + timestamp_offset),
                quest_id: QuestId::new("synthetic-quest"),
                session_id: SessionId::new("synthetic-session"),
                spirit_id: SpiritId::new("yuka"),
                execution_id: Some(ExecutionId::new("synthetic-execution")),
                runtime: Some(RuntimeBinding::new(
                    RuntimeKind::ClaudeCli,
                    RuntimeMode::Fixture,
                )),
                causation_id: None,
                correlation_id: correlation_for(&payload),
                payload,
            };
            validate_kernel_event(&event).expect("fixture host assigns durable attribution");
            event
        })
        .collect()
}

#[test]
fn stream_json_permission_raises_urgent() {
    // Given: the committed Claude fixture wrapped by a deterministic test host.
    let events = fixture_events(synthetic_demo_payloads().expect("decode synthetic fixture"));
    let permission_index = events
        .iter()
        .position(|event| {
            matches!(
                event.payload,
                KernelEventPayload::PermissionRequested { .. }
            )
        })
        .expect("fixture includes permission request");
    let mut world = StageWorld::new().with_log_cap(64);
    world.ensure_placeholder(DirectorContext::default().actor_id);

    // When: theater replays through the permission request.
    drive_kernel_events(
        &mut world,
        &events[..=permission_index],
        &DirectorContext::default(),
    );

    // Then: the request is visibly blocking and traceable.
    assert_eq!(world.attention, AttentionTier::Urgent);
    let actor = world.primary().expect("primary actor");
    assert_eq!(actor.pose, ActorPose::Upset);
    assert_eq!(actor.motion, Motion::Idle);
    assert!(world.log.iter().any(
        |line| line.text.contains("permission_requested") && line.text.contains("perm_synth_1")
    ));
}

#[test]
fn full_synthetic_quest_drives_stage_to_ambient() {
    // Given: normalized payloads with host-assigned envelope identity.
    let events = fixture_events(synthetic_demo_payloads().expect("decode synthetic fixture"));
    let mut world = StageWorld::new().with_log_cap(64);
    world.ensure_placeholder(DirectorContext::default().actor_id);

    // When: theater replays the complete event sequence.
    drive_kernel_events(&mut world, &events, &DirectorContext::default());

    // Then: the quest settles and envelope attribution remains intact.
    let snapshot = world.snapshot();
    assert_eq!(snapshot.attention, AttentionTier::Ambient);
    assert_eq!(snapshot.actors[0].pose, ActorPose::Celebrate);
    assert!(snapshot
        .log_tail
        .as_deref()
        .is_some_and(|tail| tail.contains("outcome")));
    assert!(events.iter().any(|event| {
        event.spirit_id.as_str() == "yuka"
            && event.runtime
                == Some(RuntimeBinding::new(
                    RuntimeKind::ClaudeCli,
                    RuntimeMode::Fixture,
                ))
    }));
}

#[test]
fn briefing_assembly_hook_remains_stable() {
    // Given: the placeholder briefing source and one fixture briefing.
    let source = NullBriefing;
    let context = PromptAssemblyContext {
        spirit_id: Some(SpiritId::new("yuka")),
        quest_id: Some(QuestId::new("synthetic-quest")),
    };

    // When: callers assemble prompts through the existing seam.
    let empty = source.briefing_for(&context);
    let plain = assemble_prompt("run the workshop check", empty.as_deref());
    let enriched = assemble_prompt("run the workshop check", Some("owner prefers tea"));

    // Then: the empty path stays unchanged and marked content remains explicit.
    assert_eq!(plain, "run the workshop check");
    assert!(enriched.contains("<!-- tsukumo-briefing -->"));
}

#[test]
fn fixture_host_envelopes_persist_and_reopen() {
    // Given: adapter payloads enriched only by the deterministic host seam.
    let directory = tempfile::tempdir().expect("create adapter Chronicle directory");
    let events = fixture_events(synthetic_demo_payloads().expect("decode synthetic fixture"));
    let mut store =
        tsukumo_soul::SoulStore::open(directory.path()).expect("open adapter Chronicle");

    // When: every enriched event enters Chronicle and the database is reopened.
    for event in &events {
        store
            .append_event(event)
            .expect("persist enriched adapter event");
    }
    drop(store);
    let reopened =
        tsukumo_soul::SoulStore::open(directory.path()).expect("reopen adapter Chronicle");
    let replayed = reopened
        .replay_events(tsukumo_soul::ChronicleQuery::default())
        .expect("replay adapter Chronicle");
    let replayed_events = replayed
        .iter()
        .map(|stored| stored.event.clone())
        .collect::<Vec<_>>();
    let mut world = StageWorld::new().with_log_cap(64);
    world.ensure_placeholder(DirectorContext::default().actor_id);
    drive_kernel_events(&mut world, &replayed_events, &DirectorContext::default());

    // Then: the durable fixture path remains consumable by Theater after reopen.
    assert_eq!(replayed.len(), events.len());
    assert_eq!(replayed[0].event, events[0]);
    assert_eq!(world.attention, AttentionTier::Ambient);
    assert_eq!(
        world.primary().expect("replayed primary actor").pose,
        ActorPose::Celebrate
    );
}
