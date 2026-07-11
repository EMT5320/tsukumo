//! Claude stream-json JSONL decoder.
//!
//! This module owns vendor parsing and returns only normalized kernel payloads.
//! Durable envelope identity, timestamps, and Chronicle ordering belong to host.

mod mapping;

use crate::runtime::{DecodeDisposition, DecodedRuntimeLine, RuntimeEventDecoder};
use std::io::{BufRead, BufReader, ErrorKind, Read};
use thiserror::Error;
use tsukumo_kernel::{redact_sensitive_text, KernelEventPayload};

const MAX_LINE_BYTES: usize = 1_048_576;

/// One-line Claude stream decoding failure.
#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("malformed known {event_type} event: missing {field}")]
    MalformedKnown {
        event_type: &'static str,
        field: &'static str,
    },
    #[error("malformed known {event_type} event: invalid {field}")]
    InvalidKnown {
        event_type: &'static str,
        field: &'static str,
    },
    #[error("unsupported known {event_type} event {field}: {value}")]
    UnsupportedKnown {
        event_type: &'static str,
        field: &'static str,
        value: String,
    },
    #[error("vendor JSONL line is {bytes} bytes; maximum is {maximum}")]
    LineTooLarge { bytes: usize, maximum: usize },
}

impl DecodeError {
    pub(crate) const fn missing(event_type: &'static str, field: &'static str) -> Self {
        Self::MalformedKnown { event_type, field }
    }

    pub(crate) const fn invalid(event_type: &'static str, field: &'static str) -> Self {
        Self::InvalidKnown { event_type, field }
    }

    pub(crate) fn unsupported(
        event_type: &'static str,
        field: &'static str,
        value: impl AsRef<str>,
    ) -> Self {
        let value = redact_sensitive_text(value.as_ref())
            .chars()
            .take(128)
            .collect();
        Self::UnsupportedKnown {
            event_type,
            field,
            value,
        }
    }
}

/// Multi-line stream failure with exact source line context.
#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("decode error on line {line}: {source}")]
    Decode {
        line: usize,
        #[source]
        source: DecodeError,
    },
    #[error("runtime stream ended after {lines} lines without one terminal result")]
    TruncatedStream { lines: usize },
    #[error("runtime stream emitted more than one terminal result on line {line}")]
    DuplicateTerminal { line: usize },
}

/// Stateful Claude decoder shared by recorded fixtures and live stdout.
#[derive(Debug, Default)]
pub struct ClaudeStreamDecoder {
    lines: usize,
    terminal_seen: bool,
}

impl ClaudeStreamDecoder {
    pub const fn new() -> Self {
        Self {
            lines: 0,
            terminal_seen: false,
        }
    }
}

impl RuntimeEventDecoder for ClaudeStreamDecoder {
    fn decode_line(&mut self, line: &str) -> Result<DecodedRuntimeLine, AdapterError> {
        self.lines += 1;
        let decoded = decode_runtime_line(line).map_err(|source| AdapterError::Decode {
            line: self.lines,
            source,
        })?;
        let terminal_count = decoded
            .payloads
            .iter()
            .filter(|payload| matches!(payload, KernelEventPayload::Outcome { .. }))
            .count();
        if terminal_count > 1 || (terminal_count == 1 && self.terminal_seen) {
            return Err(AdapterError::DuplicateTerminal { line: self.lines });
        }
        if terminal_count == 1 {
            self.terminal_seen = true;
        }
        Ok(DecodedRuntimeLine {
            line_number: self.lines,
            disposition: decoded.disposition,
            payloads: decoded.payloads,
        })
    }

    fn finish(&self) -> Result<(), AdapterError> {
        if self.terminal_seen {
            Ok(())
        } else {
            Err(AdapterError::TruncatedStream { lines: self.lines })
        }
    }
}

struct ParsedRuntimeLine {
    disposition: DecodeDisposition,
    payloads: Vec<KernelEventPayload>,
}

/// Parses one vendor JSONL line into zero or more normalized payloads.
pub fn parse_stream_json_line(line: &str) -> Result<Vec<KernelEventPayload>, DecodeError> {
    Ok(decode_runtime_line(line)?.payloads)
}

fn decode_runtime_line(line: &str) -> Result<ParsedRuntimeLine, DecodeError> {
    if line.len() > MAX_LINE_BYTES {
        return Err(DecodeError::LineTooLarge {
            bytes: line.len(),
            maximum: MAX_LINE_BYTES,
        });
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(ParsedRuntimeLine {
            disposition: DecodeDisposition::KnownIgnored,
            payloads: Vec::new(),
        });
    }
    let value: serde_json::Value = serde_json::from_str(trimmed)?;
    let payloads = mapping::map_value(&value)?;
    let disposition = classify_disposition(&value, &payloads);
    Ok(ParsedRuntimeLine {
        disposition,
        payloads,
    })
}

fn classify_disposition(
    value: &serde_json::Value,
    payloads: &[KernelEventPayload],
) -> DecodeDisposition {
    if !payloads.is_empty() {
        return DecodeDisposition::Emitted;
    }
    match value.get("type").and_then(serde_json::Value::as_str) {
        Some(
            "assistant"
            | "user"
            | "tool_use"
            | "tool_result"
            | "sdk_control_request"
            | "control_request"
            | "result"
            | "system"
            | "stream_event"
            | "rate_limit_event",
        ) => DecodeDisposition::KnownIgnored,
        Some(_) | None => DecodeDisposition::UnknownSkipped,
    }
}
/// Parses a recorded multi-line stream while retaining line-scoped errors.
pub fn parse_stream_json_str(body: &str) -> Result<Vec<KernelEventPayload>, AdapterError> {
    let mut payloads = Vec::new();
    for (idx, line) in body.lines().enumerate() {
        let line_no = idx + 1;
        let mut decoded = parse_stream_json_line(line).map_err(|source| AdapterError::Decode {
            line: line_no,
            source,
        })?;
        payloads.append(&mut decoded);
    }
    Ok(payloads)
}

/// Parses stream-json from a file or pipe through the same line decoder.
pub fn parse_stream_json_reader<R: Read>(
    reader: R,
) -> Result<Vec<KernelEventPayload>, AdapterError> {
    let mut reader = BufReader::new(reader);
    let mut payloads = Vec::new();
    let mut line = Vec::new();
    let mut line_no = 1;
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            if !line.is_empty() {
                decode_reader_line(&line, line_no, &mut payloads)?;
            }
            break;
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(available.len(), |index| index + 1);
        let payload_bytes = newline.unwrap_or(available.len());
        if line.len().saturating_add(payload_bytes) > MAX_LINE_BYTES {
            return Err(AdapterError::Decode {
                line: line_no,
                source: DecodeError::LineTooLarge {
                    bytes: line.len().saturating_add(payload_bytes),
                    maximum: MAX_LINE_BYTES,
                },
            });
        }
        line.extend_from_slice(&available[..payload_bytes]);
        reader.consume(consumed);
        if newline.is_some() {
            decode_reader_line(&line, line_no, &mut payloads)?;
            line.clear();
            line_no += 1;
        }
    }
    Ok(payloads)
}

fn decode_reader_line(
    line: &[u8],
    line_no: usize,
    payloads: &mut Vec<KernelEventPayload>,
) -> Result<(), AdapterError> {
    let line = std::str::from_utf8(line)
        .map_err(|error| std::io::Error::new(ErrorKind::InvalidData, error))?;
    let mut decoded = parse_stream_json_line(line).map_err(|source| AdapterError::Decode {
        line: line_no,
        source,
    })?;
    payloads.append(&mut decoded);
    Ok(())
}
