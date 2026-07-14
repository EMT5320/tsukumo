//! Stateful Codex `exec --json` decoder.

mod mapping;

use crate::runtime::{DecodeDisposition, DecodedRuntimeLine, RuntimeEventDecoder};
use crate::stream_json::{AdapterError, DecodeError, MAX_RUNTIME_LINE_BYTES};
use crate::vendor_fields::{optional_string, required_label};
use mapping::{is_documented_ignored_item, item_identity, map_command_end, map_command_start};
use serde_json::Value;
use std::collections::HashSet;
use tsukumo_kernel::{KernelEventPayload, OutcomeStatus, PersistedText};

/// Stateful decoder for one non-interactive Codex turn.
#[derive(Debug, Default)]
pub struct CodexJsonDecoder {
    lines: usize,
    thread_id: Option<String>,
    turn_open: bool,
    terminal_seen: bool,
    pending_commands: HashSet<String>,
    tool_error_seen: bool,
}

impl CodexJsonDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode_value(
        &mut self,
        value: &Value,
    ) -> Result<(DecodeDisposition, Vec<KernelEventPayload>), DecodeError> {
        let Some(event_type) = value.get("type").and_then(Value::as_str) else {
            return Ok((DecodeDisposition::UnknownSkipped, Vec::new()));
        };
        if self.terminal_seen {
            return Err(DecodeError::invalid(
                "runtime event after terminal",
                "sequence",
            ));
        }
        match event_type {
            "thread.started" => self.start_thread(value),
            "turn.started" => self.start_turn(),
            "item.started" => self.decode_item(value, ItemPhase::Started),
            "item.updated" => self.decode_item(value, ItemPhase::Updated),
            "item.completed" => self.decode_item(value, ItemPhase::Completed),
            "turn.completed" => self.complete_turn(false),
            "turn.failed" => self.complete_turn(true),
            "error" => map_runtime_error(value),
            _ => Ok((DecodeDisposition::UnknownSkipped, Vec::new())),
        }
    }

    fn start_thread(
        &mut self,
        value: &Value,
    ) -> Result<(DecodeDisposition, Vec<KernelEventPayload>), DecodeError> {
        if self.thread_id.is_some() || self.turn_open || self.terminal_seen {
            return Err(DecodeError::invalid("thread.started", "sequence"));
        }
        self.thread_id = Some(required_label(value, "thread_id", "thread.started")?);
        Ok((DecodeDisposition::KnownIgnored, Vec::new()))
    }

    fn start_turn(&mut self) -> Result<(DecodeDisposition, Vec<KernelEventPayload>), DecodeError> {
        if self.thread_id.is_none() || self.turn_open || self.terminal_seen {
            return Err(DecodeError::invalid("turn.started", "sequence"));
        }
        self.turn_open = true;
        Ok((DecodeDisposition::KnownIgnored, Vec::new()))
    }

    fn decode_item(
        &mut self,
        value: &Value,
        phase: ItemPhase,
    ) -> Result<(DecodeDisposition, Vec<KernelEventPayload>), DecodeError> {
        if !self.turn_open || self.terminal_seen {
            return Err(DecodeError::invalid(phase.event_type(), "sequence"));
        }
        let identity = item_identity(value, phase.event_type())?;
        if identity.kind != "command_execution" {
            let disposition = if is_documented_ignored_item(identity.kind) {
                DecodeDisposition::KnownIgnored
            } else {
                DecodeDisposition::UnknownSkipped
            };
            return Ok((disposition, Vec::new()));
        }
        let thread_id = self.thread_id.clone().expect("open turn has thread");
        match phase {
            ItemPhase::Started => {
                if !self.pending_commands.insert(identity.id.clone()) {
                    return Err(DecodeError::invalid(phase.event_type(), "item.id sequence"));
                }
                Ok((
                    DecodeDisposition::Emitted,
                    vec![map_command_start(identity.value, &thread_id, &identity.id)?],
                ))
            }
            ItemPhase::Completed => {
                if !self.pending_commands.contains(&identity.id) {
                    return Err(DecodeError::invalid(phase.event_type(), "item.id sequence"));
                }
                let payload = map_command_end(identity.value, &thread_id, &identity.id)?;
                self.pending_commands.remove(&identity.id);
                self.tool_error_seen |=
                    matches!(&payload, KernelEventPayload::ToolEnd { is_error: true, .. });
                Ok((DecodeDisposition::Emitted, vec![payload]))
            }
            ItemPhase::Updated => {
                if !self.pending_commands.contains(&identity.id) {
                    return Err(DecodeError::invalid(phase.event_type(), "item.id sequence"));
                }
                Ok((DecodeDisposition::KnownIgnored, Vec::new()))
            }
        }
    }

    fn complete_turn(
        &mut self,
        failed: bool,
    ) -> Result<(DecodeDisposition, Vec<KernelEventPayload>), DecodeError> {
        if self.terminal_seen {
            return Err(DecodeError::invalid("turn terminal", "duplicate"));
        }
        if !self.turn_open || !self.pending_commands.is_empty() {
            return Err(DecodeError::invalid("turn terminal", "sequence"));
        }
        self.turn_open = false;
        self.terminal_seen = true;
        let completed_with_tool_errors = !failed && self.tool_error_seen;
        let status = if failed || completed_with_tool_errors {
            OutcomeStatus::Failed
        } else {
            OutcomeStatus::Succeeded
        };
        let mut payloads = Vec::with_capacity(if failed { 2 } else { 1 });
        if failed {
            payloads.push(KernelEventPayload::Error {
                message: PersistedText::from_reviewed("Codex turn failed"),
                recoverable: false,
            });
        }
        payloads.push(KernelEventPayload::Outcome {
            status,
            summary: if failed {
                Some(PersistedText::from_reviewed("Codex turn failed"))
            } else if completed_with_tool_errors {
                Some(PersistedText::from_reviewed(
                    "Codex turn completed with tool errors",
                ))
            } else {
                None
            },
            projection_id: None,
        });
        Ok((DecodeDisposition::Emitted, payloads))
    }
}

