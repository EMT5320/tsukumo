//! Tsukumo relationship persistence and recall layer.
//!
//! SQLite is the durable authority for Chronicle and canonical state. Human
//! readable files and FTS indexes are rebuildable projections.

pub mod brief;
pub mod chronicle;
mod comparison;
pub mod export;
pub mod extract;
mod handoff;
mod handoff_error;
mod handoff_loop;
mod handoff_model;
mod handoff_repository;
mod handoff_shape;
pub mod inject;
pub mod legacy;
mod migrations;
mod projection;
mod projection_budget;
mod projection_error;
mod projection_model;
mod projection_render;
mod projection_repository;
mod projection_selection;
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
pub use comparison::{compare_projection_receipts, ProjectionComparison};
pub use export::ExportPaths;
pub use extract::{
    extract_non_blocking, ExtractError, ExtractionAttempt, ExtractionContext,
    RecordedStateExtractor, RuleStateExtractor, StateExtractor,
};
pub use handoff_error::HandoffError;
pub use handoff_loop::{OpenLoop, OpenLoopId, OpenLoopOutcome, OpenLoopTransition};
pub use handoff_model::{
    ArtifactReference, CheckpointTrigger, CheckpointWriteRequest, Decision, HandoffCheckpoint,
    NextAction, ProgressItem, ProgressStatus, StateRef,
};
pub use inject::{
    assemble_delegation_prompt, assemble_with_trace, compile_briefing, DefaultPromptAssembler,
    PromptAssembler,
};
pub use legacy::{LegacyImportContext, LegacyImportReport, LegacyImportSkip};
pub use projection_error::ProjectionError;
pub use projection_model::{
    BudgetUnit, ContentDigest, DigestAlgorithm, PreparedProjection, ProjectionBudgetUsage,
    ProjectionOmission, ProjectionOmissionReason, ProjectionReceipt, ProjectionRequest,
    ProjectionSection, ProjectionSectionDigest, ProjectionTarget, ProjectionWriteRequest,
    RedactionRecord, PROJECTION_VERSION, RENDERER_VERSION,
};
pub use skills::{Skill, SkillSocket, SkillStub};
pub use state_model::{
    EvidenceStrength, ExtractionProvenance, OperatingSystem, StateApplicability, StateDraft,
    StateKey, StateKind, StateRecord, StateScope, StateStatus, StateSubject, StateTransition,
    StateValidationError, StateWriteOutcome, StateWriteRequest,
};
pub use storage::{SoulError, SoulStore};
pub use store::{FactKind, MemoryFact};
pub use trace::{TraceEvent, TraceLog};
