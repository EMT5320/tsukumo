//! Capacity-capped brief compiler over canonical state with legacy fallback.
//!
//! Canonical SQLite state is preferred whenever present. The isolated A1 facts
//! table is read only while a store has not completed explicit legacy import.

use crate::state_model::{StateKind, StateRecord};
use crate::storage::{SoulError, SoulStore};
use crate::store::{FactKind, MemoryFact};

pub const DEFAULT_BRIEF_CHAR_CAP: usize = 800;
pub const DEFAULT_TOP_K: usize = 5;

#[derive(Debug, Clone)]
pub struct BriefOptions {
    pub char_cap: usize,
    pub top_k: usize,
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

    /// Compiles canonical active state, falling back to isolated legacy facts.
    pub fn compile(&self, store: &SoulStore) -> Result<String, SoulError> {
        let canonical = store.list_active_states()?;
        let mut items = canonical_items(canonical, &self.options);
        if !store.legacy_import_completed()? && items.len() < self.options.top_k.max(1) {
            for item in legacy_items(store, &self.options)? {
                if items.iter().all(|existing| existing.text != item.text) {
                    items.push(item);
                }
                if items.len() >= self.options.top_k.max(1) {
                    break;
                }
            }
        }
        Ok(render_brief(&items, self.options.char_cap))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BriefSection {
    User,
    Memory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BriefItem {
    section: BriefSection,
    text: String,
}

fn canonical_items(states: Vec<StateRecord>, options: &BriefOptions) -> Vec<BriefItem> {
    let query = options.query.trim().to_lowercase();
    states
        .into_iter()
        .filter(|state| {
            query.is_empty()
                || state.content.as_str().to_lowercase().contains(&query)
                || state.state_key.as_str().to_lowercase().contains(&query)
        })
        .take(options.top_k.max(1))
        .map(|state| BriefItem {
            section: match state.kind {
                StateKind::Preference | StateKind::Constraint | StateKind::Procedure => {
                    BriefSection::User
                }
                StateKind::Fact | StateKind::Milestone => BriefSection::Memory,
            },
            text: state.content.as_str().to_owned(),
        })
        .collect()
}

fn legacy_items(store: &SoulStore, options: &BriefOptions) -> Result<Vec<BriefItem>, SoulError> {
    let mut selected = store.recall(&options.query, options.top_k)?;
    if !selected.iter().any(|fact| fact.kind == FactKind::User) {
        if let Some(user) = store.list_kind(FactKind::User)?.into_iter().next() {
            selected.insert(0, user);
            selected.truncate(options.top_k.max(1));
        }
    }
    Ok(selected.into_iter().map(legacy_item).collect())
}

fn legacy_item(fact: MemoryFact) -> BriefItem {
    BriefItem {
        section: match fact.kind {
            FactKind::User => BriefSection::User,
            FactKind::Memory => BriefSection::Memory,
        },
        text: fact.text,
    }
}

fn render_brief(items: &[BriefItem], char_cap: usize) -> String {
    let mut user_lines = Vec::new();
    let mut memory_lines = Vec::new();
    let mut used = 0usize;

    for item in items {
        let line = item.text.trim();
        if line.is_empty() {
            continue;
        }
        let cost = line.chars().count() + 3;
        let line = if used + cost > char_cap {
            if used > 0 {
                break;
            }
            let available = char_cap.saturating_sub(3);
            if available == 0 {
                break;
            }
            line.chars().take(available).collect::<String>()
        } else {
            line.to_owned()
        };
        used += line.chars().count() + 3;
        match item.section {
            BriefSection::User => user_lines.push(line),
            BriefSection::Memory => memory_lines.push(line),
        }
        if used >= char_cap {
            break;
        }
    }

    let percentage = used
        .saturating_mul(100)
        .checked_div(char_cap)
        .unwrap_or(100)
        .min(100);
    let mut output = format!(
        "[{percentage}% — {used}/{char_cap}]\nPriority: if this conflicts with project rules, project rules win.\n"
    );
    append_section(&mut output, "USER", &user_lines);
    append_section(&mut output, "MEMORY", &memory_lines);
    output
}

fn append_section(output: &mut String, title: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    output.push_str("## ");
    output.push_str(title);
    output.push('\n');
    for line in lines {
        output.push_str("- ");
        output.push_str(line);
        output.push('\n');
    }
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
