//! Legacy inject/recall JSONL trace compatibility surface.
//!
//! New durable evidence belongs in Chronicle. This file remains append-only and
//! every write error is returned to the caller.

use crate::storage::SoulError;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use tsukumo_kernel::{QuestId, SessionId};

/// One typed compatibility trace line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TraceEvent {
    Inject {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        quest_id: Option<QuestId>,
        brief_chars: usize,
        goal_chars: usize,
    },
    Recall {
        query: String,
        hit_count: usize,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        session_id: Option<SessionId>,
    },
}

/// Append-only compatibility JSONL trace log.
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
        let line = serde_json::to_string(&event)?;
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
    fn appends_typed_jsonl_line() {
        // Given: an empty compatibility trace.
        let directory = tempdir().expect("create trace test directory");
        let mut log = TraceLog::open(directory.path());

        // When: a typed recall line is appended.
        log.append(TraceEvent::Recall {
            query: "gnu".into(),
            hit_count: 1,
            session_id: Some(SessionId::new("session-2")),
        })
        .expect("append recall trace");

        // Then: the JSONL line preserves its typed wire values.
        let body = std::fs::read_to_string(log.path()).expect("read trace");
        assert!(body.contains("\"type\":\"recall\""));
        assert!(body.contains("\"session_id\":\"session-2\""));
    }
}
