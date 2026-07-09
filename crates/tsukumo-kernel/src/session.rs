//! Tiny session / JSONL helpers for fixture replay and later K0.

use crate::event::KernelEvent;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error on line {line}: {source}")]
    Json {
        line: usize,
        #[source]
        source: serde_json::Error,
    },
    #[error("empty line at {line}")]
    EmptyLine { line: usize },
}

/// Parse one JSONL line into a [`KernelEvent`].
pub fn parse_jsonl_line(line: &str) -> Result<KernelEvent, serde_json::Error> {
    serde_json::from_str(line.trim())
}

/// Read an entire JSONL file of [`KernelEvent`]s (blank lines skipped).
pub fn read_jsonl_events(path: impl AsRef<Path>) -> Result<Vec<KernelEvent>, SessionError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line_no = idx + 1;
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let event = parse_jsonl_line(&line).map_err(|source| SessionError::Json {
            line: line_no,
            source,
        })?;
        events.push(event);
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{KernelEvent, ToolResult};
    use std::io::Write;
    use std::path::PathBuf;

    fn write_temp_jsonl(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("tsukumo-kernel-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        path
    }

    #[test]
    fn reads_multiline_jsonl() {
        let body = r#"
{"type":"tool_start","call_id":"1","tool":"read"}
{"type":"tool_end","call_id":"1","result":{"summary":"ok"},"is_error":false}
{"type":"turn_or_quest_end","summary":"done"}
"#;
        let path = write_temp_jsonl("sample.jsonl", body);
        let events = read_jsonl_events(&path).unwrap();
        assert_eq!(events.len(), 3);
        match &events[1] {
            KernelEvent::ToolEnd {
                result, is_error, ..
            } => {
                assert_eq!(result, &ToolResult::text("ok"));
                assert!(!is_error);
            }
            other => panic!("unexpected: {other:?}"),
        }
    }
}
