//! Provider-neutral state extraction seams.
//!
//! Extractors propose StateDraft values only. They never receive a database
//! handle and cannot bypass the deterministic StateWriter validation gate.

use crate::state_model::{
    EvidenceStrength, ExtractionProvenance, StateDraft, StateKey, StateKind, StateScope,
};
use crate::state_validation::is_explicit_gnu_user_text;
use serde::Deserialize;
use thiserror::Error;
use tsukumo_kernel::{KernelEvent, KernelEventPayload, PersistedText, SensitiveText, Timestamp};

const RECORDED_SCHEMA_VERSION: u16 = 1;
const MAX_RECORDED_INPUT_BYTES: usize = 1_048_576;
const MAX_RECORDED_DRAFTS: usize = 32;
const MAX_RECORDED_CONTENT_CHARS: usize = 4_096;

/// Typed input for deterministic or structured extraction.
#[derive(Debug, Clone)]
pub struct ExtractionContext<'a> {
    pub event: &'a KernelEvent,
    pub scope: StateScope,
}

/// State proposal boundary shared by rules, live structured models, and fixtures.
pub trait StateExtractor {
    fn extract(&self, context: &ExtractionContext<'_>) -> Result<Vec<StateDraft>, ExtractError>;
}

/// Provider-neutral extraction failure.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ExtractError {
    #[error("recorded extraction {fixture} was malformed")]
    Malformed { fixture: String },
    #[error("recorded extraction {fixture} timed out")]
    Timeout { fixture: String },
    #[error("recorded extraction {fixture} was unavailable")]
    Unavailable { fixture: String },
}

/// Non-blocking host-facing result with an observable recoverable event.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractionAttempt {
    Drafts(Vec<StateDraft>),
    Skipped {
        error: ExtractError,
        event: KernelEventPayload,
    },
}

/// Runs extraction without turning model/fixture failure into primary-task failure.
pub fn extract_non_blocking(
    extractor: &dyn StateExtractor,
    context: &ExtractionContext<'_>,
) -> ExtractionAttempt {
    match extractor.extract(context) {
        Ok(drafts) => ExtractionAttempt::Drafts(drafts),
        Err(error) => {
            let message = match &error {
                ExtractError::Malformed { .. } => "state extraction skipped: malformed output",
                ExtractError::Timeout { .. } => "state extraction skipped: timeout",
                ExtractError::Unavailable { .. } => "state extraction skipped: unavailable",
            };
            ExtractionAttempt::Skipped {
                error,
                event: KernelEventPayload::Error {
                    message: PersistedText::from_reviewed(message),
                    recoverable: true,
                },
            }
        }
    }
}

/// Deterministic rules for explicit, fully testable signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct RuleStateExtractor;

impl StateExtractor for RuleStateExtractor {
    fn extract(&self, context: &ExtractionContext<'_>) -> Result<Vec<StateDraft>, ExtractError> {
        let KernelEventPayload::UserInput { content } = &context.event.payload else {
            return Ok(Vec::new());
        };
        if !is_explicit_gnu_user_text(content.as_str()) {
            return Ok(Vec::new());
        }
        let Some(workspace) = context.scope.applicability.workspace.as_ref() else {
            return Ok(Vec::new());
        };

        Ok(vec![StateDraft {
            proposed_key: StateKey::new(format!(
                "workspace.{}.rust.toolchain.windows",
                workspace.as_str()
            )),
            kind: StateKind::Constraint,
            scope: context.scope.clone(),
            content: SensitiveText::new("Use the GNU Rust toolchain on Windows"),
            claimed_strength: EvidenceStrength::Explicit,
            evidence_refs: vec![context.event.event_id.clone()],
            provenance: ExtractionProvenance::Rule {
                name: "explicit_gnu_constraint".into(),
                version: 1,
            },
            expires_at: None,
        }])
    }
}

#[derive(Debug, Clone)]
enum RecordedOutcome {
    Proposals {
        fixture: String,
        proposals: Vec<RecordedDraft>,
    },
    Error(ExtractError),
}

/// Deterministic structured-extractor fixture for regression tests.
#[derive(Debug, Clone)]
pub struct RecordedStateExtractor {
    outcome: RecordedOutcome,
}

impl RecordedStateExtractor {
    /// Parses bounded semantic proposals; trusted scope and evidence bind at extraction time.
    pub fn from_json(fixture: impl Into<String>, body: &str) -> Result<Self, ExtractError> {
        let fixture = fixture.into();
        if body.len() > MAX_RECORDED_INPUT_BYTES {
            return Err(ExtractError::Malformed { fixture });
        }
        let envelope: RecordedEnvelope =
            serde_json::from_str(body).map_err(|_| ExtractError::Malformed {
                fixture: fixture.clone(),
            })?;
        if envelope.schema_version != RECORDED_SCHEMA_VERSION
            || envelope.drafts.len() > MAX_RECORDED_DRAFTS
            || envelope.drafts.iter().any(|draft| {
                draft.content.chars().count() > MAX_RECORDED_CONTENT_CHARS
                    || draft.proposed_key.as_str().chars().count() > 256
            })
        {
            return Err(ExtractError::Malformed { fixture });
        }
        Ok(Self {
            outcome: RecordedOutcome::Proposals {
                fixture,
                proposals: envelope.drafts,
            },
        })
    }

    pub fn malformed(fixture: impl Into<String>) -> Self {
        Self {
            outcome: RecordedOutcome::Error(ExtractError::Malformed {
                fixture: fixture.into(),
            }),
        }
    }

    pub fn timeout(fixture: impl Into<String>) -> Self {
        Self {
            outcome: RecordedOutcome::Error(ExtractError::Timeout {
                fixture: fixture.into(),
            }),
        }
    }

    pub fn unavailable(fixture: impl Into<String>) -> Self {
        Self {
            outcome: RecordedOutcome::Error(ExtractError::Unavailable {
                fixture: fixture.into(),
            }),
        }
    }
}

impl StateExtractor for RecordedStateExtractor {
    fn extract(&self, context: &ExtractionContext<'_>) -> Result<Vec<StateDraft>, ExtractError> {
        match &self.outcome {
            RecordedOutcome::Proposals { fixture, proposals } => Ok(proposals
                .iter()
                .map(|proposal| StateDraft {
                    proposed_key: proposal.proposed_key.clone(),
                    kind: proposal.kind,
                    scope: context.scope.clone(),
                    content: SensitiveText::new(proposal.content.clone()),
                    claimed_strength: EvidenceStrength::Inferred,
                    evidence_refs: vec![context.event.event_id.clone()],
                    provenance: ExtractionProvenance::Recorded {
                        fixture: fixture.clone(),
                        schema_version: RECORDED_SCHEMA_VERSION,
                    },
                    expires_at: proposal.expires_at,
                })
                .collect()),
            RecordedOutcome::Error(error) => Err(error.clone()),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedEnvelope {
    schema_version: u16,
    drafts: Vec<RecordedDraft>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordedDraft {
    proposed_key: StateKey,
    kind: StateKind,
    content: String,
    expires_at: Option<Timestamp>,
}
