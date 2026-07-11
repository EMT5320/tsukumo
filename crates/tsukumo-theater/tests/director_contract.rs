//! Public contract tests for the pure C1 event-to-stage director.

use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, OutcomeStatus, PersistedText, QuestId, SessionId,
    SpiritId, Timestamp, ToolResult, VendorEventRef, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_theater::{direct, ActorPose, AttentionTier, DirectorContext, StageEvent};

fn event(payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-director"),
        occurred_at: Timestamp::from_unix_millis(1_750_000_000_000),
        quest_id: QuestId::new("quest-director"),
        session_id: SessionId::new("session-director"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

#[test]
fn tool_start_maps_to_focus_and_work_pose() {
    // Given: one attributed tool start.
    let input = event(KernelEventPayload::ToolStart {
        vendor_call: VendorEventRef::new("fixture", "call-1"),
        tool: "read".into(),
        args: None,
        projection_id: None,
    });

    // When: the pure director maps the durable event.
    let output = direct(&input, &DirectorContext::default());

    // Then: theater focuses the matching spirit at work.
    assert!(output.iter().any(|stage| matches!(
        stage,
        StageEvent::ActorPose {
            pose: ActorPose::Work,
            spirit_id: Some(id)
        } if id.as_str() == "yuka"
    )));
    assert!(output.iter().any(|stage| matches!(
        stage,
        StageEvent::AttentionTier {
            tier: AttentionTier::Focus
        }
    )));
}

#[test]
fn permission_request_raises_urgent_and_honors_line_book() {
    // Given: a permission request and presentation-only custom copy.
    let input = event(KernelEventPayload::PermissionRequested {
        vendor_request: VendorEventRef::new("fixture", "perm-1"),
        tool: "shell".into(),
        arguments: None,
        cwd: None,
        risk_reasons: Vec::new(),
        reason: PersistedText::from_reviewed("shell: cargo test"),
    });
    let mut context = DirectorContext::default();
    context.line_book.waiting = Some("老师，需要批准".into());

    // When: the event enters theater.
    let output = direct(&input, &context);

    // Then: it blocks visibly without changing the durable payload.
    assert!(output.iter().any(|stage| matches!(
        stage,
        StageEvent::AttentionTier {
            tier: AttentionTier::Urgent
        }
    )));
    assert!(output.iter().any(|stage| matches!(
        stage,
        StageEvent::ActorPose {
            pose: ActorPose::Wait,
            ..
        }
    )));
    assert!(output.iter().any(|stage| matches!(
        stage,
        StageEvent::Bubble { text, .. } if text == "老师，需要批准"
    )));
}

#[test]
fn failed_tool_result_uses_upset_pose() {
    // Given: one normalized failed tool result.
    let input = event(KernelEventPayload::ToolEnd {
        vendor_call: VendorEventRef::new("fixture", "call-1"),
        result: ToolResult::reviewed_text("permission denied"),
        is_error: true,
        projection_id: None,
    });

    // When: theater maps the failure.
    let output = direct(&input, &DirectorContext::default());

    // Then: the visual state shows an upset actor.
    assert!(output.iter().any(|stage| matches!(
        stage,
        StageEvent::ActorPose {
            pose: ActorPose::Upset,
            ..
        }
    )));
}

#[test]
fn successful_outcome_settles_to_ambient() {
    // Given: a successful execution outcome.
    let input = event(KernelEventPayload::Outcome {
        status: OutcomeStatus::Succeeded,
        summary: Some(PersistedText::from_reviewed("all good")),
        projection_id: None,
    });

    // When: theater receives the outcome.
    let output = direct(&input, &DirectorContext::default());

    // Then: attention settles and the spirit celebrates.
    assert!(output.iter().any(|stage| matches!(
        stage,
        StageEvent::AttentionTier {
            tier: AttentionTier::Ambient
        }
    )));
    assert!(output.iter().any(|stage| matches!(
        stage,
        StageEvent::ActorPose {
            pose: ActorPose::Celebrate,
            ..
        }
    )));
}

#[test]
fn user_input_content_never_enters_theater_log() {
    // Given: user evidence containing a diagnostic sentinel.
    let input = event(KernelEventPayload::UserInput {
        content: PersistedText::from_reviewed("sentinel-private-user-text"),
    });

    // When: the event is mapped for presentation.
    let output = direct(&input, &DirectorContext::default());
    let json = serde_json::to_string(&output).expect("serialize stage output");

    // Then: theater records the event kind without copying user content.
    assert!(json.contains("user_input"));
    assert!(!json.contains("sentinel-private-user-text"));
}
