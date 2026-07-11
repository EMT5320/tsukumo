//! Rebuildable Chronicle JSONL, state Markdown, and state FTS projections.

use crate::chronicle::ChronicleQuery;
use crate::state_model::{StateKind, StateRecord, StateStatus};
use crate::state_repository::{list_states, load_state};
use crate::storage::{current_timestamp, SoulError, SoulStore};
use rusqlite::params;
use std::fs;
use std::path::{Path, PathBuf};
use tsukumo_kernel::StateId;

/// Paths written by one projection rebuild.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPaths {
    pub chronicle_jsonl: PathBuf,
    pub state_markdown: PathBuf,
    pub memory_markdown: PathBuf,
    pub user_markdown: PathBuf,
}

impl SoulStore {
    /// Rebuilds all portable C1 projections under the store data directory.
    pub fn rebuild_exports(&mut self) -> Result<ExportPaths, SoulError> {
        let root = self.data_dir.clone();
        self.rebuild_exports_at(root)
    }

    /// Rebuilds projections at an explicit root, useful for controlled exports.
    pub fn rebuild_exports_at(&mut self, root: impl AsRef<Path>) -> Result<ExportPaths, SoulError> {
        let root = root.as_ref();
        fs::create_dir_all(root)?;

        let events = self.replay_events(ChronicleQuery::default().limited_to(usize::MAX))?;
        let states = list_states(&self.conn, None)?;
        let as_of = current_timestamp()?;
        rebuild_state_fts(&mut self.conn, &states, as_of)?;

        let paths = ExportPaths {
            chronicle_jsonl: root.join("chronicle.jsonl"),
            state_markdown: root.join("STATE.md"),
            memory_markdown: root.join("MEMORY.md"),
            user_markdown: root.join("USER.md"),
        };
        let mut chronicle_body = String::new();
        for persisted in events {
            chronicle_body.push_str(&serde_json::to_string(&persisted.event)?);
            chronicle_body.push('\n');
        }
        fs::write(&paths.chronicle_jsonl, chronicle_body)?;
        fs::write(&paths.state_markdown, render_state_markdown(&states, as_of))?;
        fs::write(
            &paths.memory_markdown,
            render_compatibility_markdown(&states, CompatibilityView::Memory, as_of),
        )?;
        fs::write(
            &paths.user_markdown,
            render_compatibility_markdown(&states, CompatibilityView::User, as_of),
        )?;

        Ok(paths)
    }

    /// Searches the rebuildable active-state FTS projection.
    pub fn search_states(
        &mut self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<StateRecord>, SoulError> {
        let limit = i64::try_from(limit.max(1)).unwrap_or(i64::MAX);
        let query = query.trim();
        let as_of = current_timestamp()?;
        let states = list_states(&self.conn, None)?;
        rebuild_state_fts(&mut self.conn, &states, as_of)?;
        if query.is_empty() {
            return Ok(list_states(&self.conn, Some(as_of))?
                .into_iter()
                .take(usize::try_from(limit).unwrap_or(usize::MAX))
                .collect());
        }

        let mut statement = self.conn.prepare(
            "SELECT state_id FROM state_fts
             WHERE state_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![fts_query(query), limit], |row| {
            row.get::<_, String>(0)
        })?;
        let mut states = Vec::new();
        for state_id in rows {
            if let Some(record) = load_state(&self.conn, &StateId::new(state_id?))? {
                if record.is_active_at(as_of) {
                    states.push(record);
                }
            }
        }
        Ok(states)
    }
}

fn rebuild_state_fts(
    conn: &mut rusqlite::Connection,
    states: &[StateRecord],
    as_of: tsukumo_kernel::Timestamp,
) -> Result<(), SoulError> {
    let transaction = conn.transaction()?;
    transaction.execute("DELETE FROM state_fts", [])?;
    for state in states.iter().filter(|state| state.is_active_at(as_of)) {
        transaction.execute(
            "INSERT INTO state_fts (content, state_id, state_key)
             VALUES (?1, ?2, ?3)",
            params![
                state.content.as_str(),
                state.state_id.as_str(),
                state.state_key.as_str()
            ],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

fn render_state_markdown(states: &[StateRecord], as_of: tsukumo_kernel::Timestamp) -> String {
    let mut body = String::from("# STATE\n\n");
    for state in states {
        body.push_str(&format!(
            "- [{}] {} v{} ({}): {}\n",
            status_label(state, as_of),
            state.state_key,
            state.version,
            state.state_id,
            state.content
        ));
        if !state.evidence_refs.is_empty() {
            body.push_str("  - evidence: ");
            body.push_str(
                &state
                    .evidence_refs
                    .iter()
                    .map(|event_id| event_id.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            body.push('\n');
        }
    }
    body
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CompatibilityView {
    Memory,
    User,
}

fn render_compatibility_markdown(
    states: &[StateRecord],
    view: CompatibilityView,
    as_of: tsukumo_kernel::Timestamp,
) -> String {
    let title = match view {
        CompatibilityView::Memory => "# MEMORY\n\n",
        CompatibilityView::User => "# USER\n\n",
    };
    let mut body = String::from(title);
    for state in states
        .iter()
        .filter(|state| state.is_active_at(as_of) && compatibility_view(state) == view)
    {
        body.push_str("- ");
        body.push_str(state.content.as_str());
        body.push('\n');
    }
    body
}

fn compatibility_view(state: &StateRecord) -> CompatibilityView {
    if state.state_key.as_str().starts_with("legacy.user.") {
        return CompatibilityView::User;
    }
    if state.state_key.as_str().starts_with("legacy.memory.") {
        return CompatibilityView::Memory;
    }
    match state.kind {
        StateKind::Preference | StateKind::Constraint | StateKind::Procedure => {
            CompatibilityView::User
        }
        StateKind::Fact | StateKind::Milestone => CompatibilityView::Memory,
    }
}

fn status_label(state: &StateRecord, as_of: tsukumo_kernel::Timestamp) -> &'static str {
    if state.status == StateStatus::Active && !state.is_active_at(as_of) {
        return "expired";
    }
    match state.status {
        StateStatus::Active => "active",
        StateStatus::Superseded => "superseded",
        StateStatus::Revoked => "revoked",
    }
}

fn fts_query(raw: &str) -> String {
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
