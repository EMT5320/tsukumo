//! C1 public contract tests for durable event identities and envelopes.

use tsukumo_kernel::{
    redact_sensitive_text, validate_kernel_event, ArtifactId, CheckpointId, CorrelationId,
    EventContractError, EventId, ExecutionId, KernelEvent, KernelEventPayload, OutcomeStatus,
    OwnerId, PermissionDecision, PersistedJson, PersistedText, ProjectionId, QuestId,
    RuntimeBinding, RuntimeKind, RuntimeMode, RuntimePhase, SensitiveText, SessionId, SpiritId,
    StateId, StateLifecycleAction, Timestamp, VendorEventRef, WorkspaceId,
    KERNEL_EVENT_SCHEMA_VERSION,
};

#[test]
fn semantic_ids_roundtrip_as_transparent_strings() {
    // Given: every durable C1 identity uses a distinct public type.
    let values = [
        serde_json::to_value(EventId::new("event-1")).expect("serialize EventId"),
        serde_json::to_value(QuestId::new("quest-1")).expect("serialize QuestId"),
        serde_json::to_value(SessionId::new("session-1")).expect("serialize SessionId"),
        serde_json::to_value(OwnerId::new("owner-1")).expect("serialize OwnerId"),
        serde_json::to_value(WorkspaceId::new("workspace-1")).expect("serialize WorkspaceId"),
        serde_json::to_value(SpiritId::new("spirit-1")).expect("serialize SpiritId"),
        serde_json::to_value(ExecutionId::new("execution-1")).expect("serialize ExecutionId"),
        serde_json::to_value(StateId::new("state-1")).expect("serialize StateId"),
        serde_json::to_value(CheckpointId::new("checkpoint-1")).expect("serialize CheckpointId"),
        serde_json::to_value(ProjectionId::new("projection-1")).expect("serialize ProjectionId"),
        serde_json::to_value(CorrelationId::new("correlation-1")).expect("serialize CorrelationId"),
        serde_json::to_value(ArtifactId::new("artifact-1")).expect("serialize ArtifactId"),
    ];

    // When: the values cross the JSON boundary.
    let strings = values
        .iter()
        .map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    // Then: serde preserves the opaque string values without an object wrapper.
    assert_eq!(
        strings,
        vec![
            Some("event-1"),
            Some("quest-1"),
            Some("session-1"),
            Some("owner-1"),
            Some("workspace-1"),
            Some("spirit-1"),
            Some("execution-1"),
            Some("state-1"),
            Some("checkpoint-1"),
            Some("projection-1"),
            Some("correlation-1"),
            Some("artifact-1"),
        ]
    );
}

#[test]
fn sensitive_text_redacts_formatting_and_requires_explicit_exposure() {
    // Given: secret-bearing runtime text.
    let text = SensitiveText::new("sentinel-secret");

    // When: ordinary diagnostics format the wrapper.
    let debug = format!("{text:?}");
    let display = text.to_string();

    // Then: diagnostics redact the value while a narrow boundary can expose it.
    assert_eq!(debug, "SensitiveText([REDACTED])");
    assert_eq!(display, "[REDACTED]");
    assert_eq!(text.expose(), "sentinel-secret");
    let persisted = PersistedText::from_reviewed("persisted-sentinel");
    let json = PersistedJson::from_reviewed(serde_json::json!({"safe": "value"}));
    assert_eq!(format!("{persisted:?}"), "PersistedText([REDACTED])");
    assert_eq!(format!("{json:?}"), "PersistedJson([REDACTED])");
}

#[test]
fn untrusted_text_when_terminal_format_characters_arrive_removes_them() {
    // Given: visible text mixed with bidi, zero-width, control, and newline characters.
    let untrusted = "left\u{202e}right\u{200b}\u{0007}\nnext";

    // When: the shared persistence boundary sanitizes the text.
    let sanitized = redact_sensitive_text(untrusted);

    // Then: visible order is stable and the allowed newline becomes one space.
    assert_eq!(sanitized, "leftright next");
}

#[test]
fn kernel_event_envelope_roundtrips_with_correlation_and_runtime_identity() {
    // Given: a fully attributed tool event produced after a runtime projection.
    let event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-tool-1"),
        occurred_at: Timestamp::from_unix_millis(1_750_000_000_123),
        quest_id: QuestId::new("quest-1"),
        session_id: SessionId::new("session-1"),
        spirit_id: SpiritId::new("spirit-yuka"),
        execution_id: Some(ExecutionId::new("execution-1")),
        runtime: Some(RuntimeBinding::new(
            RuntimeKind::ClaudeCli,
            RuntimeMode::Fixture,
        )),
        causation_id: Some(EventId::new("event-projection-1")),
        correlation_id: Some(CorrelationId::new("tool-call-1")),
        payload: KernelEventPayload::ToolStart {
            vendor_call: VendorEventRef::new("claude_cli", "toolu_1"),
            tool: "Bash".into(),
            args: Some(PersistedJson::from_reviewed(
                serde_json::json!({"command": "cargo test"}),
            )),
            projection_id: Some(ProjectionId::new("projection-1")),
        },
    };

    // When: the durable envelope is serialized and reopened.
    let json = serde_json::to_string(&event).expect("serialize KernelEvent");
    let reopened: KernelEvent = serde_json::from_str(&json).expect("deserialize KernelEvent");

    // Then: the complete evidence context survives and vendor data stays namespaced.
    assert_eq!(reopened, event);
    assert!(json.contains("\"schema_version\":1"));
    assert!(json.contains("\"type\":\"tool_start\""));
    assert!(json.contains("\"namespace\":\"claude_cli\""));
    assert!(json.contains("\"projection_id\":\"projection-1\""));
}

