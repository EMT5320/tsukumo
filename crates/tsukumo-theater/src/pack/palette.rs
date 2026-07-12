//! Semantic palette validation across terminal color capabilities.

use super::error::PresentationPackError;
use super::model::{ContentId, Palette, PaletteColor, PaletteIndex, PaletteRoles};
use super::raw::RawPalette;
use super::rules::{validate_id, MAX_PALETTE_COLORS};
use std::collections::HashSet;

pub(super) fn validate_palette(raw: RawPalette) -> Result<Palette, PresentationPackError> {
    let colors = raw.colors;
    if colors.is_empty() || colors.len() > MAX_PALETTE_COLORS {
        return Err(PresentationPackError::LimitExceeded {
            field: "palette.colors",
            maximum: MAX_PALETTE_COLORS,
        });
    }
    let mut ids = HashSet::with_capacity(colors.len());
    let mut validated = Vec::with_capacity(colors.len());
    for color in colors {
        let id = validate_id(color.name, "palette.colors.name")?;
        if !ids.insert(id.clone()) {
            return Err(PresentationPackError::DuplicateId { id });
        }
        validated.push(PaletteColor {
            name: ContentId(id),
            rgb: color.rgb,
            ansi256: color.ansi256,
            monochrome: color.monochrome,
        });
    }
    let role_index = |index: u8| {
        if usize::from(index) >= validated.len() {
            Err(PresentationPackError::InvalidPaletteIndex { index })
        } else {
            Ok(PaletteIndex(index))
        }
    };
    let roles = PaletteRoles {
        ink: role_index(raw.roles.ink)?,
        surface: role_index(raw.roles.surface)?,
        border: role_index(raw.roles.border)?,
        text_primary: role_index(raw.roles.text_primary)?,
        text_muted: role_index(raw.roles.text_muted)?,
        accent: role_index(raw.roles.accent)?,
        urgent: role_index(raw.roles.urgent)?,
    };
    validate_palette_contrast(&validated, roles)?;
    Ok(Palette {
        colors: validated,
        roles,
    })
}

const MIN_PRESENTATION_CONTRAST_RATIO: f64 = 2.0;

fn validate_palette_contrast(
    colors: &[PaletteColor],
    roles: PaletteRoles,
) -> Result<(), PresentationPackError> {
    let backgrounds = [("ink", roles.ink), ("surface", roles.surface)];
    let foregrounds = [
        ("border", roles.border),
        ("text_primary", roles.text_primary),
        ("text_muted", roles.text_muted),
        ("accent", roles.accent),
        ("urgent", roles.urgent),
    ];
    for (background_name, background) in backgrounds {
        for (foreground_name, foreground) in foregrounds {
            validate_resolved_contrast(
                colors[background.as_usize()].rgb,
                colors[foreground.as_usize()].rgb,
                background_name,
                foreground_name,
                "true_color",
            )?;
            validate_resolved_contrast(
                ansi256_rgb(colors[background.as_usize()].ansi256),
                ansi256_rgb(colors[foreground.as_usize()].ansi256),
                background_name,
                foreground_name,
                "ansi256",
            )?;
            validate_resolved_contrast(
                monochrome_rgb(colors[background.as_usize()].monochrome),
                monochrome_rgb(colors[foreground.as_usize()].monochrome),
                background_name,
                foreground_name,
                "monochrome",
            )?;
        }
    }
    Ok(())
}

fn validate_resolved_contrast(
    background: [u8; 3],
    foreground: [u8; 3],
    background_name: &str,
    foreground_name: &str,
    capability: &str,
) -> Result<(), PresentationPackError> {
    let ratio = contrast_ratio(background, foreground);
    if ratio < MIN_PRESENTATION_CONTRAST_RATIO {
        return Err(PresentationPackError::InvalidModel {
            field: "palette.roles",
            reason: format!(
                "{foreground_name} must have at least {MIN_PRESENTATION_CONTRAST_RATIO:.1}:1 contrast with {background_name} in {capability}"
            ),
        });
    }
    Ok(())
}

fn contrast_ratio(left: [u8; 3], right: [u8; 3]) -> f64 {
    let left = relative_luminance(left);
    let right = relative_luminance(right);
    let (lighter, darker) = if left >= right {
        (left, right)
    } else {
        (right, left)
    };
    (lighter + 0.05) / (darker + 0.05)
}

fn relative_luminance(rgb: [u8; 3]) -> f64 {
    let channel = |value: u8| {
        let value = f64::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(rgb[0]) + 0.7152 * channel(rgb[1]) + 0.0722 * channel(rgb[2])
}

fn ansi256_rgb(index: u8) -> [u8; 3] {
    const ANSI16: [[u8; 3]; 16] = [
        [0, 0, 0],
        [128, 0, 0],
        [0, 128, 0],
        [128, 128, 0],
        [0, 0, 128],
        [128, 0, 128],
        [0, 128, 128],
        [192, 192, 192],
        [128, 128, 128],
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 0],
        [0, 0, 255],
        [255, 0, 255],
        [0, 255, 255],
        [255, 255, 255],
    ];
    match index {
        0..=15 => ANSI16[usize::from(index)],
        16..=231 => {
            const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
            let cube = index - 16;
            [
                LEVELS[usize::from(cube / 36)],
                LEVELS[usize::from((cube % 36) / 6)],
                LEVELS[usize::from(cube % 6)],
            ]
        }
        232..=255 => {
            let level = 8 + (index - 232) * 10;
            [level, level, level]
        }
    }
}

const fn monochrome_rgb(tone: super::model::MonochromeTone) -> [u8; 3] {
    let level = match tone {
        super::model::MonochromeTone::Black => 0,
        super::model::MonochromeTone::DarkGray => 85,
        super::model::MonochromeTone::Gray => 170,
        super::model::MonochromeTone::White => 255,
    };
    [level, level, level]
}
