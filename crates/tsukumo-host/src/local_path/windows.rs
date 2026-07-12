//! Windows-specific handle guards for local directory trees.

use super::rules::{io_error, unsafe_path};
use super::LocalPathError;
use std::fs::{self, File};
use std::io::Write;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::AsRawHandle;
use std::path::{Component, Path, PathBuf};
use windows_sys::Win32::Storage::FileSystem::{
    GetDriveTypeW, GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
};

const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_FIXED: u32 = 3;
const DRIVE_RAMDISK: u32 = 6;
const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const FILE_SHARE_READ: u32 = 0x1;
const FILE_SHARE_WRITE: u32 = 0x2;
const FILE_READ_ATTRIBUTES: u32 = 0x80;
const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;

pub(super) fn is_reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

pub(super) fn validate_windows_drive(path: &Path) -> Result<(), LocalPathError> {
    use std::os::windows::ffi::OsStrExt;

    let Some(letter) = path.components().find_map(|component| match component {
        Component::Prefix(prefix) => match prefix.kind() {
            std::path::Prefix::Disk(letter) => Some(letter),
            _ => None,
        },
        _ => None,
    }) else {
        return Err(unsafe_path(path, "path has no local drive prefix"));
    };
    let root = format!("{}:\\", char::from(letter));
    let wide = std::ffi::OsStr::new(&root)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    // SAFETY: Category 8 (FFI boundary). `wide` is a live NUL-terminated UTF-16
    // drive-root buffer for the complete duration of this read-only Windows call.
    let drive_type = unsafe { GetDriveTypeW(wide.as_ptr()) };
    if !matches!(drive_type, DRIVE_FIXED | DRIVE_REMOVABLE | DRIVE_RAMDISK) {
        return Err(unsafe_path(
            path,
            "remote and non-local drives are disabled",
        ));
    }
    Ok(())
}

pub(super) fn lock_existing_directory_chain(path: &Path) -> Result<Vec<File>, LocalPathError> {
    directory_chain(path, false)
}

pub(super) fn prepare_and_lock_directory_chain(path: &Path) -> Result<Vec<File>, LocalPathError> {
    directory_chain(path, true)
}

fn directory_chain(path: &Path, create_missing: bool) -> Result<Vec<File>, LocalPathError> {
    let mut current = PathBuf::new();
    let mut locks = Vec::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if !matches!(component, Component::Normal(_)) {
            continue;
        }
        match open_directory(&current) {
            Ok(file) => locks.push(file),
            Err(error) if create_missing && is_not_found(&error) => {
                fs::create_dir(&current).map_err(|source| io_error(&current, source))?;
                // Opening the new entry no-follow catches any concurrent junction replacement.
                locks.push(open_directory(&current)?);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(locks)
}

fn open_directory(path: &Path) -> Result<File, LocalPathError> {
    let file = fs::OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|source| io_error(path, source))?;
    let attributes = file
        .metadata()
        .map_err(|source| io_error(path, source))?
        .file_attributes();
    if attributes & FILE_ATTRIBUTE_DIRECTORY == 0 || attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(unsafe_path(path, "expected a non-reparse local directory"));
    }
    Ok(file)
}

pub(super) fn open_existing_regular_file(path: &Path) -> Result<File, LocalPathError> {
    let file = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|source| io_error(path, source))?;
    validate_regular_handle(path, &file)?;
    Ok(file)
}

pub(super) fn open_or_create_guarded_file(
    path: &Path,
    initial: &[u8],
) -> Result<File, LocalPathError> {
    let options = || {
        let mut options = fs::OpenOptions::new();
        options
            .read(true)
            .write(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
        options
    };
    let mut create = options();
    create.create_new(true);
    let file =
        match create.open(path) {
            Ok(mut file) => {
                file.write_all(initial)
                    .and_then(|()| file.sync_data())
                    .map_err(|source| io_error(path, source))?;
                file
            }
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => options()
                .open(path)
                .map_err(|source| io_error(path, source))?,
            Err(source) => return Err(io_error(path, source)),
        };
    validate_regular_handle(path, &file)?;
    Ok(file)
}

pub(super) fn has_multiple_links(path: &Path, _metadata: &fs::Metadata) -> bool {
    open_existing_regular_file(path).is_err()
}

fn validate_regular_handle(path: &Path, file: &File) -> Result<(), LocalPathError> {
    let metadata = file.metadata().map_err(|source| io_error(path, source))?;
    if !metadata.is_file() || is_reparse_or_symlink(&metadata) {
        return Err(unsafe_path(path, "expected a non-reparse regular file"));
    }
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: Category 8 (FFI boundary). `file` owns a live handle for the call,
    // `information` is aligned writable storage, and Windows initializes it on success.
    let succeeded = unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) };
    if succeeded == 0 || information.nNumberOfLinks > 1 {
        return Err(unsafe_path(path, "hard-linked files are disabled"));
    }
    Ok(())
}

fn is_not_found(error: &LocalPathError) -> bool {
    matches!(
        error,
        LocalPathError::Io { source, .. } if source.kind() == std::io::ErrorKind::NotFound
    )
}
