//! Shared validation rules for pack manifests and logical-pixel assets.

use super::error::PresentationPackError;
use super::model::{PaletteIndex, PixelGrid, RelativeAssetPath};
use std::path::{Component, Path, PathBuf};
use tsukumo_kernel::is_terminal_unsafe_character;

pub(super) const MAX_COPY_CHARS: usize = 512;
pub(super) const MAX_ID_CHARS: usize = 64;
pub(super) const MAX_PALETTE_COLORS: usize = 32;
pub(super) const MAX_SCENE_WIDTH: u16 = 120;
pub(super) const MAX_SCENE_HEIGHT: u16 = 60;
pub(super) const MAX_SPRITE_WIDTH: u16 = 32;
pub(super) const MAX_SPRITE_HEIGHT: u16 = 40;
pub(super) const MAX_FRAMES_PER_POSE: usize = 16;
pub(super) const MAX_SPRITE_FRAMES: usize = 80;
pub(super) const MAX_SCENE_LAYERS: usize = 16;
pub(super) const MAX_SCENE_FACILITIES: usize = 32;
const MAX_ASSET_PATH_CHARS: usize = 240;

pub(super) fn validate_id(
    value: String,
    field: &'static str,
) -> Result<String, PresentationPackError> {
    let count = value.chars().count();
    if count == 0 || count > MAX_ID_CHARS {
        return Err(PresentationPackError::InvalidModel {
            field,
            reason: format!("id length must be within 1..={MAX_ID_CHARS}"),
        });
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_".contains(&byte))
    {
        return Err(PresentationPackError::InvalidModel {
            field,
            reason: "id must use lowercase ASCII letters, digits, hyphen, or underscore".into(),
        });
    }
    Ok(value)
}

pub(super) fn validate_copy(
    value: String,
    field: &'static str,
) -> Result<String, PresentationPackError> {
    let count = value.chars().count();
    if count == 0 {
        return Err(PresentationPackError::InvalidModel {
            field,
            reason: "copy cannot be empty".into(),
        });
    }
    if value.chars().any(is_terminal_unsafe_character) {
        return Err(PresentationPackError::InvalidModel {
            field,
            reason: "copy cannot contain terminal control characters".into(),
        });
    }
    if count > MAX_COPY_CHARS {
        return Err(PresentationPackError::LimitExceeded {
            field,
            maximum: MAX_COPY_CHARS,
        });
    }
    Ok(value)
}

pub(super) fn validate_optional_copy(
    value: Option<String>,
    field: &'static str,
) -> Result<Option<String>, PresentationPackError> {
    value.map(|text| validate_copy(text, field)).transpose()
}

pub(super) fn validate_path(value: String) -> Result<RelativeAssetPath, PresentationPackError> {
    let count = value.chars().count();
    if count == 0 || count > MAX_ASSET_PATH_CHARS || value.chars().any(is_terminal_unsafe_character)
    {
        return Err(PresentationPackError::InvalidModel {
            field: "manifest.assets",
            reason: "asset path must be bounded and free of control characters".into(),
        });
    }
    let path = PathBuf::from(value);
    let valid = !path.is_absolute()
        && path.components().all(|component| match component {
            Component::Normal(value) => safe_asset_component(Path::new(value)),
            Component::Prefix(_)
            | Component::RootDir
            | Component::CurDir
            | Component::ParentDir => false,
        });
    if !valid {
        return Err(PresentationPackError::InvalidPath { path });
    }
    Ok(RelativeAssetPath(path))
}

fn safe_asset_component(component: &Path) -> bool {
    let Some(value) = component.to_str() else {
        return false;
    };
    if value.is_empty() || value.contains(':') || value.ends_with('.') || value.ends_with(' ') {
        return false;
    }
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
    !matches!(
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
pub(super) fn decode_grid(
    rows: Vec<String>,
    expected: Option<(u16, u16)>,
    palette_len: usize,
    field: &'static str,
) -> Result<PixelGrid, PresentationPackError> {
    let height = u16::try_from(rows.len()).map_err(|_| PresentationPackError::InvalidModel {
        field,
        reason: "pixel height does not fit u16".into(),
    })?;
    let width = rows.first().map(|row| row.chars().count()).ok_or_else(|| {
        PresentationPackError::InvalidModel {
            field,
            reason: "pixel rows cannot be empty".into(),
        }
    })?;
    let width = u16::try_from(width).map_err(|_| PresentationPackError::InvalidModel {
        field,
        reason: "pixel width does not fit u16".into(),
    })?;
    if rows
        .iter()
        .any(|row| row.chars().count() != usize::from(width))
    {
        return Err(PresentationPackError::InvalidModel {
            field,
            reason: "pixel rows must have equal widths".into(),
        });
    }
    if expected.is_some_and(|size| size != (width, height)) {
        return Err(PresentationPackError::InvalidModel {
            field,
            reason: format!("pixel grid is {width}x{height}, expected {expected:?}"),
        });
    }

    let mut pixels = Vec::with_capacity(usize::from(width) * usize::from(height));
    for row in rows {
        for token in row.chars() {
            pixels.push(decode_pixel(token, palette_len)?);
        }
    }
    Ok(PixelGrid {
        width,
        height,
        pixels,
    })
}

fn decode_pixel(
    token: char,
    palette_len: usize,
) -> Result<Option<PaletteIndex>, PresentationPackError> {
    if token == '.' {
        return Ok(None);
    }
    let digit = token
        .to_digit(36)
        .filter(|_| token.is_ascii_digit() || ('A'..='V').contains(&token))
        .ok_or_else(|| PresentationPackError::InvalidModel {
            field: "pixels",
            reason: format!("unsupported pixel token {token:?}"),
        })?;
    let index = u8::try_from(digit).map_err(|_| PresentationPackError::InvalidModel {
        field: "pixels",
        reason: "palette index does not fit u8".into(),
    })?;
    if usize::from(index) >= palette_len {
        return Err(PresentationPackError::InvalidPaletteIndex { index });
    }
    Ok(Some(PaletteIndex(index)))
}
