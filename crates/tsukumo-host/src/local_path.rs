//! Local-only filesystem boundary for storage and external presentation packs.

mod guard;
mod rules;
#[cfg(test)]
mod tests;
#[cfg(windows)]
mod windows;

pub(crate) use guard::LocalDirectoryGuard;
use std::io;
use std::path::Path;
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

pub(crate) fn prepare_data_directory(
    data_dir: &Path,
) -> Result<LocalDirectoryGuard, LocalPathError> {
    let mut guard = LocalDirectoryGuard::prepare(data_dir)?;
    guard.validate_tree()?;
    guard
        .ensure_directory(Path::new("skills"))
        .and_then(|()| guard.ensure_guarded_file(Path::new("soul.db"), b""))
        .and_then(|()| guard.ensure_guarded_file(Path::new("soul.db-journal"), b""))
        .and_then(|()| guard.ensure_guarded_file(Path::new("soul.db-wal"), b""))
        .and_then(|()| guard.ensure_guarded_file(Path::new("soul.db-shm"), b""))
        .and_then(|()| guard.ensure_guarded_file(Path::new("MEMORY.md"), b"# MEMORY\n\n"))
        .and_then(|()| guard.ensure_guarded_file(Path::new("USER.md"), b"# USER\n\n"))?;
    Ok(guard)
}
