//! Prompt / briefing assembly hook for Phase R (relationship probe).
//!
//! A1 owns the **assembly point** (`BriefingSource` + [`assemble_prompt`]).
//! Phase R (`tsukumo-soul`) owns briefing **content**:
//! `BriefCompiler::compile` → pass `Some(brief)` into [`assemble_prompt`].
//! Do not depend on soul from this crate — the host wires the two.

/// Context available when assembling a runtime prompt (thin on purpose).
#[derive(Debug, Clone, Default)]
pub struct PromptAssemblyContext {
    /// Soft executor identity (not a vendor string).
    pub executor_id: Option<String>,
    /// Quest / session label for later tracing.
    pub quest_id: Option<String>,
}

/// Source of capacity-capped briefing text injected at prompt assembly.
///
/// Real content: `tsukumo_soul::BriefCompiler` (host implements this trait
/// or calls `compile` and passes the string into [`assemble_prompt`]).
pub trait BriefingSource {
    fn briefing_for(&self, ctx: &PromptAssemblyContext) -> Option<String>;
}

/// Always returns `None` — placeholder until Phase R.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullBriefing;

impl BriefingSource for NullBriefing {
    fn briefing_for(&self, _ctx: &PromptAssemblyContext) -> Option<String> {
        None
    }
}

/// Alias kept for call-site clarity in demos / tests.
pub type StubBriefing = NullBriefing;

/// Merge an optional briefing into a base user/system prompt body.
///
/// Empty briefing → base unchanged. Non-empty → append a marked block so
/// later §8.3 managed-region discipline can find it.
pub fn assemble_prompt(base: &str, briefing: Option<&str>) -> String {
    match briefing.map(str::trim).filter(|s| !s.is_empty()) {
        None => base.to_string(),
        Some(brief) => format!(
            "{base}\n\n<!-- tsukumo-briefing -->\n{brief}\n<!-- /tsukumo-briefing -->"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a1_null_briefing_leaves_base() {
        let src = NullBriefing;
        let ctx = PromptAssemblyContext {
            executor_id: Some("gina".into()),
            quest_id: Some("q1".into()),
        };
        assert!(src.briefing_for(&ctx).is_none());
        assert_eq!(assemble_prompt("do the thing", None), "do the thing");
    }

    #[test]
    fn a1_fixture_briefing_marks_block() {
        let out = assemble_prompt("task", Some("user likes tea"));
        assert!(out.contains("<!-- tsukumo-briefing -->"));
        assert!(out.contains("user likes tea"));
    }
}
