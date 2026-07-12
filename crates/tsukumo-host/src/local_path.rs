//! Local-only filesystem boundary for storage and external presentation packs.

mod guard;
mod rules;
#[cfg(test)]
mod tests;
#[cfg(windows)]
mod windows;

pub(crate) use guard::LocalDirectoryGuard;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum LocalPathError {
    #[error("unsafe local path {path}: {reason}")]
    Unsafe { path: String, reason: &'static str },
    #[error("failed to access local path {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: io::Error,
    },
}
