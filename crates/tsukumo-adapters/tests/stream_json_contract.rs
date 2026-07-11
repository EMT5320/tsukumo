//! Contract tests for the Claude stream-json normalization boundary.

use std::cell::Cell;
use std::io::Read;
use std::rc::Rc;
use tsukumo_adapters::{
    parse_stream_json_line, parse_stream_json_reader, parse_stream_json_str, AdapterError,
    DecodeError,
};
use tsukumo_kernel::{
    KernelEventPayload, OutcomeStatus, PersistedJson, PersistedText, VendorEventRef,
};

#[test]
fn tool_use_normalizes_to_vendor_neutral_payload() {
    // Given: one documented Claude assistant tool-use block.
    let line = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"toolu_1","name":"Bash","input":{"command":"git status"}}]}}"#;

    // When: the adapter decodes the vendor line.
    let payloads = parse_stream_json_line(line).expect("decode tool use");

    // Then: only a namespaced, vendor-neutral payload crosses the boundary.
    assert_eq!(
        payloads,
        vec![KernelEventPayload::ToolStart {
            vendor_call: VendorEventRef::new("claude_cli", "toolu_1"),
            tool: "Bash".into(),
            args: Some(PersistedJson::from_reviewed(
                serde_json::json!({"command": "git status"})
            )),
            projection_id: None,
        }]
    );
}

#[test]
fn malformed_known_tool_use_is_an_error() {
    // Given: a known tool-use shape with no durable vendor call ID.
    let line = r#"{"type":"tool_use","name":"Bash","input":{"command":"git status"}}"#;

    // When: the adapter decodes the malformed known event.
    let error = parse_stream_json_line(line).expect_err("missing id must fail");

    // Then: the decoder reports the exact known boundary field.
    assert!(matches!(
        error,
        DecodeError::MalformedKnown {
            event_type: "tool_use",
            field: "id"
        }
    ));
}

#[test]
fn permission_request_and_outcome_use_shared_payload_contract() {
    // Given: a permission request followed by a successful result.
    let body = concat!(
        "{\"type\":\"sdk_control_request\",\"request\":{\"subtype\":\"permission\",",
        "\"request_id\":\"perm_1\",\"tool_name\":\"Bash\",",
        "\"tool_input\":{\"command\":\"cargo test\"}}}\n",
        "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,",
        "\"result\":\"done\"}\n"
    );

    // When: the recorded stream is normalized.
    let payloads = parse_stream_json_str(body).expect("decode fixture");

    // Then: permission and outcome variants are provider-neutral and reviewed.
    assert_eq!(
        payloads[0],
        KernelEventPayload::PermissionRequested {
            vendor_request: VendorEventRef::new("claude_cli", "perm_1"),
            tool: "Bash".into(),
            arguments: Some(PersistedJson::from_reviewed(
                serde_json::json!({"command": "cargo test"}),
            )),
            cwd: None,
            risk_reasons: Vec::new(),
            reason: PersistedText::from_reviewed("Bash: cargo test"),
        }
    );
    assert_eq!(
        payloads[1],
        KernelEventPayload::Outcome {
            status: OutcomeStatus::Succeeded,
            summary: Some(PersistedText::from_reviewed("done")),
            projection_id: None,
        }
    );
}

#[test]
fn multiline_errors_preserve_the_vendor_line_number() {
    // Given: one ignored event before a malformed known tool result.
    let body = concat!(
        "{\"type\":\"system\",\"subtype\":\"init\"}\n",
        "{\"type\":\"tool_result\",\"content\":\"missing call id\"}\n"
    );

    // When: the multi-line helper decodes the stream.
    let error = parse_stream_json_str(body).expect_err("known malformed line must fail");

    // Then: the caller receives exact line context and a typed source error.
    assert!(matches!(
        error,
        AdapterError::Decode {
            line: 2,
            source: DecodeError::MalformedKnown {
                event_type: "tool_result",
                field: "tool_use_id"
            }
        }
    ));
}

#[test]
fn malformed_result_fields_never_fabricate_success() {
    for (line, expected) in [
        (r#"{"type":"result","is_error":false}"#, "missing subtype"),
        (
            r#"{"type":"result","subtype":"success","is_error":"false"}"#,
            "invalid is_error",
        ),
        (
            r#"{"type":"result","subtype":"future_status","is_error":false}"#,
            "unsupported subtype",
        ),
    ] {
        let error = parse_stream_json_line(line).expect_err(expected);
        match expected {
            "missing subtype" => assert!(matches!(
                error,
                DecodeError::MalformedKnown {
                    event_type: "result",
                    field: "subtype"
                }
            )),
            "invalid is_error" => assert!(matches!(
                error,
                DecodeError::InvalidKnown {
                    event_type: "result",
                    field: "is_error"
                }
            )),
            "unsupported subtype" => {
                assert!(matches!(error, DecodeError::UnsupportedKnown { .. }))
            }
            _ => unreachable!("covered test case"),
        }
    }
}

#[test]
fn vendor_secrets_and_control_sequences_are_redacted_before_normalization() {
    let body = concat!(
        "{\"type\":\"tool_use\",\"id\":\"toolu_safe\",\"name\":\"Bash\",",
        "\"input\":{\"command\":\"echo ok\",\"api_key\":\"SENTINEL-Aa1234567890_SECRET\"}}\n",
        "{\"type\":\"tool_result\",\"tool_use_id\":\"toolu_safe\",",
        "\"content\":\"api_key=SENTINEL-Aa1234567890_SECRET\"}\n"
    );

    let payloads = parse_stream_json_str(body).expect("decode redacted vendor stream");
    let serialized = serde_json::to_string(&payloads).expect("serialize normalized payloads");
    assert!(!serialized.contains("SENTINEL"));
    assert!(serialized.contains("[REDACTED]"));
    assert!(!format!("{:?}", payloads[0]).contains("echo ok"));
}

struct CountingReader {
    body: Vec<u8>,
    offset: usize,
    consumed: Rc<Cell<usize>>,
}

impl Read for CountingReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let remaining = &self.body[self.offset..];
        let count = remaining.len().min(buffer.len());
        buffer[..count].copy_from_slice(&remaining[..count]);
        self.offset += count;
        self.consumed.set(self.offset);
        Ok(count)
    }
}

#[test]
fn reader_stops_consuming_once_the_line_budget_is_exceeded() {
    // Given: a runtime stream with one unbounded line and an observable reader count.
    let consumed = Rc::new(Cell::new(0));
    let reader = CountingReader {
        body: vec![b'x'; 2_097_152],
        offset: 0,
        consumed: consumed.clone(),
    };

    // When: the streaming boundary encounters the configured one-megabyte limit.
    let error = parse_stream_json_reader(reader).expect_err("oversized line must fail");

    // Then: the decoder stops close to the boundary instead of buffering the full line.
    assert!(matches!(
        error,
        AdapterError::Decode {
            source: DecodeError::LineTooLarge { .. },
            ..
        }
    ));
    assert!(
        consumed.get() < 1_100_000,
        "reader consumed {} bytes",
        consumed.get()
    );
}
