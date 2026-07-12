//! Pack-aware terminal chrome colors with capability fallbacks.

use super::halfblock::{resolve_color, ColorCapability};
use crate::pack::{Palette, PaletteColor};
use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub(super) struct Theme {
    pub ink: Color,
    pub surface: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub urgent: Color,
}

impl Theme {
    pub fn from_palette(palette: &Palette, capability: ColorCapability) -> Self {
        let roles = palette.roles;
        Self {
            ink: resolve(palette.color(roles.ink), capability),
            surface: resolve(palette.color(roles.surface), capability),
            border: resolve(palette.color(roles.border), capability),
            text: resolve(palette.color(roles.text_primary), capability),
            muted: resolve(palette.color(roles.text_muted), capability),
            accent: resolve(palette.color(roles.accent), capability),
            urgent: resolve(palette.color(roles.urgent), capability),
        }
    }

    /// Uses host-fixed high-contrast colors for permission and safety decisions.
    pub const fn safety(capability: ColorCapability) -> Self {
        match capability {
            ColorCapability::TrueColor => Self {
                ink: Color::Rgb(0, 0, 0),
                surface: Color::Rgb(24, 24, 24),
                border: Color::Rgb(255, 255, 255),
                text: Color::Rgb(255, 255, 255),
                muted: Color::Rgb(192, 192, 192),
                accent: Color::Rgb(0, 255, 255),
                urgent: Color::Rgb(255, 255, 0),
            },
            ColorCapability::Ansi256 => Self {
                ink: Color::Indexed(0),
                surface: Color::Indexed(236),
                border: Color::Indexed(15),
                text: Color::Indexed(15),
                muted: Color::Indexed(250),
                accent: Color::Indexed(14),
                urgent: Color::Indexed(11),
            },
            ColorCapability::Monochrome => Self {
                ink: Color::Black,
                surface: Color::Black,
                border: Color::White,
                text: Color::White,
                muted: Color::Gray,
                accent: Color::White,
                urgent: Color::White,
            },
        }
    }
}

const fn resolve(color: &PaletteColor, capability: ColorCapability) -> Color {
    resolve_color(color.rgb, color.ansi256, color.monochrome, capability)
}
