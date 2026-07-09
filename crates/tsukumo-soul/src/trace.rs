//! Inject / recall trace stub (R6).
//!
//! Append-only JSONL under the soul data dir. Schema is intentionally thin
//! so A1 / later §8.6 can extend without breaking the probe.

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::store::SoulError;

/// One trace line for inject or recall observability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TraceEvent {
    Inject {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        quest_id: Option<String>,
        brief_chars: usize,
        goal_chars: usize,
    },
    Recall {
        query: String,
        hit_count: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
    },
}

/// Append-only JSONL trace log (`inject_trace.jsonl` by default).
#[derive(Debug)]
pub struct TraceLog {
    path: PathBuf,
}

impl TraceLog {
    pub fn open(data_dir: impl AsRef<Path>) -> Self {
        Self {
            path: data_dir.as_ref().join("inject_trace.jsonl"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&mut self, event: TraceEvent) -> Result<(), SoulError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(&event).map_err(|e| {
            SoulError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn appends_jsonl_line() {
        let dir = tempdir().unwrap();
        let mut log = TraceLog::open(dir.path());
        log.append(TraceEvent::Recall {
            query: "gnu".into(),
            hit_count: 1,
            session_id: Some("s2".into()),
        })
        .unwrap();
        let body = std::fs::read_to_string(log.path()).unwrap();
        assert!(body.contains("\"type\":\"recall\""));
        assert!(body.contains("gnu"));
    }
}
