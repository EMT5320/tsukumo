//! JSONL helpers for persisted event fixtures and Chronicle replay.

use crate::event::KernelEvent;
use crate::{validate_kernel_event, EventContractError};
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read};
use std::path::Path;
use thiserror::Error;

const MAX_EVENT_LINE_BYTES: usize = 1_048_576;

#[derive(Debug, Error)]
pub enum EventDecodeError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Contract(#[from] EventContractError),
    #[error("durable event JSONL line is {bytes} bytes; maximum is {maximum}")]
    LineTooLarge { bytes: usize, maximum: usize },
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("event decode error on line {line}: {source}")]
    Decode {
        line: usize,
        #[source]
        source: EventDecodeError,
    },
}

/// Parses and validates one durable KernelEvent JSONL line.
pub fn parse_jsonl_line(line: &str) -> Result<KernelEvent, EventDecodeError> {
    let event = serde_json::from_str(line.trim())?;
    validate_kernel_event(&event)?;
    Ok(event)
}

/// Reads validated KernelEvent envelopes while preserving source line context.
pub fn read_jsonl_events(path: impl AsRef<Path>) -> Result<Vec<KernelEvent>, SessionError> {
    read_jsonl_reader(File::open(path)?)
}

/// Reads validated envelopes from a bounded file or pipe.
pub fn read_jsonl_reader<R: Read>(reader: R) -> Result<Vec<KernelEvent>, SessionError> {
    let mut reader = BufReader::new(reader);
    let mut events = Vec::new();
    let mut line = Vec::new();
    let mut line_no = 1;
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            if !line.is_empty() {
                decode_reader_line(&line, line_no, &mut events)?;
            }
            break;
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(available.len(), |index| index + 1);
        let payload_bytes = newline.unwrap_or(available.len());
        if line.len().saturating_add(payload_bytes) > MAX_EVENT_LINE_BYTES {
            return Err(SessionError::Decode {
                line: line_no,
                source: EventDecodeError::LineTooLarge {
                    bytes: line.len().saturating_add(payload_bytes),
                    maximum: MAX_EVENT_LINE_BYTES,
                },
            });
        }
        line.extend_from_slice(&available[..payload_bytes]);
        reader.consume(consumed);
        if newline.is_some() {
            decode_reader_line(&line, line_no, &mut events)?;
            line.clear();
            line_no += 1;
        }
    }
    Ok(events)
}

