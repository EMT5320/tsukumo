//! Durable identities shared across Tsukumo crates.
//!
//! Each semantic identifier is a distinct type so unrelated ledger keys cannot
//! be mixed accidentally. A spirit remains stable while its runtime binding
//! changes between executions.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Creates an opaque identifier from its durable string value.
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Returns the durable string representation.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }
    };
}

string_id!(
    /// Globally unique identifier for a persisted Chronicle event.
    EventId
);
string_id!(
    /// Identifier for a user-visible quest or delegated objective.
    QuestId
);
string_id!(
    /// Identifier for a host session that may contain several executions.
    SessionId
);
string_id!(
    /// Persistent identifier for the human owner of relationship state.
    OwnerId
);
string_id!(
    /// Persistent identifier for a workspace that owns project state.
    WorkspaceId
);
string_id!(
    /// Persistent Tsukumo identity independent of the current runtime.
    SpiritId
);
string_id!(
    /// Identifier for one concrete runtime execution.
    ExecutionId
);
string_id!(
    /// Identifier for one immutable canonical state version.
    StateId
);
string_id!(
    /// Identifier for one immutable handoff checkpoint.
    CheckpointId
);
string_id!(
    /// Identifier for one immutable runtime projection.
    ProjectionId
);
string_id!(
    /// Identifier that connects request/response or start/end events.
    CorrelationId
);
string_id!(
    /// Identifier for a retained or generated artifact.
    ArtifactId
);

/// Runtime family used for an execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Builtin,
    Acp,
    ClaudeCli,
    CodexCli,
}

/// How the runtime data entered Tsukumo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    OwnedProcess,
    Fixture,
    Synthetic,
}

/// Runtime attachment for an execution, distinct from SpiritId.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeBinding {
    pub kind: RuntimeKind,
    pub mode: RuntimeMode,
}

impl RuntimeBinding {
    /// Creates a runtime binding from its family and transport mode.
    pub const fn new(kind: RuntimeKind, mode: RuntimeMode) -> Self {
        Self { kind, mode }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spirit_id_roundtrips_json() {
        // Given: a persistent spirit identity.
        let id = SpiritId::new("yuka");

        // When: the ID crosses JSON.
        let json = serde_json::to_string(&id).expect("serialize SpiritId");
        let reopened: SpiritId = serde_json::from_str(&json).expect("deserialize SpiritId");

        // Then: the transparent wire value remains stable.
        assert_eq!(json, "\"yuka\"");
        assert_eq!(reopened, id);
    }

    #[test]
    fn runtime_binding_keeps_runtime_separate_from_transport() {
        // Given: a recorded Claude fixture.
        let binding = RuntimeBinding::new(RuntimeKind::ClaudeCli, RuntimeMode::Fixture);

        // When: the binding is serialized.
        let json = serde_json::to_string(&binding).expect("serialize RuntimeBinding");

        // Then: runtime family and transport mode remain explicit.
        assert_eq!(json, r#"{"kind":"claude_cli","mode":"fixture"}"#);
    }
}
