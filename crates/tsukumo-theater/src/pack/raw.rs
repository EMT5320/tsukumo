//! Serde-only boundary shapes converted into the validated pack model.

use super::model::{MonochromeTone, SemanticPose};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawPackManifest {
    pub schema_version: u16,
    pub id: String,
    pub display_name: String,
    pub world: RawWorldPresentation,
    pub companion: RawCompanionPresentation,
    pub terminology: RawTerminology,
    pub line_book: RawLineBook,
    pub palette: RawPalette,
    pub assets: RawPackAssets,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawWorldPresentation {
    pub name: String,
    pub subtitle: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawCompanionPresentation {
    pub actor_id: String,
    pub display_name: String,
    pub romanized_name: String,
    pub title: String,
    pub owner_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawTerminology {
    pub workshop: String,
    pub quest_board: String,
    pub runtime_portal: String,
    pub memory_cabinet: String,
    pub projection_desk: String,
    pub permission_station: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(super) struct RawLineBook {
    pub tool_start: Option<String>,
    pub tool_end_ok: Option<String>,
    pub tool_end_err: Option<String>,
    pub waiting: Option<String>,
    pub outcome: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawPackAssets {
    pub scene: String,
    pub sprite: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawPalette {
    pub colors: Vec<RawPaletteColor>,
    pub roles: RawPaletteRoles,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawPaletteRoles {
    pub ink: u8,
    pub surface: u8,
    pub border: u8,
    pub text_primary: u8,
    pub text_muted: u8,
    pub accent: u8,
    pub urgent: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawPaletteColor {
    pub name: String,
    pub rgb: [u8; 3],
    pub ansi256: u8,
    pub monochrome: MonochromeTone,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawSceneDefinition {
    pub width: u16,
    pub height: u16,
    pub layers: Vec<RawSceneLayer>,
    pub facilities: Vec<RawFacilityDefinition>,
    pub walk_bounds: RawWalkBounds,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawSceneLayer {
    pub id: String,
    pub x: u16,
    pub y: u16,
    pub rows: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawFacilityDefinition {
    pub id: String,
    pub label: String,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawWalkBounds {
    pub min_x: u16,
    pub max_x: u16,
    pub y: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawSpriteAtlas {
    pub frame_width: u16,
    pub frame_height: u16,
    pub frames: Vec<RawSpriteFrame>,
    pub animations: Vec<RawAnimationDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawSpriteFrame {
    pub id: String,
    pub rows: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawAnimationDefinition {
    pub pose: SemanticPose,
    pub frames: Vec<String>,
    pub frame_ticks: u8,
}
