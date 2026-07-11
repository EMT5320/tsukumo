//! Bounded legacy recall and raw migration queries.

use super::{
    projection_safe, FactKind, MemoryFact, MAX_LEGACY_FACTS, MAX_LEGACY_ID_CHARS,
    MAX_LEGACY_KIND_CHARS, MAX_LEGACY_SESSION_CHARS, MAX_LEGACY_TEXT_CHARS, MAX_TOTAL_LEGACY_CHARS,
};
use crate::storage::{SoulError, SoulStore};
use rusqlite::params;

impl SoulStore {
    /// Recalls a bounded set from the isolated legacy facts index.
    pub fn recall(&self, query: &str, limit: usize) -> Result<Vec<MemoryFact>, SoulError> {
        let requested = i64::try_from(limit.max(1)).unwrap_or(i64::MAX);
        let limit = requested.min(MAX_LEGACY_FACTS);
        let query = query.trim();
        if query.is_empty() {
            let mut statement = self.conn.prepare(
                "SELECT id, kind, text, session_id
                 FROM facts
                 WHERE length(id) <= ?1 AND length(kind) <= ?2
                   AND length(text) <= ?3 AND length(session_id) <= ?4
                 ORDER BY created_at DESC LIMIT ?5",
            )?;
            return Self::collect_projection_facts(
                &mut statement,
                params![
                    MAX_LEGACY_ID_CHARS,
                    MAX_LEGACY_KIND_CHARS,
                    MAX_LEGACY_TEXT_CHARS,
                    MAX_LEGACY_SESSION_CHARS,
                    limit
                ],
            );
        }

        let mut statement = self.conn.prepare(
            "SELECT f.id, f.kind, f.text, f.session_id
             FROM facts_fts
             JOIN facts f ON f.rowid = facts_fts.rowid
             WHERE facts_fts MATCH ?1
               AND length(f.id) <= ?2 AND length(f.kind) <= ?3
               AND length(f.text) <= ?4 AND length(f.session_id) <= ?5
             ORDER BY rank
             LIMIT ?6",
        )?;
        let hits = Self::collect_projection_facts(
            &mut statement,
            params![
                fts5_query(query),
                MAX_LEGACY_ID_CHARS,
                MAX_LEGACY_KIND_CHARS,
                MAX_LEGACY_TEXT_CHARS,
                MAX_LEGACY_SESSION_CHARS,
                limit
            ],
        )?;
        if !hits.is_empty() {
            return Ok(hits);
        }

        let pattern = format!("%{query}%");
        let mut statement = self.conn.prepare(
            "SELECT id, kind, text, session_id
             FROM facts
             WHERE text LIKE ?1 COLLATE NOCASE
               AND length(id) <= ?2 AND length(kind) <= ?3
               AND length(text) <= ?4 AND length(session_id) <= ?5
             ORDER BY created_at DESC
             LIMIT ?6",
        )?;
        Self::collect_projection_facts(
            &mut statement,
            params![
                pattern,
                MAX_LEGACY_ID_CHARS,
                MAX_LEGACY_KIND_CHARS,
                MAX_LEGACY_TEXT_CHARS,
                MAX_LEGACY_SESSION_CHARS,
                limit
            ],
        )
    }

    pub fn list_kind(&self, kind: FactKind) -> Result<Vec<MemoryFact>, SoulError> {
        let mut statement = self.conn.prepare(
            "SELECT id, kind, text, session_id
             FROM facts
             WHERE kind = ?1
               AND length(id) <= ?2 AND length(kind) <= ?3
               AND length(text) <= ?4 AND length(session_id) <= ?5
             ORDER BY created_at ASC, id ASC
             LIMIT ?6",
        )?;
        Self::collect_projection_facts(
            &mut statement,
            params![
                kind.as_str(),
                MAX_LEGACY_ID_CHARS,
                MAX_LEGACY_KIND_CHARS,
                MAX_LEGACY_TEXT_CHARS,
                MAX_LEGACY_SESSION_CHARS,
                MAX_LEGACY_FACTS
            ],
        )
    }

    pub(crate) fn list_legacy_facts(&self) -> Result<Vec<MemoryFact>, SoulError> {
        let (count, max_id, max_kind, max_text, max_session, total) = self.conn.query_row(
            "SELECT COUNT(*), COALESCE(MAX(length(id)), 0),
                    COALESCE(MAX(length(kind)), 0), COALESCE(MAX(length(text)), 0),
                    COALESCE(MAX(length(session_id)), 0),
                    COALESCE(SUM(
                        length(id) + length(kind) + length(text) + length(session_id)
                    ), 0)
             FROM facts",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                ))
            },
        )?;
        if count > MAX_LEGACY_FACTS
            || max_id > MAX_LEGACY_ID_CHARS
            || max_kind > MAX_LEGACY_KIND_CHARS
            || max_text > MAX_LEGACY_TEXT_CHARS
            || max_session > MAX_LEGACY_SESSION_CHARS
            || total > MAX_TOTAL_LEGACY_CHARS
        {
            return Err(SoulError::LegacyBudgetExceeded);
        }
        let mut statement = self.conn.prepare(
            "SELECT id, kind, text, session_id
             FROM facts ORDER BY created_at ASC, id ASC LIMIT ?1",
        )?;
        Self::collect_facts(&mut statement, params![MAX_LEGACY_FACTS])
    }

    fn collect_projection_facts(
        statement: &mut rusqlite::Statement<'_>,
        parameters: impl rusqlite::Params,
    ) -> Result<Vec<MemoryFact>, SoulError> {
        Ok(Self::collect_facts(statement, parameters)?
            .into_iter()
            .filter(projection_safe)
            .collect())
    }

    fn collect_facts(
        statement: &mut rusqlite::Statement<'_>,
        parameters: impl rusqlite::Params,
    ) -> Result<Vec<MemoryFact>, SoulError> {
        let rows = statement.query_map(parameters, |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        let mut facts = Vec::new();
        for row in rows {
            let (id, kind_value, text, session_id) = row?;
            let kind = FactKind::from_str(&kind_value)
                .ok_or_else(|| SoulError::InvalidKind(kind_value.clone()))?;
            facts.push(MemoryFact {
                id,
                kind,
                text,
                session_id,
            });
        }
        Ok(facts)
    }
}

fn fts5_query(raw: &str) -> String {
    let tokens = raw
        .split(|character: char| !character.is_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{}\"*", token.replace('"', "\"\"")))
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        format!("\"{}\"", raw.replace('"', "\"\""))
    } else {
        tokens.join(" ")
    }
}
