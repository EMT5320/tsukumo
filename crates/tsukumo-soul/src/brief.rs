//! Capacity-capped brief compiler (pull ≫ push).
//!
//! Default cap is character-based for the probe — not a full tokenizer.
//! Adapters must not push the entire chronicle into every delegation.

use crate::store::{FactKind, MemoryFact, SoulError, SoulStore};

/// Default character budget for the compiled brief body (excluding header).
///
/// Calibrated small for the mid-early probe: enough for a handful of facts,
/// far below dumping a full MEMORY.md. Raise later when token accounting lands.
pub const DEFAULT_BRIEF_CHAR_CAP: usize = 800;

/// Default top-k facts selected by recall relevance (plus freeze-snapshot seeds).
pub const DEFAULT_TOP_K: usize = 5;

/// Options for [`BriefCompiler::compile`].
#[derive(Debug, Clone)]
pub struct BriefOptions {
    /// Max characters in the brief body (facts section).
    pub char_cap: usize,
    /// Max facts from relevance recall.
    pub top_k: usize,
    /// Optional free-text query for FTS ranking. Empty → recent facts only.
    pub query: String,
}

impl Default for BriefOptions {
    fn default() -> Self {
        Self {
            char_cap: DEFAULT_BRIEF_CHAR_CAP,
            top_k: DEFAULT_TOP_K,
            query: String::new(),
        }
    }
}

impl BriefOptions {
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }
}

/// Compiles a capacity-capped relationship brief from the soul store.
#[derive(Debug, Default)]
pub struct BriefCompiler {
    options: BriefOptions,
}

impl BriefCompiler {
    pub fn new(options: BriefOptions) -> Self {
        Self { options }
    }

    pub fn with_defaults() -> Self {
        Self::default()
    }

    pub fn options(&self) -> &BriefOptions {
        &self.options
    }

    /// Compile a fresh brief for this delegation.
    ///
    /// Format (no anime personality words):
    /// ```text
    /// [brief N% — used/cap]
    /// Priority: if this conflicts with project rules, project rules win.
    /// ## USER
    /// - ...
    /// ## MEMORY
    /// - ...
    /// ```
    pub fn compile(&self, store: &SoulStore) -> Result<String, SoulError> {
        let mut selected = store.recall(&self.options.query, self.options.top_k)?;

        // Prefer at least one USER fact when present (owner model anchor).
        if !selected.iter().any(|f| f.kind == FactKind::User) {
            if let Some(u) = store.list_kind(FactKind::User)?.into_iter().next() {
                selected.insert(0, u);
                selected.truncate(self.options.top_k.max(1));
            }
        }

        Ok(render_brief(&selected, self.options.char_cap))
    }
}

fn render_brief(facts: &[MemoryFact], char_cap: usize) -> String {
    let mut user_lines: Vec<String> = Vec::new();
    let mut memory_lines: Vec<String> = Vec::new();
    let mut used = 0usize;

    for fact in facts {
        let line = fact.text.trim();
        if line.is_empty() {
            continue;
        }
        let cost = line.chars().count() + 3; // "- " + "\n"
        let line_owned = if used + cost > char_cap {
            if used > 0 {
                break;
            }
            // Single oversized first line: truncate to remaining budget.
            let take = char_cap.saturating_sub(3);
            if take == 0 {
                break;
            }
            line.chars().take(take).collect::<String>()
        } else {
            line.to_string()
        };
        let actual_cost = line_owned.chars().count() + 3;
        used += actual_cost;
        match fact.kind {
            FactKind::User => user_lines.push(line_owned),
            FactKind::Memory => memory_lines.push(line_owned),
        }
        if used >= char_cap {
            break;
        }
    }

    let pct = if char_cap == 0 {
        100
    } else {
        ((used * 100) / char_cap).min(100)
    };

    let mut out = format!(
        "[{pct}% — {used}/{char_cap}]\nPriority: if this conflicts with project rules, project rules win.\n"
    );
    if !user_lines.is_empty() {
        out.push_str("## USER\n");
        for line in &user_lines {
            out.push_str("- ");
            out.push_str(line);
            out.push('\n');
        }
    }
    if !memory_lines.is_empty() {
        out.push_str("## MEMORY\n");
        for line in &memory_lines {
            out.push_str("- ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn respects_char_cap() {
        let dir = tempdir().unwrap();
        let mut store = SoulStore::open(dir.path()).unwrap();
        for i in 0..20 {
            store
                .remember_memory(
                    format!("m{i}"),
                    "s1",
                    format!("Fact number {i} with some padding text for budget"),
                )
                .unwrap();
        }
        let compiler = BriefCompiler::new(BriefOptions {
            char_cap: 120,
            top_k: 20,
            query: String::new(),
        });
        let brief = compiler.compile(&store).unwrap();
        assert!(brief.contains("["));
        assert!(brief.contains("/120]"));
        assert!(!brief.contains("傲娇"));
    }

    #[test]
    fn top_k_limits_facts() {
        let dir = tempdir().unwrap();
        let mut store = SoulStore::open(dir.path()).unwrap();
        store
            .remember_user("u1", "s1", "Owner likes concise answers")
            .unwrap();
        for i in 0..10 {
            store
                .remember_memory(format!("m{i}"), "s1", format!("memory item {i}"))
                .unwrap();
        }
        let compiler = BriefCompiler::new(BriefOptions {
            char_cap: DEFAULT_BRIEF_CHAR_CAP,
            top_k: 2,
            query: "memory".into(),
        });
        let brief = compiler.compile(&store).unwrap();
        let bullet_count = brief.lines().filter(|l| l.starts_with("- ")).count();
        assert!(bullet_count <= 3); // top_k + possible USER seed
    }
}
