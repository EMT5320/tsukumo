//! Prompt assembly compatibility seam for the relationship probe.
//!
//! Soul owns briefing content and the future host wires it to runtime launch.
//! This crate keeps only the assembly boundary and shared typed identities.

use tsukumo_kernel::{QuestId, SpiritId};

/// Typed context available when assembling a runtime prompt.
#[derive(Debug, Clone, Default)]
pub struct PromptAssemblyContext {
    pub spirit_id: Option<SpiritId>,
    pub quest_id: Option<QuestId>,
}

/// Source of capacity-capped briefing text injected at prompt assembly.
pub trait BriefingSource {
    fn briefing_for(&self, context: &PromptAssemblyContext) -> Option<String>;
}

/// Placeholder source used until host wires a Soul brief compiler.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullBriefing;

impl BriefingSource for NullBriefing {
    fn briefing_for(&self, _context: &PromptAssemblyContext) -> Option<String> {
        None
    }
}

/// Alias retained for call-site clarity in demos and tests.
pub type StubBriefing = NullBriefing;

/// Merges an optional briefing into a base prompt body.
///
/// The marked region remains a compatibility surface until canonical C1
/// projection replaces this probe assembly path.
pub fn assemble_prompt(base: &str, briefing: Option<&str>) -> String {
    match briefing.map(str::trim).filter(|text| !text.is_empty()) {
        None => base.to_owned(),
        Some(brief) => {
            format!("{base}\n\n<!-- tsukumo-briefing -->\n{brief}\n<!-- /tsukumo-briefing -->")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_briefing_leaves_base_prompt_unchanged() {
        // Given: typed spirit and quest context with no briefing source.
        let source = NullBriefing;
        let context = PromptAssemblyContext {
            spirit_id: Some(SpiritId::new("yuka")),
            quest_id: Some(QuestId::new("quest-1")),
        };

        // When: the placeholder source and assembler run.
        let briefing = source.briefing_for(&context);
        let prompt = assemble_prompt("do the thing", briefing.as_deref());

        // Then: no empty managed region is injected.
        assert_eq!(prompt, "do the thing");
    }

    #[test]
    fn fixture_briefing_uses_explicit_marked_region() {
        // Given: one reviewed compatibility briefing.
        let briefing = "user likes tea";

        // When: it is assembled into the prompt.
        let prompt = assemble_prompt("task", Some(briefing));

        // Then: the region remains explicit and discoverable.
        assert!(prompt.contains("<!-- tsukumo-briefing -->"));
        assert!(prompt.contains(briefing));
    }
}