impl RuntimeEventDecoder for CodexJsonDecoder {
    fn decode_line(&mut self, line: &str) -> Result<DecodedRuntimeLine, AdapterError> {
        self.lines += 1;
        let line_number = self.lines;
        if line.len() > MAX_RUNTIME_LINE_BYTES {
            return Err(AdapterError::Decode {
                line: line_number,
                source: DecodeError::LineTooLarge {
                    bytes: line.len(),
                    maximum: MAX_RUNTIME_LINE_BYTES,
                },
            });
        }
        if line.trim().is_empty() {
            return Ok(DecodedRuntimeLine {
                line_number,
                disposition: DecodeDisposition::KnownIgnored,
                payloads: Vec::new(),
            });
        }
        let value: Value =
            serde_json::from_str(line.trim()).map_err(|source| AdapterError::Decode {
                line: line_number,
                source: DecodeError::Json(source),
            })?;
        if self.terminal_seen
            && matches!(
                value.get("type").and_then(Value::as_str),
                Some("turn.completed" | "turn.failed")
            )
        {
            return Err(AdapterError::DuplicateTerminal { line: line_number });
        }
        let (disposition, payloads) =
            self.decode_value(&value)
                .map_err(|source| AdapterError::Decode {
                    line: line_number,
                    source,
                })?;
        Ok(DecodedRuntimeLine {
            line_number,
            disposition,
            payloads,
        })
    }

    fn finish(&self) -> Result<(), AdapterError> {
        if self.terminal_seen && !self.turn_open && self.pending_commands.is_empty() {
            Ok(())
        } else {
            Err(AdapterError::TruncatedStream { lines: self.lines })
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ItemPhase {
    Started,
    Updated,
    Completed,
}

impl ItemPhase {
    const fn event_type(self) -> &'static str {
        match self {
            Self::Started => "item.started",
            Self::Updated => "item.updated",
            Self::Completed => "item.completed",
        }
    }
}

fn map_runtime_error(
    value: &Value,
) -> Result<(DecodeDisposition, Vec<KernelEventPayload>), DecodeError> {
    let message = optional_string(value, "message", "error")?.unwrap_or("Codex runtime error");
    Ok((
        DecodeDisposition::Emitted,
        vec![KernelEventPayload::Error {
            message: PersistedText::from_redacted(message),
            recoverable: false,
        }],
    ))
}
