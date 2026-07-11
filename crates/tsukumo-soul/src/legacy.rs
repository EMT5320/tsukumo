//! Explicit, idempotent migration from the isolated A1 facts table.

use crate::state_model::{
    EvidenceStrength, ExtractionProvenance, StateDraft, StateKey, StateKind, StateScope,
    StateSubject, StateTransition, StateWriteOutcome, StateWriteRequest,
};
use crate::storage::{SoulError, SoulStore};
use crate::store::MemoryFact;
use rusqlite::OptionalExtension;
const LEGACY_IMPORTER_VERSION: i64 = 1;

use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SensitiveText, SessionId,
    SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};

/// Caller-supplied identity for newly created legacy import events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyImportContext {
    pub spirit_id: SpiritId,
    pub quest_id: QuestId,
    pub occurred_at: Timestamp,
}

/// One skipped legacy row and its observable reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyImportSkip {
    pub fact_id: String,
    pub reason: String,
}

/// Summary of one explicit legacy migration pass.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LegacyImportReport {
    pub imported: usize,
    pub unchanged: usize,
    pub skipped: Vec<LegacyImportSkip>,
}

impl SoulStore {
    /// Imports legacy facts as low-strength, unresolved Facts only.
    pub fn import_legacy_facts(
        &mut self,
        context: LegacyImportContext,
    ) -> Result<LegacyImportReport, SoulError> {
        let facts = self.list_legacy_facts()?;
        let mut report = LegacyImportReport::default();
        for fact in facts {
            match self.existing_legacy_import(&fact)? {
                ExistingLegacyImport::Matching => {
                    report.unchanged += 1;
                    continue;
                }
                ExistingLegacyImport::Conflicting(reason) => {
                    report.skipped.push(LegacyImportSkip {
                        fact_id: fact.id,
                        reason,
                    });
                    continue;
                }
                ExistingLegacyImport::Absent => {}
            }

            let request = legacy_request(&context, &fact);
            match self.apply_state(request) {
                Ok(StateWriteOutcome::Created(_)) => report.imported += 1,
                Ok(StateWriteOutcome::Unchanged(_)) => report.unchanged += 1,
                Ok(StateWriteOutcome::Superseded(_) | StateWriteOutcome::Revoked(_)) => {
                    report.skipped.push(LegacyImportSkip {
                        fact_id: fact.id,
                        reason: "unexpected legacy lifecycle outcome".into(),
                    });
                }
                Err(error @ SoulError::StateValidation(_))
                | Err(error @ SoulError::ConflictingEvent { .. })
                | Err(error @ SoulError::EventContract(_)) => {
                    report.skipped.push(LegacyImportSkip {
                        fact_id: fact.id,
                        reason: error.to_string(),
                    });
                }
                Err(error) => return Err(error),
            }
        }
        if report.skipped.is_empty() {
            self.conn.execute(
                "INSERT INTO legacy_import_runs (source_table, completed_at, importer_version)
                 VALUES ('facts', ?1, ?2)
                 ON CONFLICT(source_table) DO UPDATE SET
                    completed_at = excluded.completed_at,
                    importer_version = excluded.importer_version",
                [
                    context.occurred_at.as_unix_millis(),
                    LEGACY_IMPORTER_VERSION,
                ],
            )?;
        }
        Ok(report)
    }

