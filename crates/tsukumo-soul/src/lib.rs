//! Tsukumo relationship persistence and recall layer.
//!
//! SQLite is the durable authority for Chronicle and canonical state. Human
//! readable files and FTS indexes are rebuildable projections.

pub mod brief;
pub mod chronicle;
pub mod export;
pub mod extract;
pub mod inject;
pub mod legacy;
pub mod skills;
pub mod state;
mod state_codec;
mod state_model;
mod state_repository;
mod state_scope;
mod state_validation;
mod storage;
pub mod store;
pub mod trace;

pub use brief::{BriefCompiler, BriefOptions, DEFAULT_BRIEF_CHAR_CAP, DEFAULT_TOP_K};
pub use chronicle::{AppendOutcome, ChronicleQuery, PersistedEvent};
pub use export::ExportPaths;
pub use extract::{
    extract_non_blocking, ExtractError, ExtractionAttempt, ExtractionContext,
    RecordedStateExtractor, RuleStateExtractor, StateExtractor,
};
pub use inject::{
    assemble_delegation_prompt, assemble_with_trace, compile_briefing, DefaultPromptAssembler,
    PromptAssembler,
};
pub use legacy::{LegacyImportContext, LegacyImportReport, LegacyImportSkip};
pub use skills::{Skill, SkillSocket, SkillStub};
pub use state_model::{
    EvidenceStrength, ExtractionProvenance, OperatingSystem, StateApplicability, StateDraft,
    StateKey, StateKind, StateRecord, StateScope, StateStatus, StateSubject, StateTransition,
    StateValidationError, StateWriteOutcome, StateWriteRequest,
};
pub use storage::{SoulError, SoulStore};
pub use store::{FactKind, MemoryFact};
pub use trace::{TraceEvent, TraceLog};
