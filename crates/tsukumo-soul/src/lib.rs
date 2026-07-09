//! Tsukumo relationship layer (mid-early probe).
//!
//! Canonical memory lives here — adapters compile briefs into prompts;
//! theater never owns the store. No dual species growth schemas.
//!
//! ## A1 coordination
//! Call [`assemble_delegation_prompt`] (or [`DefaultPromptAssembler`]) at the
//! adapter / session-owner prompt assembly point. Pass a capacity-capped
//! brief from [`BriefCompiler`] — do not push the full chronicle.

pub mod brief;
pub mod inject;
pub mod skills;
pub mod store;
pub mod trace;

pub use brief::{BriefCompiler, BriefOptions, DEFAULT_BRIEF_CHAR_CAP, DEFAULT_TOP_K};
pub use inject::{
    assemble_delegation_prompt, assemble_with_trace, compile_briefing, DefaultPromptAssembler,
    PromptAssembler,
};
pub use skills::{Skill, SkillSocket, SkillStub};
pub use store::{FactKind, MemoryFact, SoulError, SoulStore};
pub use trace::{TraceEvent, TraceLog};
