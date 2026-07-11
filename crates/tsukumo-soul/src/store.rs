//! Compatibility facade for the A1 facts and recall APIs.
//!
//! New C1 writes use Chronicle and canonical state. The facts table and
//! MEMORY/USER files remain readable until explicit legacy import completes.

mod queries;

use crate::storage::{SoulError, SoulStore};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tsukumo_kernel::contains_sensitive_material;

const MAX_LEGACY_FACTS: i64 = 10_000;
const MAX_LEGACY_ID_CHARS: i64 = 256;
const MAX_LEGACY_KIND_CHARS: i64 = 16;
const MAX_LEGACY_SESSION_CHARS: i64 = 256;
const MAX_LEGACY_TEXT_CHARS: i64 = 16_384;
const MAX_TOTAL_LEGACY_CHARS: i64 = 16_777_216;

/// Legacy fact category used by the compatibility snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactKind {
    Memory,
    User,
}

impl FactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::User => "user",
        }
    }

    pub(crate) fn from_str(value: &str) -> Option<Self> {
        match value {
            "memory" => Some(Self::Memory),
            "user" => Some(Self::User),
            _ => None,
        }
    }

    fn snapshot_file(self) -> &'static str {
        match self {
            Self::Memory => "MEMORY.md",
            Self::User => "USER.md",
        }
    }
}

/// One legacy recallable fact row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryFact {
    pub id: String,
    pub kind: FactKind,
    pub text: String,
    pub session_id: String,
}

impl SoulStore {
    pub fn snapshot_path(&self, kind: FactKind) -> PathBuf {
        self.data_dir.join(kind.snapshot_file())
    }

    /// Writes one legacy fact and refreshes its compatibility snapshot.
    pub fn remember(&mut self, fact: MemoryFact) -> Result<(), SoulError> {
        let text = fact.text.trim();
        if text.is_empty() {
            return Err(SoulError::EmptyText);
        }
        if !valid_legacy_identifier(&fact.id, MAX_LEGACY_ID_CHARS)
            || !valid_legacy_identifier(&fact.session_id, MAX_LEGACY_SESSION_CHARS)
        {
            return Err(SoulError::InvalidLegacyMetadata);
        }
        if text.chars().count() > usize::try_from(MAX_LEGACY_TEXT_CHARS).unwrap_or(usize::MAX) {
            return Err(SoulError::LegacyBudgetExceeded);
        }
        if contains_sensitive_material(text) {
            return Err(SoulError::SensitiveLegacyContent);
        }
        let transaction = self.conn.transaction()?;
        transaction.execute(
            "INSERT OR REPLACE INTO facts (id, kind, text, session_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![fact.id, fact.kind.as_str(), text, fact.session_id],
        )?;
        // A later legacy write requires another explicit import pass.
        transaction.execute(
            "DELETE FROM legacy_import_runs WHERE source_table = 'facts'",
            [],
        )?;
        transaction.commit()?;
        self.rewrite_snapshot(fact.kind)
    }

    pub fn remember_memory(
        &mut self,
        id: impl Into<String>,
        session_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<(), SoulError> {
        self.remember(MemoryFact {
            id: id.into(),
            kind: FactKind::Memory,
            text: text.into(),
            session_id: session_id.into(),
        })
    }

    pub fn remember_user(
        &mut self,
        id: impl Into<String>,
        session_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<(), SoulError> {
        self.remember(MemoryFact {
            id: id.into(),
            kind: FactKind::User,
            text: text.into(),
            session_id: session_id.into(),
        })
    }

    pub fn read_snapshot(&self, kind: FactKind) -> Result<String, SoulError> {
        Ok(fs::read_to_string(self.snapshot_path(kind))?)
    }

    pub(crate) fn rewrite_legacy_snapshots(&self) -> Result<(), SoulError> {
        self.rewrite_snapshot(FactKind::Memory)?;
        self.rewrite_snapshot(FactKind::User)
    }

    fn rewrite_snapshot(&self, kind: FactKind) -> Result<(), SoulError> {
        let title = match kind {
            FactKind::Memory => "# MEMORY",
            FactKind::User => "# USER",
        };
        let mut body = format!("{title}\n\n");
        for fact in self.list_kind(kind)? {
            body.push_str("- ");
            body.push_str(&fact.text);
            body.push('\n');
        }
        fs::write(self.snapshot_path(kind), body)?;
        Ok(())
    }
}

fn valid_legacy_identifier(value: &str, max_chars: i64) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= usize::try_from(max_chars).unwrap_or(usize::MAX)
        && !value.chars().any(char::is_control)
}

fn projection_safe(fact: &MemoryFact) -> bool {
    valid_legacy_identifier(&fact.id, MAX_LEGACY_ID_CHARS)
        && valid_legacy_identifier(&fact.session_id, MAX_LEGACY_SESSION_CHARS)
        && !fact.text.trim().is_empty()
        && fact.text.chars().count() <= usize::try_from(MAX_LEGACY_TEXT_CHARS).unwrap_or(usize::MAX)
        && !contains_sensitive_material(&fact.text)
}