    pub(crate) fn legacy_import_completed(&self) -> Result<bool, SoulError> {
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM legacy_import_runs
                 WHERE source_table = 'facts' AND importer_version = ?1",
                [LEGACY_IMPORTER_VERSION],
                |row| row.get::<_, i64>(0),
            )
            .optional()?
            .is_some())
    }

    fn existing_legacy_import(&self, fact: &MemoryFact) -> Result<ExistingLegacyImport, SoulError> {
        let source_event_id = legacy_source_event_id(fact);
        let state_id = legacy_state_id(fact);
        let lifecycle_event_id = legacy_lifecycle_event_id(fact);
        let state = self.state(&state_id)?;
        let source = self.event(&source_event_id)?;
        let lifecycle = self.event(&lifecycle_event_id)?;
        if state.is_none() && source.is_none() && lifecycle.is_none() {
            return Ok(ExistingLegacyImport::Absent);
        }
        let Some(state) = state else {
            return Ok(ExistingLegacyImport::Conflicting(
                "partial legacy import is missing canonical state".into(),
            ));
        };
        let Some(source) = source else {
            return Ok(ExistingLegacyImport::Conflicting(
                "partial legacy import is missing source event".into(),
            ));
        };
        let Some(lifecycle) = lifecycle else {
            return Ok(ExistingLegacyImport::Conflicting(
                "partial legacy import is missing lifecycle event".into(),
            ));
        };

        let source_matches = matches!(
            &source.event.payload,
            KernelEventPayload::LegacyImported {
                source_id,
                kind,
                content,
            } if source_id == &fact.id
                && kind == fact.kind.as_str()
                && content.as_str() == fact.text
        );
        let lifecycle_matches = matches!(
            &lifecycle.event.payload,
            KernelEventPayload::StateLifecycle {
                state_id: actual_state_id,
                action: StateLifecycleAction::Created,
                prior_state_id: None,
                ..
            } if actual_state_id == &state_id
        );
        let state_matches = state.kind == StateKind::Fact
            && state.strength == EvidenceStrength::Imported
            && state.scope.subject == StateSubject::Unresolved
            && state.content.as_str() == fact.text
            && state.evidence_refs == vec![source_event_id]
            && matches!(
                state.provenance,
                ExtractionProvenance::LegacyImport { ref table } if table == "facts"
            );
        if source_matches && lifecycle_matches && state_matches {
            Ok(ExistingLegacyImport::Matching)
        } else {
            Ok(ExistingLegacyImport::Conflicting(
                "legacy source row changed after its canonical import".into(),
            ))
        }
    }
}

enum ExistingLegacyImport {
    Absent,
    Matching,
    Conflicting(String),
}

fn legacy_request(context: &LegacyImportContext, fact: &MemoryFact) -> StateWriteRequest {
    let occurred_at = context.occurred_at;
    let source_event_id = legacy_source_event_id(fact);
    let state_id = legacy_state_id(fact);
    let session_id = SessionId::new(fact.session_id.clone());

    let source_event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: source_event_id.clone(),
        occurred_at,
        quest_id: context.quest_id.clone(),
        session_id: session_id.clone(),
        spirit_id: context.spirit_id.clone(),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::LegacyImported {
            source_id: fact.id.clone(),
            kind: fact.kind.as_str().into(),
            content: PersistedText::from_reviewed(fact.text.clone()),
        },
    };
    let lifecycle_event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: legacy_lifecycle_event_id(fact),
        occurred_at,
        quest_id: context.quest_id.clone(),
        session_id,
        spirit_id: context.spirit_id.clone(),
        execution_id: None,
        runtime: None,
        causation_id: Some(source_event_id.clone()),
        correlation_id: None,
        payload: KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
    };
    let draft = StateDraft {
        proposed_key: StateKey::new(format!("legacy.{}.{}", fact.kind.as_str(), fact.id)),
        kind: StateKind::Fact,
        scope: StateScope::unresolved(),
        content: SensitiveText::new(fact.text.clone()),
        claimed_strength: EvidenceStrength::Imported,
        evidence_refs: vec![source_event_id],
        provenance: ExtractionProvenance::LegacyImport {
            table: "facts".into(),
        },
        expires_at: None,
    };

    StateWriteRequest::new(
        StateTransition::Create {
            state_id,
            draft,
            created_at: occurred_at,
        },
        lifecycle_event,
    )
    .with_source_event(source_event)
}

fn legacy_source_event_id(fact: &MemoryFact) -> EventId {
    EventId::new(format!("legacy-import:event:{}", fact.id))
}

fn legacy_lifecycle_event_id(fact: &MemoryFact) -> EventId {
    EventId::new(format!("legacy-import:lifecycle:{}", fact.id))
}

fn legacy_state_id(fact: &MemoryFact) -> StateId {
    StateId::new(format!("legacy-import:state:{}", fact.id))
}
