//! Claude stream-json JSONL decoder.
//!
//! This module owns vendor parsing and returns only normalized kernel payloads.
//! Durable envelope identity, timestamps, and Chronicle ordering belong to host.

mod mapping;

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
}

/// Parses one vendor JSONL line into zero or more normalized payloads.
pub fn parse_stream_json_line(line: &str) -> Result<Vec<KernelEventPayload>, DecodeError> {
    if line.len() > MAX_LINE_BYTES {
        return Err(DecodeError::LineTooLarge {
            bytes: line.len(),
            maximum: MAX_LINE_BYTES,
        });
    }
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let value = serde_json::from_str(trimmed)?;
    mapping::map_value(&value)
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
