//! Host-owned presentation-pack source selection and bounded local file I/O.

use crate::local_path::{LocalDirectoryGuard, LocalPathError};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tsukumo_theater::{
    parse_presentation_pack, presentation_pack_assets, PackDocuments, PresentationPackError,
    ValidatedPresentationPack,
};

const MAX_PACK_BYTES: usize = 4 * 1024 * 1024;
const DEFAULT_MANIFEST: &str = include_str!("../content/default-shiori/pack.json");
const DEFAULT_SCENE: &str = include_str!("../content/default-shiori/scene.json");
const DEFAULT_SPRITE: &str = include_str!("../content/default-shiori/sprites/shiori.json");

/// Selects the bundled pack or one explicit external directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentationPackSource {
    EmbeddedDefault,
    Directory(PathBuf),
}

/// Failures while selecting and reading presentation data.
#[derive(Debug, Error)]
pub enum PresentationPackLoadError {
    #[error("failed to access presentation-pack path {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: io::Error,
    },
    #[error("presentation-pack local path rejected: {0}")]
    LocalPath(String),
    #[error("presentation-pack input exceeds {maximum} bytes: {path}")]
    FileTooLarge { path: String, maximum: usize },
    #[error("presentation-pack file is not UTF-8: {path}")]
    InvalidUtf8 {
        path: String,
        #[source]
        source: std::string::FromUtf8Error,
    },
    #[error(transparent)]
    Pack(#[from] PresentationPackError),
}

/// Loads and validates one pack before terminal state is changed.
pub fn load_presentation_pack(
    source: &PresentationPackSource,
) -> Result<ValidatedPresentationPack, PresentationPackLoadError> {
    match source {
        PresentationPackSource::EmbeddedDefault => parse_presentation_pack(PackDocuments::new(
            DEFAULT_MANIFEST,
            DEFAULT_SCENE,
            DEFAULT_SPRITE,
        ))
        .map_err(PresentationPackLoadError::from),
        PresentationPackSource::Directory(root) => load_directory(root),
    }
}

fn load_directory(root: &Path) -> Result<ValidatedPresentationPack, PresentationPackLoadError> {
    let mut root = LocalDirectoryGuard::existing(root).map_err(local_path_error)?;
    let manifest_path = root.root().join("pack.json");
    let manifest_file = root
        .open_regular_file(Path::new("pack.json"))
        .map_err(local_path_error)?;
    let manifest = read_bounded_utf8(manifest_file, &manifest_path, MAX_PACK_BYTES)?;
    let assets = presentation_pack_assets(&manifest)?;
    let scene_path = root.root().join(assets.scene.as_path());
    let scene_file = root
        .open_regular_file(assets.scene.as_path())
        .map_err(local_path_error)?;
    let sprite_path = root.root().join(assets.sprite.as_path());
    let sprite_file = root
        .open_regular_file(assets.sprite.as_path())
        .map_err(local_path_error)?;
    let remaining = MAX_PACK_BYTES.checked_sub(manifest.len()).ok_or(
        PresentationPackLoadError::FileTooLarge {
            path: safe_path_label(&manifest_path),
            maximum: MAX_PACK_BYTES,
        },
    )?;
    let scene = read_bounded_utf8(scene_file, &scene_path, remaining)?;
    let remaining =
        remaining
            .checked_sub(scene.len())
            .ok_or(PresentationPackLoadError::FileTooLarge {
                path: safe_path_label(&scene_path),
                maximum: MAX_PACK_BYTES,
            })?;
    let sprite = read_bounded_utf8(sprite_file, &sprite_path, remaining)?;

    parse_presentation_pack(PackDocuments::new(&manifest, &scene, &sprite))
        .map_err(PresentationPackLoadError::from)
}

fn read_bounded_utf8(
    file: File,
    path: &Path,
    maximum: usize,
) -> Result<String, PresentationPackLoadError> {
    let metadata = file
        .metadata()
        .map_err(|source| PresentationPackLoadError::Io {
            path: safe_path_label(path),
            source,
        })?;
    if !metadata.is_file() {
        return Err(PresentationPackLoadError::LocalPath(format!(
            "{} is not a regular file",
            safe_path_label(path)
        )));
    }
    let limit = u64::try_from(maximum).unwrap_or(u64::MAX).saturating_add(1);
    let mut bytes = Vec::with_capacity(maximum.min(64 * 1024));
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(|source| PresentationPackLoadError::Io {
            path: safe_path_label(path),
            source,
        })?;
    if bytes.len() > maximum {
        return Err(PresentationPackLoadError::FileTooLarge {
            path: safe_path_label(path),
            maximum,
        });
    }
    String::from_utf8(bytes).map_err(|source| PresentationPackLoadError::InvalidUtf8 {
        path: safe_path_label(path),
        source,
    })
}

fn local_path_error(error: LocalPathError) -> PresentationPackLoadError {
    PresentationPackLoadError::LocalPath(error.to_string())
}

fn safe_path_label(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .flat_map(char::escape_default)
        .take(512)
        .collect()
}
