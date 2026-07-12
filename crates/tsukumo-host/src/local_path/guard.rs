//! Directory capabilities and guarded file access.

#[cfg(not(windows))]
use super::rules::validate_existing_components;
use super::rules::{
    checked_absolute, has_multiple_links, io_error, is_reparse_or_symlink, unsafe_path,
    validate_lexical,
};
use super::LocalPathError;
use std::fs::{self, File};
#[cfg(not(windows))]
use std::io::Write;
use std::path::{Component, Path, PathBuf};

/// Holds no-delete handles for every trusted Windows path component.
pub(crate) struct LocalDirectoryGuard {
    root: PathBuf,
    #[cfg(windows)]
    directory_locks: Vec<File>,
    #[cfg(windows)]
    file_locks: Vec<File>,
}

impl LocalDirectoryGuard {
    pub(crate) fn existing(path: &Path) -> Result<Self, LocalPathError> {
        let root = checked_absolute(path)?;
        #[cfg(windows)]
        let directory_locks = super::windows::lock_existing_directory_chain(&root)?;
        #[cfg(not(windows))]
        {
            validate_existing_components(&root)?;
            let metadata = fs::metadata(&root).map_err(|source| io_error(&root, source))?;
            if !metadata.is_dir() {
                return Err(unsafe_path(&root, "expected a local directory"));
            }
        }
        Ok(Self {
            root,
            #[cfg(windows)]
            directory_locks,
            #[cfg(windows)]
            file_locks: Vec::new(),
        })
    }

    pub(crate) fn prepare(path: &Path) -> Result<Self, LocalPathError> {
        let root = checked_absolute(path)?;
        #[cfg(windows)]
        let directory_locks = super::windows::prepare_and_lock_directory_chain(&root)?;
        #[cfg(not(windows))]
        {
            validate_existing_components(&root)?;
            fs::create_dir_all(&root).map_err(|source| io_error(&root, source))?;
            validate_existing_components(&root)?;
        }
        Ok(Self {
            root,
            #[cfg(windows)]
            directory_locks,
            #[cfg(windows)]
            file_locks: Vec::new(),
        })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn open_regular_file(&mut self, relative: &Path) -> Result<File, LocalPathError> {
        let path = self.relative_path(relative)?;
        self.lock_parent_chain(&path)?;
        #[cfg(windows)]
        return super::windows::open_existing_regular_file(&path);
        #[cfg(not(windows))]
        existing_regular_file(&path)
            .and_then(|path| File::open(&path).map_err(|source| io_error(&path, source)))
    }

    pub(crate) fn ensure_directory(&mut self, relative: &Path) -> Result<(), LocalPathError> {
        let path = self.relative_path(relative)?;
        #[cfg(windows)]
        self.directory_locks
            .extend(super::windows::prepare_and_lock_directory_chain(&path)?);
        #[cfg(not(windows))]
        {
            validate_existing_components(&path)?;
            fs::create_dir_all(&path).map_err(|source| io_error(&path, source))?;
            validate_existing_components(&path)?;
        }
        Ok(())
    }

    pub(crate) fn ensure_guarded_file(
        &mut self,
        relative: &Path,
        initial: &[u8],
    ) -> Result<(), LocalPathError> {
        let path = self.relative_path(relative)?;
        self.lock_parent_chain(&path)?;
        #[cfg(windows)]
        self.file_locks
            .push(super::windows::open_or_create_guarded_file(&path, initial)?);
        #[cfg(not(windows))]
        ensure_regular_file(&path, initial)?;
        Ok(())
    }

    pub(crate) fn validate_tree(&mut self) -> Result<(), LocalPathError> {
        const MAX_ENTRIES: usize = 10_000;
        let mut pending = vec![self.root.clone()];
        let mut visited = 0usize;
        while let Some(directory) = pending.pop() {
            self.lock_directory_chain(&directory)?;
            for entry in fs::read_dir(&directory).map_err(|source| io_error(&directory, source))? {
                let entry = entry.map_err(|source| io_error(&directory, source))?;
                visited = visited.saturating_add(1);
                if visited > MAX_ENTRIES {
                    return Err(unsafe_path(
                        &self.root,
                        "local tree exceeds the entry budget",
                    ));
                }
                let path = entry.path();
                let metadata =
                    fs::symlink_metadata(&path).map_err(|source| io_error(&path, source))?;
                if is_reparse_or_symlink(&metadata) {
                    return Err(unsafe_path(
                        &path,
                        "symbolic links and reparse points are disabled",
                    ));
                }
                if metadata.is_dir() {
                    pending.push(path);
                } else if !metadata.is_file() || has_multiple_links(&path, &metadata) {
                    return Err(unsafe_path(
                        &path,
                        "only regular single-link files and directories are allowed",
                    ));
                }
            }
        }
        Ok(())
    }

    fn relative_path(&self, relative: &Path) -> Result<PathBuf, LocalPathError> {
        if relative.as_os_str().is_empty()
            || relative.is_absolute()
            || !relative
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err(unsafe_path(relative, "expected a normalized relative path"));
        }
        let path = self.root.join(relative);
        validate_lexical(&path)?;
        Ok(path)
    }

    fn lock_parent_chain(&mut self, path: &Path) -> Result<(), LocalPathError> {
        let parent = path
            .parent()
            .ok_or_else(|| unsafe_path(path, "local file has no parent directory"))?;
        self.lock_directory_chain(parent)
    }

    fn lock_directory_chain(&mut self, path: &Path) -> Result<(), LocalPathError> {
        #[cfg(windows)]
        self.directory_locks
            .extend(super::windows::lock_existing_directory_chain(path)?);
        #[cfg(not(windows))]
        validate_existing_components(path)?;
        Ok(())
    }
}

#[cfg(not(windows))]
fn existing_regular_file(path: &Path) -> Result<PathBuf, LocalPathError> {
    validate_existing_components(path)?;
    let metadata = fs::metadata(path).map_err(|source| io_error(path, source))?;
    if !metadata.is_file() || has_multiple_links(path, &metadata) {
        return Err(unsafe_path(path, "expected a regular local file"));
    }
    Ok(path.to_path_buf())
}

#[cfg(not(windows))]
fn ensure_regular_file(path: &Path, initial: &[u8]) -> Result<(), LocalPathError> {
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).create(true);
    let mut file = options
        .open(path)
        .map_err(|source| io_error(path, source))?;
    if file
        .metadata()
        .map_err(|source| io_error(path, source))?
        .len()
        == 0
    {
        file.write_all(initial)
            .and_then(|()| file.sync_data())
            .map_err(|source| io_error(path, source))?;
    }
    let metadata = file.metadata().map_err(|source| io_error(path, source))?;
    if !metadata.is_file() || has_multiple_links(path, &metadata) {
        return Err(unsafe_path(path, "expected a regular local file"));
    }
    Ok(())
}
