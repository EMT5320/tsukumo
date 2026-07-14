//! Failure and compatibility tests for the stateful Codex JSONL decoder.

use tsukumo_adapters::{
    AdapterError, CodexJsonDecoder, DecodeDisposition, DecodeError, RuntimeEventDecoder,
};
use tsukumo_kernel::{KernelEventPayload, OutcomeStatus};

#[test]
fn failed_and_declined_commands_remain_explicit_tool_errors() {
    for (status, exit_code) in [("failed", 1), ("declined", -1)] {
        // Given: a valid Codex turn with one unsuccessful command completion.
        let mut decoder = CodexJsonDecoder::new();
        decode(
            &mut decoder,
            r#"{"type":"thread.started","thread_id":"thread-negative"}"#,
        );
        decode(&mut decoder, r#"{"type":"turn.started"}"#);
        decode(
            &mut decoder,
            r#"{"type":"item.started","item":{"id":"item_0","type":"command_execution","command":"blocked command","aggregated_output":"","exit_code":null,"status":"in_progress"}}"#,
        );
        let completed = format!(
            r#"{{"type":"item.completed","item":{{"id":"item_0","type":"command_execution","command":"blocked command","aggregated_output":"blocked","exit_code":{exit_code},"status":"{status}"}}}}"#
        );

        // When: the command and enclosing turn both complete.
        let tool_end = decoder
            .decode_line(&completed)
            .expect("decode unsuccessful command");
        let terminal = decoder
            .decode_line(r#"{"type":"turn.completed","usage":{}}"#)
            .expect("decode terminal turn");

        // Then: command failure remains visible and cannot become task success.
        assert!(matches!(
            tool_end.payloads.as_slice(),
            [KernelEventPayload::ToolEnd { is_error: true, .. }]
        ));
        assert!(matches!(
            terminal.payloads.as_slice(),
            [KernelEventPayload::Outcome {
                status: OutcomeStatus::Failed,
                summary: Some(summary),
                ..
            }] if summary.as_str() == "Codex turn completed with tool errors"
        ));
        decoder.finish().expect("negative command turn is terminal");
    }
}

#[test]
fn later_success_does_not_erase_an_earlier_tool_error() {
    // Given: one failed command followed by a successful fallback in the same turn.
    let mut decoder = CodexJsonDecoder::new();
    decode(
        &mut decoder,
        r#"{"type":"thread.started","thread_id":"thread-mixed"}"#,
    );
    decode(&mut decoder, r#"{"type":"turn.started"}"#);
    for (item, status, exit_code) in [("failed", "failed", 1), ("fallback", "completed", 0)] {
        decode(
            &mut decoder,
            &format!(
                r#"{{"type":"item.started","item":{{"id":"{item}","type":"command_execution","command":"{item}","aggregated_output":"","exit_code":null,"status":"in_progress"}}}}"#
            ),
        );
        decode(
            &mut decoder,
            &format!(
                r#"{{"type":"item.completed","item":{{"id":"{item}","type":"command_execution","command":"{item}","aggregated_output":"done","exit_code":{exit_code},"status":"{status}"}}}}"#
            ),
        );
    }

    // When: Codex reports that the enclosing model turn completed normally.
    let terminal = decoder
        .decode_line(r#"{"type":"turn.completed","usage":{}}"#)
        .expect("decode mixed terminal turn");

    // Then: the normalized task outcome remains failed for owner review.
    assert!(matches!(
        terminal.payloads.as_slice(),
        [KernelEventPayload::Outcome {
            status: OutcomeStatus::Failed,
            ..
        }]
    ));
    decoder.finish().expect("mixed command turn is terminal");
}

#[test]
fn decoder_rejects_unpaired_truncated_and_duplicate_command_sequences() {
    // Given: a turn whose command has started but has not completed.
    let mut truncated = CodexJsonDecoder::new();
    decode(
        &mut truncated,
        r#"{"type":"thread.started","thread_id":"thread-truncated"}"#,
    );
    decode(&mut truncated, r#"{"type":"turn.started"}"#);
    decode(
        &mut truncated,
        r#"{"type":"item.started","item":{"id":"item_0","type":"command_execution","command":"pending","aggregated_output":"","exit_code":null,"status":"in_progress"}}"#,
    );

    // When: a documented update arrives for the pending command.
    let update = truncated
        .decode_line(r#"{"type":"item.updated","item":{"id":"item_0","type":"command_execution"}}"#)
        .expect("decode paired command update");

    // Then: updates remain observable and EOF still rejects the pending command.
    assert_eq!(update.disposition, DecodeDisposition::KnownIgnored);
    assert!(matches!(
        truncated.finish(),
        Err(AdapterError::TruncatedStream { lines: 4 })
    ));
    let mut unpaired = CodexJsonDecoder::new();
    decode(
        &mut unpaired,
        r#"{"type":"thread.started","thread_id":"thread-unpaired"}"#,
    );
    decode(&mut unpaired, r#"{"type":"turn.started"}"#);
    assert!(matches!(
        unpaired.decode_line(
            r#"{"type":"item.completed","item":{"id":"item_0","type":"command_execution","command":"missing start","aggregated_output":"","exit_code":0,"status":"completed"}}"#
        ),
        Err(AdapterError::Decode {
            line: 3,
            source: DecodeError::InvalidKnown {
                event_type: "item.completed",
                field: "item.id sequence"
            }
        })
    ));

    // Given/When: a known command item omits its required type.
    let mut malformed = CodexJsonDecoder::new();
    decode(
        &mut malformed,
        r#"{"type":"thread.started","thread_id":"thread-malformed"}"#,
    );
    decode(&mut malformed, r#"{"type":"turn.started"}"#);
    assert!(matches!(
        malformed.decode_line(r#"{"type":"item.started","item":{"id":"item_0"}}"#),
        Err(AdapterError::Decode {
            line: 3,
            source: DecodeError::MalformedKnown {
                event_type: "item.started",
                field: "type"
            }
        })
    ));

    // Given/When: a second terminal event arrives after a completed turn.
    let mut duplicate = CodexJsonDecoder::new();
    decode(
        &mut duplicate,
        r#"{"type":"thread.started","thread_id":"thread-terminal"}"#,
    );
    decode(&mut duplicate, r#"{"type":"turn.started"}"#);
    decode(&mut duplicate, r#"{"type":"turn.completed","usage":{}}"#);

    // Then: the shared adapter error preserves the duplicate line number.
    assert!(matches!(
        duplicate.decode_line(r#"{"type":"turn.completed","usage":{}}"#),
        Err(AdapterError::DuplicateTerminal { line: 4 })
    ));
}

#[test]
fn turn_failure_and_runtime_error_emit_bounded_failure_facts() {
    // Given: a valid turn that terminates through the documented failure family.
    let mut failed = CodexJsonDecoder::new();
    decode(
        &mut failed,
        r#"{"type":"thread.started","thread_id":"thread-failed"}"#,
    );
    decode(&mut failed, r#"{"type":"turn.started"}"#);

    // When: the turn emits its terminal failure.
    let terminal = failed
        .decode_line(r#"{"type":"turn.failed"}"#)
        .expect("decode failed turn");

    // Then: the adapter records a generic error and one failed terminal outcome.
    assert!(matches!(
        terminal.payloads.as_slice(),
        [
            KernelEventPayload::Error {
                recoverable: false,
                ..
            },
            KernelEventPayload::Outcome {
                status: OutcomeStatus::Failed,
                ..
            }
        ]
    ));
    failed.finish().expect("failed turn is terminal");

    // Given/When: a standalone runtime error contains untrusted sensitive text.
    let mut runtime_error = CodexJsonDecoder::new();
    let error = runtime_error
        .decode_line(r#"{"type":"error","message":"api_key=SENTINEL-Aa1234567890_SECRET"}"#)
        .expect("decode runtime error");
    let serialized = serde_json::to_string(&error.payloads).expect("serialize error payload");

    // Then: the error remains observable and its message is redacted.
    assert!(matches!(
        error.payloads.as_slice(),
        [KernelEventPayload::Error {
            recoverable: false,
            ..
        }]
    ));
    assert!(!serialized.contains("SENTINEL"));
    assert!(serialized.contains("[REDACTED]"));

    // Given/When: command arguments and output contain the same sentinel.
    let mut command = CodexJsonDecoder::new();
    decode(
        &mut command,
        r#"{"type":"thread.started","thread_id":"thread-secret"}"#,
    );
    decode(&mut command, r#"{"type":"turn.started"}"#);
    let start = command
        .decode_line(
            r#"{"type":"item.started","item":{"id":"item_0","type":"command_execution","command":"echo api_key=SENTINEL-Aa1234567890_SECRET","aggregated_output":"","exit_code":null,"status":"in_progress"}}"#,
        )
        .expect("decode secret-bearing command");
    let end = command
        .decode_line(
            r#"{"type":"item.completed","item":{"id":"item_0","type":"command_execution","command":"echo api_key=SENTINEL-Aa1234567890_SECRET","aggregated_output":"api_key=SENTINEL-Aa1234567890_SECRET","exit_code":0,"status":"completed"}}"#,
        )
        .expect("decode secret-bearing output");
    let serialized =
        serde_json::to_string(&(start.payloads, end.payloads)).expect("serialize command payloads");

    // Then: neither normalized tool boundary retains the sentinel.
    assert!(!serialized.contains("SENTINEL"));
    assert!(serialized.contains("[REDACTED]"));
}

#[test]
fn known_and_future_non_tool_items_are_observable_without_payloads() {
    // Given: every documented non-tool item family and one future family.
    let mut decoder = CodexJsonDecoder::new();
    decode(
        &mut decoder,
        r#"{"type":"thread.started","thread_id":"thread-items"}"#,
    );
    decode(&mut decoder, r#"{"type":"turn.started"}"#);

    // When/Then: documented items remain payload-free and explicitly counted.
    for (index, kind) in [
        "agent_message",
        "reasoning",
        "file_change",
        "mcp_tool_call",
        "web_search",
        "plan_update",
    ]
    .into_iter()
    .enumerate()
    {
        let line = format!(
            r#"{{"type":"item.completed","item":{{"id":"item_{index}","type":"{kind}"}}}}"#
        );
        let documented = decoder.decode_line(&line).expect("decode documented item");
        assert!(documented.payloads.is_empty());
        assert_eq!(documented.disposition, DecodeDisposition::KnownIgnored);
    }

    // When/Then: a future family remains an observable compatibility skip.
    let future = decoder
        .decode_line(
            r#"{"type":"item.completed","item":{"id":"item_future","type":"future_item","value":1}}"#,
        )
        .expect("decode future item");
    assert!(future.payloads.is_empty());
    assert_eq!(future.disposition, DecodeDisposition::UnknownSkipped);
}

fn decode(decoder: &mut CodexJsonDecoder, line: &str) {
    decoder.decode_line(line).expect("decode setup line");
}
