//! Prompt assembly hook for A1 adapters / session owners.
//!
//! ## Coordination with `tsukumo-adapters`
//! Adapters expose `BriefingSource` + `assemble_prompt` (marked
//! `<!-- tsukumo-briefing -->` block). Phase R fills the briefing **content**:
//!
//! 1. `BriefCompiler::compile(&store)` → capacity-capped brief string
//! 2. Pass `Some(brief.as_str())` into adapters' `assemble_prompt(base, briefing)`
//!
//! This module also offers a self-contained assembler for hosts that do not
//! depend on adapters yet. No anime personality words — facts only.

use crate::brief::{BriefCompiler, BriefOptions};
use crate::store::{SoulError, SoulStore};
use crate::trace::{TraceEvent, TraceLog};

/// Trait for assembling a runtime delegation prompt (soul-side).
///
/// Prefer adapters' `assemble_prompt` when the drive crate is on the path;
/// use this when the host only depends on `tsukumo-soul`.
pub trait PromptAssembler {
    /// Combine `brief` + `user_goal` into a single prompt string for the runtime.
    fn assemble(&self, brief: &str, user_goal: &str) -> String;
}

/// Default assembler: brief block + goal.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultPromptAssembler;

impl PromptAssembler for DefaultPromptAssembler {
    fn assemble(&self, brief: &str, user_goal: &str) -> String {
        assemble_delegation_prompt(brief, user_goal)
    }
}

/// Free function form of the inject hook.
///
/// Output shape:
/// ```text
/// ## Relationship brief
/// <brief>
///
/// ## Goal
/// <user_goal>
/// ```
pub fn assemble_delegation_prompt(brief: &str, user_goal: &str) -> String {
    let brief = brief.trim();
    let goal = user_goal.trim();
    let mut out = String::new();
    if !brief.is_empty() {
        out.push_str("## Relationship brief\n");
        out.push_str(brief);
        if !brief.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str("## Goal\n");
    out.push_str(goal);
    if !goal.is_empty() && !goal.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Compile a brief from the store (helper for A1 `BriefingSource` impls).
pub fn compile_briefing(
    store: &SoulStore,
    options: BriefOptions,
) -> Result<String, SoulError> {
    BriefCompiler::new(options).compile(store)
}

/// Assemble and optionally append an inject trace line (R6 stub).
pub fn assemble_with_trace(
    brief: &str,
    user_goal: &str,
    quest_id: Option<&str>,
    trace: Option<&mut TraceLog>,
) -> String {
    let prompt = assemble_delegation_prompt(brief, user_goal);
    if let Some(log) = trace {
        let _ = log.append(TraceEvent::Inject {
            quest_id: quest_id.map(|s| s.to_string()),
            brief_chars: brief.chars().count(),
            goal_chars: user_goal.chars().count(),
        });
    }
    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_without_personality_words() {
        let brief = "[10% — 20/800]\n## USER\n- Owner prefers gnu toolchain\n";
        let prompt = assemble_delegation_prompt(brief, "Fix the linker");
        assert!(prompt.contains("Relationship brief"));
        assert!(prompt.contains("gnu toolchain"));
        assert!(prompt.contains("## Goal"));
        assert!(prompt.contains("Fix the linker"));
        assert!(!prompt.contains("傲娇"));
        assert!(!prompt.contains("本小姐"));
    }

    #[test]
    fn trait_matches_free_fn() {
        let a = DefaultPromptAssembler.assemble("b", "g");
        let b = assemble_delegation_prompt("b", "g");
        assert_eq!(a, b);
    }
}