#[test]
fn lifecycle_payloads_cover_c1_evidence_boundaries() {
    // Given: the payload variants later C1 layers must persist and replay.
    let payloads = vec![
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("use GNU on Windows"),
        },
        KernelEventPayload::RuntimeLifecycle {
            phase: RuntimePhase::Started,
        },
        KernelEventPayload::RuntimeSwitched {
            previous: None,
            current: RuntimeBinding::new(RuntimeKind::CodexCli, RuntimeMode::Fixture),
        },
        KernelEventPayload::PermissionRequested {
            vendor_request: VendorEventRef::new("claude_cli", "perm-1"),
            tool: "Bash".into(),
            arguments: Some(PersistedJson::from_reviewed(
                serde_json::json!({"command": "cargo test"}),
            )),
            cwd: Some(PersistedText::from_reviewed("D:/WorkSpace/tsukumo")),
            risk_reasons: vec![PersistedText::from_reviewed("shell execution")],
            reason: PersistedText::from_reviewed("run workspace tests"),
        },
        KernelEventPayload::PermissionDecided {
            vendor_request: VendorEventRef::new("claude_cli", "perm-1"),
            decision: PermissionDecision::AllowOnce,
        },
        KernelEventPayload::StateLifecycle {
            state_id: StateId::new("state-1"),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
        KernelEventPayload::CheckpointCreated {
            checkpoint_id: CheckpointId::new("checkpoint-1"),
            version: 1,
        },
        KernelEventPayload::ProjectionCreated {
            projection_id: ProjectionId::new("projection-1"),
            checkpoint_id: CheckpointId::new("checkpoint-1"),
        },
        KernelEventPayload::Outcome {
            status: OutcomeStatus::Succeeded,
            summary: Some(PersistedText::from_reviewed("done")),
            projection_id: Some(ProjectionId::new("projection-1")),
        },
    ];

    // When: every normalized payload crosses its JSON boundary.
    let roundtripped = payloads
        .iter()
        .map(|payload| {
            let json = serde_json::to_string(payload).expect("serialize payload");
            serde_json::from_str::<KernelEventPayload>(&json).expect("deserialize payload")
        })
        .collect::<Vec<_>>();

    // Then: the shared contract represents every required C1 evidence boundary.
    assert_eq!(roundtripped, payloads);
}

#[test]
fn durable_attribution_gate_covers_tool_permission_projection_and_outcome() {
    let payloads = [
        KernelEventPayload::ToolStart {
            vendor_call: VendorEventRef::new("fixture", "call-1"),
            tool: "shell".into(),
            args: None,
            projection_id: Some(ProjectionId::new("projection-1")),
        },
        KernelEventPayload::PermissionRequested {
            vendor_request: VendorEventRef::new("fixture", "perm-1"),
            tool: "shell".into(),
            arguments: None,
            cwd: None,
            risk_reasons: Vec::new(),
            reason: PersistedText::from_reviewed("run tests"),
        },
        KernelEventPayload::ProjectionCreated {
            projection_id: ProjectionId::new("projection-1"),
            checkpoint_id: CheckpointId::new("checkpoint-1"),
        },
        KernelEventPayload::Outcome {
            status: OutcomeStatus::Succeeded,
            summary: None,
            projection_id: Some(ProjectionId::new("projection-1")),
        },
    ];

    for (index, payload) in payloads.into_iter().enumerate() {
        let mut event = KernelEvent {
            schema_version: KERNEL_EVENT_SCHEMA_VERSION,
            event_id: EventId::new(format!("event-attribution-{index}")),
            occurred_at: Timestamp::from_unix_millis(1_750_001_500_000),
            quest_id: QuestId::new("quest-attribution"),
            session_id: SessionId::new("session-attribution"),
            spirit_id: SpiritId::new("yuka"),
            execution_id: None,
            runtime: None,
            causation_id: None,
            correlation_id: None,
            payload,
        };
        assert!(matches!(
            validate_kernel_event(&event),
            Err(EventContractError::MissingAttribution {
                field: "execution_id",
                ..
            })
        ));

        event.execution_id = Some(ExecutionId::new("execution-1"));
        event.runtime = Some(RuntimeBinding::new(
            RuntimeKind::ClaudeCli,
            RuntimeMode::Fixture,
        ));
        event.correlation_id = Some(CorrelationId::new("correlation-1"));
        validate_kernel_event(&event).expect("fully attributed durable event");
    }
}

#[test]
fn durable_attribution_rejects_empty_semantic_ids() {
    // Given: a tool event whose required IDs are present but empty.
    let event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-empty-attribution"),
        occurred_at: Timestamp::from_unix_millis(1_750_001_500_001),
        quest_id: QuestId::new("quest-attribution"),
        session_id: SessionId::new("session-attribution"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(ExecutionId::new("")),
        runtime: Some(RuntimeBinding::new(
            RuntimeKind::ClaudeCli,
            RuntimeMode::Fixture,
        )),
        causation_id: None,
        correlation_id: Some(CorrelationId::new("")),
        payload: KernelEventPayload::ToolStart {
            vendor_call: VendorEventRef::new("fixture", "call-1"),
            tool: "shell".into(),
            args: None,
            projection_id: Some(ProjectionId::new("")),
        },
    };

    // When/Then: value validation rejects IDs that cannot identify a replay chain.
    assert!(matches!(
        validate_kernel_event(&event),
        Err(EventContractError::InvalidField { .. })
    ));
}
