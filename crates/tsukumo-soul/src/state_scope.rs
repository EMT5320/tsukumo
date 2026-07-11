//! Subject and applicability coordinates for canonical relationship state.

use serde::{Deserialize, Serialize};
use tsukumo_kernel::{OwnerId, SpiritId, WorkspaceId};

/// Subject whose relationship state is being described.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StateSubject {
    Owner {
        owner_id: OwnerId,
    },
    Workspace {
        workspace_id: WorkspaceId,
    },
    Spirit {
        spirit_id: SpiritId,
    },
    Relationship {
        owner_id: OwnerId,
        spirit_id: SpiritId,
    },
    /// Transitional ownership used only by reviewed legacy imports.
    Unresolved,
}

/// Operating-system applicability coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingSystem {
    Windows,
    Linux,
    Macos,
}

/// Context coordinates that decide when a state may be selected.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateApplicability {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<OperatingSystem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub task_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub language_tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<String>,
}

impl StateApplicability {
    fn canonicalized(&self) -> Self {
        let mut normalized = self.clone();
        normalize_tags(&mut normalized.task_tags);
        normalize_tags(&mut normalized.language_tags);
        normalize_tags(&mut normalized.required_capabilities);
        normalized
    }
}

/// Ownership and applicability of one state version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateScope {
    pub subject: StateSubject,
    pub applicability: StateApplicability,
}

impl StateScope {
    /// Creates the frozen GNU example scope for one workspace and OS.
    pub fn workspace_os(workspace: impl Into<String>, operating_system: OperatingSystem) -> Self {
        let workspace_id = WorkspaceId::new(workspace);
        Self {
            subject: StateSubject::Workspace {
                workspace_id: workspace_id.clone(),
            },
            applicability: StateApplicability {
                workspace: Some(workspace_id),
                operating_system: Some(operating_system),
                task_tags: vec!["rust_build".into(), "rust_test".into()],
                language_tags: vec!["rust".into()],
                required_capabilities: Vec::new(),
            },
        }
    }

    pub fn owner(owner_id: OwnerId) -> Self {
        Self {
            subject: StateSubject::Owner { owner_id },
            applicability: StateApplicability::default(),
        }
    }

    pub fn spirit(spirit_id: SpiritId) -> Self {
        Self {
            subject: StateSubject::Spirit { spirit_id },
            applicability: StateApplicability::default(),
        }
    }

    pub fn relationship(owner_id: OwnerId, spirit_id: SpiritId) -> Self {
        Self {
            subject: StateSubject::Relationship {
                owner_id,
                spirit_id,
            },
            applicability: StateApplicability::default(),
        }
    }

    pub fn unresolved() -> Self {
        Self {
            subject: StateSubject::Unresolved,
            applicability: StateApplicability::default(),
        }
    }

    pub(crate) fn canonical_key(&self) -> Result<String, serde_json::Error> {
        let normalized = Self {
            subject: self.subject.clone(),
            applicability: self.applicability.canonicalized(),
        };
        serde_json::to_string(&normalized)
    }

    /// Returns whether every state applicability coordinate is satisfied.
    pub(crate) fn applies_to(&self, context: &Self) -> bool {
        if self.subject != context.subject {
            return false;
        }
        let required = self.applicability.canonicalized();
        let available = context.applicability.canonicalized();
        option_matches(&required.workspace, &available.workspace)
            && option_matches(&required.operating_system, &available.operating_system)
            && is_subset(&required.task_tags, &available.task_tags)
            && is_subset(&required.language_tags, &available.language_tags)
            && is_subset(
                &required.required_capabilities,
                &available.required_capabilities,
            )
    }

    /// Counts applicability coordinates for deterministic specificity ranking.
    pub(crate) fn specificity_score(&self) -> usize {
        let applicability = self.applicability.canonicalized();
        usize::from(applicability.workspace.is_some())
            + usize::from(applicability.operating_system.is_some())
            + applicability.task_tags.len()
            + applicability.language_tags.len()
            + applicability.required_capabilities.len()
    }
}

fn option_matches<T: PartialEq>(required: &Option<T>, available: &Option<T>) -> bool {
    match required {
        None => true,
        Some(value) => available.as_ref() == Some(value),
    }
}

fn is_subset(required: &[String], available: &[String]) -> bool {
    required
        .iter()
        .all(|value| available.binary_search(value).is_ok())
}

fn normalize_tags(tags: &mut Vec<String>) {
    for tag in tags.iter_mut() {
        *tag = tag.trim().to_ascii_lowercase();
    }
    tags.retain(|tag| !tag.is_empty());
    tags.sort();
    tags.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_subject_variants_roundtrip() {
        let subjects = [
            StateSubject::Owner {
                owner_id: OwnerId::new("owner"),
            },
            StateSubject::Workspace {
                workspace_id: WorkspaceId::new("workspace"),
            },
            StateSubject::Spirit {
                spirit_id: SpiritId::new("spirit"),
            },
            StateSubject::Relationship {
                owner_id: OwnerId::new("owner"),
                spirit_id: SpiritId::new("spirit"),
            },
        ];
        for subject in subjects {
            let json = serde_json::to_string(&subject).expect("serialize subject");
            let reopened =
                serde_json::from_str::<StateSubject>(&json).expect("deserialize subject");
            assert_eq!(reopened, subject);
        }
    }

    #[test]
    fn canonical_scope_key_sorts_and_deduplicates_applicability_tags() {
        let mut left = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
        left.applicability.task_tags = vec![
            "rust_test".into(),
            " Rust_Build ".into(),
            "rust_test".into(),
        ];
        let mut right = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
        right.applicability.task_tags = vec!["rust_build".into(), "rust_test".into()];

        assert_eq!(
            left.canonical_key().expect("canonicalize left"),
            right.canonical_key().expect("canonicalize right")
        );
    }
}
