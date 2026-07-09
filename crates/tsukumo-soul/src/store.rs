//! Canonical soul store: MEMORY.md / USER.md snapshots + sqlite FTS5 index.
//!
//! File snapshots are the human-readable source of truth under `data_dir`.
//! The FTS index is derived for cross-session recall (pull, not push).

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Kind of durable fact. Maps to Hermes-style MEMORY / USER freeze snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactKind {
    /// Guild archive / shared memory (`MEMORY.md`).
    Memory,
    /// Owner model / preferences (`USER.md`).
    User,
}

impl FactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            FactKind::Memory => "memory",
            FactKind::User => "user",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "memory" => Some(FactKind::Memory),
            "user" => Some(FactKind::User),
            _ => None,
        }
    }

    fn snapshot_file(self) -> &'static str {
        match self {
            FactKind::Memory => "MEMORY.md",
            FactKind::User => "USER.md",
        }
    }
}

/// One recallable fact line (canonical IR unit for the probe).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryFact {
    pub id: String,
    pub kind: FactKind,
    /// Plain factual sentence — no anime personality words.
    pub text: String,
    /// Session that wrote this fact (cross-session recall key).
    pub session_id: String,
}

#[derive(Debug, Error)]
pub enum SoulError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid fact kind: {0}")]
    InvalidKind(String),
    #[error("empty fact text")]
    EmptyText,
}

/// On-disk soul store rooted at `data_dir`.
///
/// Layout:
/// ```text
/// data_dir/
///   MEMORY.md      # freeze snapshot (memory facts)
///   USER.md        # freeze snapshot (user-model facts)
///   skills/        # empty socket (see skills module)
///   soul.db        # sqlite + FTS5 recall index
///   inject_trace.jsonl  # optional inject/recall stub log
/// ```
pub struct SoulStore {
    data_dir: PathBuf,
    conn: Connection,
}

impl SoulStore {
    /// Open or create a store under `data_dir`.
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self, SoulError> {
        let data_dir = data_dir.as_ref().to_path_buf();
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("skills"))?;

        let db_path = data_dir.join("soul.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS facts (
                id TEXT PRIMARY KEY NOT NULL,
                kind TEXT NOT NULL,
                text TEXT NOT NULL,
                session_id TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE VIRTUAL TABLE IF NOT EXISTS facts_fts USING fts5(
                text,
                id UNINDEXED,
                kind UNINDEXED,
                session_id UNINDEXED,
                content='facts',
                content_rowid='rowid'
            );
            CREATE TRIGGER IF NOT EXISTS facts_ai AFTER INSERT ON facts BEGIN
                INSERT INTO facts_fts(rowid, text, id, kind, session_id)
                VALUES (new.rowid, new.text, new.id, new.kind, new.session_id);
            END;
            CREATE TRIGGER IF NOT EXISTS facts_ad AFTER DELETE ON facts BEGIN
                INSERT INTO facts_fts(facts_fts, rowid, text, id, kind, session_id)
                VALUES ('delete', old.rowid, old.text, old.id, old.kind, old.session_id);
            END;
            "#,
        )?;

        // Ensure snapshot files exist (empty headers ok).
        for kind in [FactKind::Memory, FactKind::User] {
            let path = data_dir.join(kind.snapshot_file());
            if !path.exists() {
                let header = match kind {
                    FactKind::Memory => "# MEMORY\n\n",
                    FactKind::User => "# USER\n\n",
                };
                fs::write(&path, header)?;
            }
        }

        Ok(Self { data_dir, conn })
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn skills_dir(&self) -> PathBuf {
        self.data_dir.join("skills")
    }

    pub fn snapshot_path(&self, kind: FactKind) -> PathBuf {
        self.data_dir.join(kind.snapshot_file())
    }

    /// Append a fact: updates FTS index and rewrites the freeze snapshot.
    pub fn remember(&mut self, fact: MemoryFact) -> Result<(), SoulError> {
        let text = fact.text.trim();
        if text.is_empty() {
            return Err(SoulError::EmptyText);
        }
        self.conn.execute(
            "INSERT OR REPLACE INTO facts (id, kind, text, session_id) VALUES (?1, ?2, ?3, ?4)",
            params![fact.id, fact.kind.as_str(), text, fact.session_id],
        )?;
        self.rewrite_snapshot(fact.kind)?;
        Ok(())
    }

    /// Convenience: remember a memory-kind fact.
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

