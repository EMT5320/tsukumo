//! Soft identity stubs: executor identity ≠ runtime backend.
//!
//! Do not introduce dual species tables (付丧神 vs 雇佣兵). Growth and
//! memory attach to [`ExecutorId`], not to vendor strings.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Opaque executor identity. Stable across backend swaps.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExecutorId(pub String);

impl ExecutorId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExecutorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for ExecutorId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ExecutorId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// How an executor is currently backed. Orthogonal to identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    Builtin,
    Acp,
    StreamJson,
    Watcher,
    /// Fixture / recorded replay — not a live runtime.
    Fixture,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executor_id_roundtrips_json() {
        let id = ExecutorId::new("gina");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"gina\"");
        let back: ExecutorId = serde_json::from_str(&json).unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn backend_kind_is_snake_case() {
        let json = serde_json::to_string(&BackendKind::StreamJson).unwrap();
        assert_eq!(json, "\"stream_json\"");
    }
}
