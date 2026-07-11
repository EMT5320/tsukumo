//! Evidence-strength and explicit-language trust rules.

use crate::state_model::{
    EvidenceStrength, ExtractionProvenance, OperatingSystem, StateDraft, StateKind, StateSubject,
    StateValidationError,
};
use crate::storage::SoulError;
use tsukumo_kernel::{KernelEvent, KernelEventPayload};
pub(crate) fn is_explicit_gnu_user_text(text: &str) -> bool {
    let normalized = text
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let normalized = normalized
        .trim_end_matches(|character: char| {
            matches!(character, '.' | '!' | ';' | ':' | '\u{3002}' | '\u{ff01}')
        })
        .trim_end();

    matches!(
        normalized,
        "use gnu on windows"
            | "use gnu rust toolchain on windows"
            | "use the gnu rust toolchain on windows"
            | "always use gnu on windows"
            | "always use the gnu rust toolchain on windows"
            | "tsukumo uses gnu on windows"
            | "tsukumo uses the gnu rust toolchain on windows"
            | "tsukumo always uses gnu on windows"
            | "tsukumo always uses the gnu rust toolchain on windows"
            | "this project uses gnu on windows"
            | "this project uses the gnu rust toolchain on windows"
            | "this project always uses gnu on windows"
            | "this project always uses the gnu rust toolchain on windows"
            | "\u{5728} windows \u{4e0a}\u{56fa}\u{5b9a}\u{4f7f}\u{7528} gnu"
            | "\u{5728} windows \u{4e0a}\u{56fa}\u{5b9a}\u{4f7f}\u{7528} gnu rust \u{5de5}\u{5177}\u{94fe}"
            | "\u{5728} windows \u{4e0a}\u{7edf}\u{4e00}\u{4f7f}\u{7528} gnu"
            | "\u{5728} windows \u{4e0a}\u{7edf}\u{4e00}\u{4f7f}\u{7528} gnu rust \u{5de5}\u{5177}\u{94fe}"
    )
}

pub(super) fn validate_strength(
    draft: &StateDraft,
    evidence: &[KernelEvent],
) -> Result<(), SoulError> {
    if draft.claimed_strength == EvidenceStrength::Repeated {
        let mut distinct = draft.evidence_refs.clone();
        distinct.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        distinct.dedup();
        if distinct.len() < 2 {
            return Err(StateValidationError::RepeatedEvidenceRequired.into());
        }
    }
    if draft.kind == StateKind::Constraint && draft.claimed_strength != EvidenceStrength::Explicit {
        return Err(StateValidationError::InferredConstraint.into());
    }
    if draft.claimed_strength != EvidenceStrength::Explicit {
        return Ok(());
    }

    let trusted_rule = matches!(
        &draft.provenance,
        ExtractionProvenance::Rule { name, version }
            if name == "explicit_gnu_constraint" && *version == 1
    );
    if !trusted_rule || !is_valid_explicit_gnu_draft(draft) {
        return Err(StateValidationError::UntrustedExplicit.into());
    }
    let explicit_evidence = evidence.iter().any(|event| {
        matches!(
            &event.payload,
            KernelEventPayload::UserInput { content }
                if is_explicit_gnu_user_text(content.as_str())
        )
    });
    if !explicit_evidence {
        return Err(StateValidationError::ExplicitEvidenceRequired.into());
    }
    Ok(())
}

fn is_valid_explicit_gnu_draft(draft: &StateDraft) -> bool {
    let Some(workspace) = draft.scope.applicability.workspace.as_ref() else {
        return false;
    };
    matches!(
        &draft.scope.subject,
        StateSubject::Workspace { workspace_id } if workspace_id == workspace
    ) && draft.scope.applicability.operating_system == Some(OperatingSystem::Windows)
        && draft.proposed_key.as_str()
            == format!("workspace.{}.rust.toolchain.windows", workspace.as_str())
        && draft.content.expose() == "Use the GNU Rust toolchain on Windows"
}

#[cfg(test)]
mod tests {
    use super::is_explicit_gnu_user_text;

    #[test]
    fn explicit_gnu_rule_rejects_ambiguous_or_negative_language() {
        // Given: safe positive commands plus negative, question, and one-off observations.
        let positive = [
            "Use GNU on Windows",
            "Always use GNU on Windows",
            "Tsukumo always uses the GNU Rust toolchain on Windows",
            "\u{5728} Windows \u{4e0a}\u{56fa}\u{5b9a}\u{4f7f}\u{7528} GNU Rust \u{5de5}\u{5177}\u{94fe}",
        ];
        let rejected = [
            "Do not use GNU on Windows",
            "Use MSVC, not GNU, on Windows",
            "Use MSVC instead of GNU on Windows",
            "Use a non-GNU toolchain on Windows",
            "\u{4e0d}\u{518d}\u{5728} Windows \u{4e0a}\u{56fa}\u{5b9a}\u{4f7f}\u{7528} GNU Rust \u{5de5}\u{5177}\u{94fe}",
            "Use Gnumeric on Windows",
            "Should we use GNU on Windows?",
            "I use GNU once on Windows",
            "Maybe use GNU on Windows",
            "\u{4e0d}\u{8981}\u{5728} Windows \u{4e0a}\u{4f7f}\u{7528} GNU",
        ];

        // When/Then: only unambiguous durable instructions match the trusted rule.
        for text in positive {
            assert!(
                is_explicit_gnu_user_text(text),
                "expected positive match: {text}"
            );
        }
        for text in rejected {
            assert!(
                !is_explicit_gnu_user_text(text),
                "expected rejected match: {text}"
            );
        }
    }
}
