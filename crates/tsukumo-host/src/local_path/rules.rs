//! Lexical and metadata rules shared by guarded path operations.

use super::LocalPathError;
use std::fs;
use std::path::{Component, Path, PathBuf};
use tsukumo_kernel::is_terminal_unsafe_character;

pub(super) fn checked_absolute(path: &Path) -> Result<PathBuf, LocalPathError> {
    validate_lexical(path)?;
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|source| io_error(path, source))?
            .join(path)
    };
    #[cfg(target_os = "macos")]
    let absolute = normalize_macos_system_alias(absolute);
    validate_lexical(&absolute)?;
    #[cfg(windows)]
    super::windows::validate_windows_drive(&absolute)?;
    Ok(absolute)
}

#[cfg(target_os = "macos")]
fn normalize_macos_system_alias(path: PathBuf) -> PathBuf {
    for (alias, canonical) in [
        (Path::new("/var"), Path::new("/private/var")),
        (Path::new("/tmp"), Path::new("/private/tmp")),
    ] {
        if let Ok(relative) = path.strip_prefix(alias) {
            return canonical.join(relative);
        }
    }
    path
}

pub(super) fn validate_lexical(path: &Path) -> Result<(), LocalPathError> {
    if path.as_os_str().is_empty() {
        return Err(unsafe_path(path, "path cannot be empty"));
    }
    for component in path.components() {
        #[cfg(windows)]
        if let Component::Prefix(prefix) = component {
            if !matches!(prefix.kind(), std::path::Prefix::Disk(_)) {
                return Err(unsafe_path(
                    path,
                    "UNC, verbatim, and device paths are disabled",
                ));
            }
        }
        if let Component::Normal(value) = component {
            let value = value.to_string_lossy();
            if value.chars().any(is_terminal_unsafe_character)
                || value.contains(':')
                || value.ends_with('.')
                || value.ends_with(' ')
                || is_windows_device_name(&value)
            {
                return Err(unsafe_path(
                    path,
                    "path component is terminal-unsafe or device-like",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub(super) fn validate_existing_components(path: &Path) -> Result<(), LocalPathError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if is_reparse_or_symlink(&metadata) => {
                return Err(unsafe_path(
                    path,
                    "symbolic links and reparse points are disabled",
                ));
            }
            Ok(_) => {}
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => break,
            Err(source) => return Err(io_error(&current, source)),
        }
    }
    Ok(())
}

#[cfg(windows)]
pub(super) fn is_reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    super::windows::is_reparse_or_symlink(metadata)
}

#[cfg(not(windows))]
pub(super) fn is_reparse_or_symlink(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

#[cfg(windows)]
pub(super) fn has_multiple_links(path: &Path, metadata: &fs::Metadata) -> bool {
    super::windows::has_multiple_links(path, metadata)
}

#[cfg(unix)]
pub(super) fn has_multiple_links(_path: &Path, metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    metadata.nlink() > 1
}

#[cfg(not(any(windows, unix)))]
pub(super) fn has_multiple_links(_path: &Path, _metadata: &fs::Metadata) -> bool {
    false
}

fn is_windows_device_name(value: &str) -> bool {
    // Win32 also recognizes superscript one, two, and three in COM/LPT aliases.
    let stem = value
        .split('.')
        .next()
        .unwrap_or_default()
        .trim_end_matches(['.', ' '])
        .chars()
        .map(|character| match character {
            '\u{00b9}' => '1',
            '\u{00b2}' => '2',
            '\u{00b3}' => '3',
            other => other.to_ascii_uppercase(),
        })
        .collect::<String>();
    matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "CLOCK$"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

fn safe_path_label(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .flat_map(char::escape_default)
        .take(512)
        .collect()
}

pub(super) fn unsafe_path(path: &Path, reason: &'static str) -> LocalPathError {
    LocalPathError::Unsafe {
        path: safe_path_label(path),
        reason,
    }
}

pub(super) fn io_error(path: &Path, source: std::io::Error) -> LocalPathError {
    LocalPathError::Io {
        path: safe_path_label(path),
        source,
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn fixed_macos_system_aliases_are_normalized_before_symlink_validation() {
        assert_eq!(
            checked_absolute(Path::new("/var/folders/tsukumo")).expect("normalize /var"),
            PathBuf::from("/private/var/folders/tsukumo")
        );
        assert_eq!(
            checked_absolute(Path::new("/tmp/tsukumo")).expect("normalize /tmp"),
            PathBuf::from("/private/tmp/tsukumo")
        );
    }
}