    /// Convenience: remember a user-model fact.
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

    /// FTS5 recall. Empty query returns recent facts (newest first), still capped by `limit`.
    ///
    /// If FTS returns no rows (e.g. CJK tokenizer edge), falls back to case-insensitive
    /// substring `LIKE` — still capped; never dumps the full chronicle.
    pub fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryFact>, SoulError> {
        let limit = limit.max(1);
        let query = query.trim();
        if query.is_empty() {
            let mut stmt = self.conn.prepare(
                "SELECT id, kind, text, session_id FROM facts ORDER BY created_at DESC LIMIT ?1",
            )?;
            return Self::collect_facts(&mut stmt, params![limit as i64]);
        }

        let fts_query = fts5_query(query);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT f.id, f.kind, f.text, f.session_id
            FROM facts_fts
            JOIN facts f ON f.rowid = facts_fts.rowid
            WHERE facts_fts MATCH ?1
            ORDER BY rank
            LIMIT ?2
            "#,
        )?;
        let hits = Self::collect_facts(&mut stmt, params![fts_query, limit as i64])?;
        if !hits.is_empty() {
            return Ok(hits);
        }

        // Substring fallback (no LLM curation — pure local scan, still limited).
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, text, session_id FROM facts WHERE text LIKE ?1 COLLATE NOCASE ORDER BY created_at DESC LIMIT ?2",
        )?;
        Self::collect_facts(&mut stmt, params![pattern, limit as i64])
    }

    /// All facts of a kind, oldest first (for snapshot / brief freeze).
    pub fn list_kind(&self, kind: FactKind) -> Result<Vec<MemoryFact>, SoulError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, text, session_id FROM facts WHERE kind = ?1 ORDER BY created_at ASC",
        )?;
        Self::collect_facts(&mut stmt, params![kind.as_str()])
    }

    /// Read freeze snapshot file as UTF-8 text.
    pub fn read_snapshot(&self, kind: FactKind) -> Result<String, SoulError> {
        Ok(fs::read_to_string(self.snapshot_path(kind))?)
    }

    fn rewrite_snapshot(&self, kind: FactKind) -> Result<(), SoulError> {
        let facts = self.list_kind(kind)?;
        let title = match kind {
            FactKind::Memory => "# MEMORY",
            FactKind::User => "# USER",
        };
        let mut body = String::from(title);
        body.push_str("\n\n");
        for f in &facts {
            body.push_str("- ");
            body.push_str(&f.text);
            body.push('\n');
        }
        fs::write(self.snapshot_path(kind), body)?;
        Ok(())
    }

    fn collect_facts(
        stmt: &mut rusqlite::Statement<'_>,
        params: impl rusqlite::Params,
    ) -> Result<Vec<MemoryFact>, SoulError> {
        let rows = stmt.query_map(params, |row| {
            let kind_s: String = row.get(1)?;
            Ok((
                row.get::<_, String>(0)?,
                kind_s,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, kind_s, text, session_id) = row?;
            let kind = FactKind::from_str(&kind_s)
                .ok_or_else(|| SoulError::InvalidKind(kind_s.clone()))?;
            out.push(MemoryFact {
                id,
                kind,
                text,
                session_id,
            });
        }
        Ok(out)
    }
}

/// Build a conservative FTS5 MATCH string from free text.
fn fts5_query(raw: &str) -> String {
    let tokens: Vec<&str> = raw
        .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() {
        // Fall back to phrase quote of the whole string (escaped quotes).
        let escaped = raw.replace('"', "\"\"");
        return format!("\"{escaped}\"");
    }
    // AND of tokens; each token gets a prefix wildcard for soft match.
    tokens
        .iter()
        .map(|t| format!("{t}*"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn remember_writes_snapshot_and_fts() {
        let dir = tempdir().unwrap();
        let mut store = SoulStore::open(dir.path()).unwrap();
        store
            .remember_user("u1", "session-a", "Owner prefers gnu toolchain on Windows")
            .unwrap();
        let snap = store.read_snapshot(FactKind::User).unwrap();
        assert!(snap.contains("gnu toolchain"));
        let hits = store.recall("gnu toolchain", 5).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "u1");
    }

    #[test]
    fn skills_dir_exists() {
        let dir = tempdir().unwrap();
        let store = SoulStore::open(dir.path()).unwrap();
        assert!(store.skills_dir().is_dir());
    }
}
