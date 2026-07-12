//! Presentation-only configuration consumed by the pure Director.

use crate::pack::{PresentationActorId, ValidatedPresentationPack};
use serde::Serialize;

/// Optional presentation copy keyed by coarse situation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct LineBook {
    pub tool_start: Option<String>,
    pub tool_end_ok: Option<String>,
    pub tool_end_err: Option<String>,
    pub waiting: Option<String>,
    pub outcome: Option<String>,
    pub error: Option<String>,
}

/// Explicit actor identity and copy used for stage mapping.
#[derive(Debug, Clone)]
pub struct DirectorContext {
    pub actor_id: PresentationActorId,
    pub line_book: LineBook,
}

impl DirectorContext {
    pub fn new(actor_id: PresentationActorId, line_book: LineBook) -> Self {
        Self {
            actor_id,
            line_book,
        }
    }

    pub fn from_pack(pack: &ValidatedPresentationPack) -> Self {
        Self::new(pack.companion().actor_id.clone(), pack.line_book().clone())
    }
}

impl Default for DirectorContext {
    fn default() -> Self {
        Self::new(
            PresentationActorId("companion".to_owned()),
            LineBook::default(),
        )
    }
}
