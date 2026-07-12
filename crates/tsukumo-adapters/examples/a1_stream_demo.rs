//! Credential-free adapter -> envelope -> theater print demo.

use std::env;
use tsukumo_adapters::{
    assemble_prompt, synthetic_demo_payloads, BriefingSource, NullBriefing, PromptAssemblyContext,
};
use tsukumo_kernel::{
    validate_kernel_event, CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload,
    ProjectionId, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SessionId, SpiritId,
    Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_theater::{
    drive_kernel_events, render_frame_string, DirectorContext, StageWorld, DEFAULT_FRAME_HEIGHT,
    DEFAULT_FRAME_WIDTH,
};

fn fixture_event(index: usize, mut payload: KernelEventPayload) -> KernelEvent {
    let projection = ProjectionId::new("demo-projection");
    match &mut payload {
        KernelEventPayload::ToolStart { projection_id, .. }
        | KernelEventPayload::ToolEnd { projection_id, .. }
        | KernelEventPayload::Outcome { projection_id, .. } => {
            *projection_id = Some(projection);
        }
        _ => {}
    }
    let correlation_id = match &payload {
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
        } => Some(CorrelationId::new("demo-projection-chain")),
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
    };
    let timestamp_offset = i64::try_from(index).expect("fixture index fits i64");

    let event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(format!("demo-event-{:02}", index + 1)),
        occurred_at: Timestamp::from_unix_millis(1_750_000_100_000 + timestamp_offset),
        quest_id: QuestId::new("demo-quest"),
        session_id: SessionId::new("demo-session"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(ExecutionId::new("demo-execution")),
        runtime: Some(RuntimeBinding::new(
            RuntimeKind::ClaudeCli,
            RuntimeMode::Synthetic,
        )),
        causation_id: None,
        correlation_id,
        payload,
    };
    validate_kernel_event(&event).expect("demo host assigns durable attribution");
    event
}

fn main() {
    let stop_at_wait = env::args().any(|argument| argument == "--stop-at-wait");

    // The adapter owns vendor normalization; this demo explicitly simulates
    // the future host boundary that assigns durable envelope metadata.
    let events = synthetic_demo_payloads()
        .expect("synthetic fixture must decode")
        .into_iter()
        .enumerate()
        .map(|(index, payload)| fixture_event(index, payload))
        .collect::<Vec<_>>();
    let end = if stop_at_wait {
        events
            .iter()
            .position(|event| {
                matches!(
                    event.payload,
                    KernelEventPayload::PermissionRequested { .. }
                )
            })
            .map_or(events.len(), |index| index + 1)
    } else {
        events.len()
    };

    let briefing = NullBriefing.briefing_for(&PromptAssemblyContext {
        spirit_id: Some(SpiritId::new("yuka")),
        quest_id: Some(QuestId::new("demo-quest")),
    });
    let _prompt = assemble_prompt("synthetic workshop quest", briefing.as_deref());

    let mut world = StageWorld::new().with_log_cap(24);
    world.ensure_placeholder(DirectorContext::default().actor_id);
    drive_kernel_events(&mut world, &events[..end], &DirectorContext::default());

    let frame = render_frame_string(&world, DEFAULT_FRAME_WIDTH, DEFAULT_FRAME_HEIGHT);
    println!("{frame}");
    println!();
    println!(
        "C1 demo: {} durable events -> stage (attention={:?}, pose={:?}, log={})",
        end,
        world.attention,
        world.primary().map(|actor| actor.pose),
        world.log.len()
    );
}
