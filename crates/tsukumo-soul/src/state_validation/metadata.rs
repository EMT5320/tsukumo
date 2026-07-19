//! Bounded state key, provenance, subject, and applicability validation.

use crate::state_model::{
    EvidenceStrength, ExtractionProvenance, StateDraft, StateKind, StateSubject,
    StateValidationError,
};
use crate::storage::SoulError;
use tsukumo_kernel::contains_sensitive_material;
pub(super) fn validate_metadata(draft: &StateDraft) -> Result<(), SoulError> {
    const MAX_KEY_CHARS: usize = 256;
    const MAX_CONTENT_CHARS: usize = 16_384;
    const MAX_EVIDENCE_REFS: usize = 64;
    let key = draft.proposed_key.as_str();
    let key_is_safe = valid_metadata_text(key, MAX_KEY_CHARS)
        && key.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | ':' | '-')
        });
    let key_matches_subject = match &draft.scope.subject {
        StateSubject::Workspace { workspace_id } => {
            key.starts_with(&format!("workspace.{}.", workspace_id.as_str()))
        }
        StateSubject::Owner { .. } => key.starts_with("owner."),
        StateSubject::Spirit { .. } => key.starts_with("spirit."),
        StateSubject::Relationship { .. } => key.starts_with("relationship."),
        StateSubject::Unresolved => key.starts_with("legacy."),
    };
    let provenance_is_safe = match &draft.provenance {
        ExtractionProvenance::Rule { name, .. } => valid_metadata_text(name, 128),
        ExtractionProvenance::StructuredModel {
            provider, model, ..
        } => valid_metadata_text(provider, 128) && valid_metadata_text(model, 128),
        ExtractionProvenance::Recorded { fixture, .. } => valid_metadata_text(fixture, 128),
        ExtractionProvenance::LegacyImport { table } => valid_metadata_text(table, 128),
    };
    if !key_is_safe
        || !key_matches_subject
        || !provenance_is_safe
        || draft.content.expose().chars().count() > MAX_CONTENT_CHARS
        || draft.evidence_refs.len() > MAX_EVIDENCE_REFS
    {
        return Err(StateValidationError::InvalidMetadata.into());
    }
    Ok(())
}

fn valid_metadata_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty()
        && value.chars().count() <= max_chars
        && !value.chars().any(char::is_control)
        && !contains_sensitive_material(value)
}
pub(super) fn validate_scope(draft: &StateDraft) -> Result<(), SoulError> {
    if matches!(draft.scope.subject, StateSubject::Unresolved) {
        if is_valid_legacy_draft(draft) {
            return Ok(());
        }
        return Err(StateValidationError::UnresolvedScope.into());
    }

    let valid_subject = match &draft.scope.subject {
        StateSubject::Owner { owner_id } => valid_metadata_text(owner_id.as_str(), 128),
        StateSubject::Workspace { workspace_id } => {
            valid_metadata_text(workspace_id.as_str(), 128)
                && draft.scope.applicability.workspace.as_ref() == Some(workspace_id)
        }
        StateSubject::Spirit { spirit_id } => valid_metadata_text(spirit_id.as_str(), 128),
        StateSubject::Relationship {
            owner_id,
            spirit_id,
        } => {
            valid_metadata_text(owner_id.as_str(), 128)
                && valid_metadata_text(spirit_id.as_str(), 128)
        }
        StateSubject::Unresolved => false,
    };
    let valid_workspace = draft
        .scope
        .applicability
        .workspace
        .as_ref()
        .is_none_or(|workspace| valid_metadata_text(workspace.as_str(), 128));
    let valid_tags = [
        &draft.scope.applicability.task_tags,
        &draft.scope.applicability.language_tags,
        &draft.scope.applicability.required_capabilities,
    ]
    .iter()
    .all(|tags| tags.len() <= 32 && tags.iter().all(|tag| valid_metadata_text(tag, 128)));
    if !valid_subject || !valid_workspace || !valid_tags {
        return Err(StateValidationError::InvalidScope.into());
    }
    Ok(())
}

fn is_valid_legacy_draft(draft: &StateDraft) -> bool {
    matches!(draft.provenance, ExtractionProvenance::LegacyImport { .. })
        && draft.claimed_strength == EvidenceStrength::Imported
        && draft.kind == StateKind::Fact
}
