//! Typed presentation-pack boundary failures.

use std::path::PathBuf;
use thiserror::Error;

/// Failures produced while parsing or validating inert presentation data.
#[derive(Debug, Error)]
pub enum PresentationPackError {
    #[error("invalid JSON in {path}: {source}")]
    InvalidJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("unsupported presentation-pack schema {found}")]
    UnsupportedSchema { found: u16 },
    #[error("presentation-pack path must stay relative to its root: {path}")]
    InvalidPath { path: PathBuf },
    #[error("duplicate presentation-pack id: {id}")]
    DuplicateId { id: String },
    #[error("presentation-pack field {field} exceeds the maximum of {maximum}")]
    LimitExceeded { field: &'static str, maximum: usize },
    #[error("presentation-pack references missing palette index {index}")]
    InvalidPaletteIndex { index: u8 },
    #[error("invalid presentation-pack field {field}: {reason}")]
    InvalidModel { field: &'static str, reason: String },
}