fn decode_reader_line(
    line: &[u8],
    line_no: usize,
    events: &mut Vec<KernelEvent>,
) -> Result<(), SessionError> {
    let line = std::str::from_utf8(line)
        .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error))?;
    if line.trim().is_empty() {
        return Ok(());
    }
    let event = parse_jsonl_line(line).map_err(|source| SessionError::Decode {
        line: line_no,
        source,
    })?;
    events.push(event);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CorrelationId, EventId, ExecutionId, KernelEventPayload, OutcomeStatus, PersistedText,
        ProjectionId, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SessionId, SpiritId,
        Timestamp, ToolResult, VendorEventRef, KERNEL_EVENT_SCHEMA_VERSION,
    };
    use std::io::Write;
    use std::path::PathBuf;

    fn write_temp_jsonl(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("tsukumo-kernel-tests");
        std::fs::create_dir_all(&dir).expect("create kernel test directory");
        let path = dir.join(name);
        let mut file = File::create(&path).expect("create kernel fixture");
        file.write_all(body.as_bytes())
            .expect("write kernel fixture");
        path
    }

    fn fixture_event(id: &str, payload: KernelEventPayload) -> KernelEvent {
        KernelEvent {
            schema_version: KERNEL_EVENT_SCHEMA_VERSION,
            event_id: EventId::new(id),
            occurred_at: Timestamp::from_unix_millis(1_750_000_000_000),
            quest_id: QuestId::new("quest-fixture"),
            session_id: SessionId::new("session-fixture"),
            spirit_id: SpiritId::new("yuka"),
            execution_id: Some(ExecutionId::new("execution-fixture")),
            runtime: Some(RuntimeBinding::new(
                RuntimeKind::ClaudeCli,
                RuntimeMode::Fixture,
            )),
            causation_id: None,
            correlation_id: Some(CorrelationId::new("correlation-fixture")),
            payload,
        }
    }

    #[test]
    fn reads_multiline_jsonl_envelopes() {
        // Given: three durable envelopes separated by a blank line.
        let events = [
            fixture_event(
                "event-1",
                KernelEventPayload::ToolStart {
                    vendor_call: VendorEventRef::new("fixture", "call-1"),
                    tool: "read".into(),
                    args: None,
                    projection_id: Some(ProjectionId::new("projection-fixture")),
                },
            ),
            fixture_event(
                "event-2",
                KernelEventPayload::ToolEnd {
                    vendor_call: VendorEventRef::new("fixture", "call-1"),
                    result: ToolResult::reviewed_text("ok"),
                    is_error: false,
                    projection_id: Some(ProjectionId::new("projection-fixture")),
                },
            ),
            fixture_event(
                "event-3",
                KernelEventPayload::Outcome {
                    status: OutcomeStatus::Succeeded,
                    summary: Some(PersistedText::from_reviewed("done")),
                    projection_id: None,
                },
            ),
        ];
        let body = format!(
            "{}\n\n{}\n{}\n",
            serde_json::to_string(&events[0]).expect("serialize first fixture event"),
            serde_json::to_string(&events[1]).expect("serialize second fixture event"),
            serde_json::to_string(&events[2]).expect("serialize third fixture event"),
        );
        let path = write_temp_jsonl("sample-c1.jsonl", &body);

        // When: the fixture is reopened through the shared JSONL path.
        let reopened = read_jsonl_events(&path).expect("read C1 fixture");

        // Then: order and typed payload content survive.
        assert_eq!(reopened, events);
        match &reopened[1].payload {
            KernelEventPayload::ToolEnd {
                result, is_error, ..
            } => {
                assert_eq!(result, &ToolResult::reviewed_text("ok"));
                assert!(!is_error);
            }
            other => panic!("unexpected payload: {other:?}"),
        }
    }

    #[test]
    fn rejects_newer_schema_and_unattributed_tool_events() {
        let newer = fixture_event(
            "event-newer",
            KernelEventPayload::Outcome {
                status: OutcomeStatus::Succeeded,
                summary: None,
                projection_id: None,
            },
        );
        let mut newer = newer;
        newer.schema_version = KERNEL_EVENT_SCHEMA_VERSION + 1;
        assert!(matches!(
            parse_jsonl_line(&serde_json::to_string(&newer).expect("serialize newer event")),
            Err(EventDecodeError::Contract(
                EventContractError::UnsupportedSchema { .. }
            ))
        ));

        let mut unattributed = fixture_event(
            "event-unattributed",
            KernelEventPayload::ToolStart {
                vendor_call: VendorEventRef::new("fixture", "call-1"),
                tool: "read".into(),
                args: None,
                projection_id: Some(ProjectionId::new("projection-fixture")),
            },
        );
        unattributed.correlation_id = None;
        assert!(matches!(
            parse_jsonl_line(
                &serde_json::to_string(&unattributed).expect("serialize unattributed event")
            ),
            Err(EventDecodeError::Contract(
                EventContractError::MissingAttribution {
                    field: "correlation_id",
                    ..
                }
            ))
        ));
    }
    struct GuardedReader {
        remaining: usize,
        consumed: usize,
    }

    impl std::io::Read for GuardedReader {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            if self.consumed > 1_100_000 {
                return Err(std::io::Error::other("reader exceeded bounded budget"));
            }
            let count = self.remaining.min(buffer.len());
            buffer[..count].fill(b'x');
            self.remaining -= count;
            self.consumed += count;
            Ok(count)
        }
    }

    #[test]
    fn durable_jsonl_reader_stops_at_the_line_budget() {
        // Given: one two-megabyte line from a corrupted durable source.
        let reader = GuardedReader {
            remaining: 2_097_152,
            consumed: 0,
        };

        // When/Then: the boundary rejects before the guarded reader reports overconsumption.
        assert!(matches!(
            read_jsonl_reader(reader),
            Err(SessionError::Decode {
                source: EventDecodeError::LineTooLarge { .. },
                ..
            })
        ));
    }
}
