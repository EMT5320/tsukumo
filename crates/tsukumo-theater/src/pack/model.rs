//! Validated presentation-pack model consumed by the theater.

use crate::director::LineBook;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const PACK_SCHEMA_VERSION: u16 = 1;

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(pub(crate) String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

string_id!(PackId);
string_id!(PresentationActorId);
string_id!(ContentId);

/// A path proven to be relative and unable to escape a pack root.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct RelativeAssetPath(pub(crate) PathBuf);

impl RelativeAssetPath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackManifest {
    pub schema_version: u16,
    pub id: PackId,
    pub display_name: String,
    pub world: WorldPresentation,
    pub companion: CompanionPresentation,
    pub terminology: Terminology,
    pub line_book: LineBook,
    pub palette: Palette,
    pub assets: PackAssets,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldPresentation {
    pub name: String,
    pub subtitle: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompanionPresentation {
    pub actor_id: PresentationActorId,
    pub display_name: String,
    pub romanized_name: String,
    pub title: String,
    pub owner_address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Terminology {
    pub workshop: String,
    pub quest_board: String,
    pub runtime_portal: String,
    pub memory_cabinet: String,
    pub projection_desk: String,
    pub permission_station: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PackAssets {
    pub scene: RelativeAssetPath,
    pub sprite: RelativeAssetPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonochromeTone {
    Black,
    DarkGray,
    Gray,
    White,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PaletteColor {
    pub name: ContentId,
    pub rgb: [u8; 3],
    pub ansi256: u8,
    pub monochrome: MonochromeTone,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Palette {
    pub colors: Vec<PaletteColor>,
    pub roles: PaletteRoles,
}

/// Required semantic colors used by product chrome across every pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PaletteRoles {
    pub ink: PaletteIndex,
    pub surface: PaletteIndex,
    pub border: PaletteIndex,
    pub text_primary: PaletteIndex,
    pub text_muted: PaletteIndex,
    pub accent: PaletteIndex,
    pub urgent: PaletteIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct PaletteIndex(pub(crate) u8);

impl PaletteIndex {
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl Palette {
    /// Resolves one already-validated semantic palette index.
    pub fn color(&self, index: PaletteIndex) -> &PaletteColor {
        &self.colors[index.as_usize()]
    }
}

/// Dense row-major logical pixels. A missing value is transparent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PixelGrid {
    pub width: u16,
    pub height: u16,
    pub(crate) pixels: Vec<Option<PaletteIndex>>,
}

impl PixelGrid {
    pub fn pixel(&self, x: u16, y: u16) -> Option<PaletteIndex> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.pixels[usize::from(y) * usize::from(self.width) + usize::from(x)]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SceneLayer {
    pub id: ContentId,
    pub x: u16,
    pub y: u16,
    pub pixels: PixelGrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FacilityDefinition {
    pub id: ContentId,
    pub label: String,
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WalkBounds {
    pub min_x: u16,
    pub max_x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SceneDefinition {
    pub width: u16,
    pub height: u16,
    pub layers: Vec<SceneLayer>,
    pub facilities: Vec<FacilityDefinition>,
    pub walk_bounds: WalkBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPose {
    Idle,
    Work,
    Wait,
    Urgent,
    Celebrate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpriteFrame {
    pub id: ContentId,
    pub pixels: PixelGrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnimationDefinition {
    pub pose: SemanticPose,
    pub frame_indices: Vec<usize>,
    pub frame_ticks: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpriteAtlas {
    pub frame_width: u16,
    pub frame_height: u16,
    pub frames: Vec<SpriteFrame>,
    pub animations: Vec<AnimationDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PresentationPack {
    pub(crate) manifest: PackManifest,
    pub(crate) scene: SceneDefinition,
    pub(crate) sprites: SpriteAtlas,
}

/// Presentation data that has passed schema, limit, and reference checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ValidatedPresentationPack(pub(crate) PresentationPack);

impl ValidatedPresentationPack {
    pub fn manifest(&self) -> &PackManifest {
        &self.0.manifest
    }

    pub fn companion(&self) -> &CompanionPresentation {
        &self.0.manifest.companion
    }

    pub fn line_book(&self) -> &LineBook {
        &self.0.manifest.line_book
    }

    pub fn palette(&self) -> &Palette {
        &self.0.manifest.palette
    }

    pub fn scene(&self) -> &SceneDefinition {
        &self.0.scene
    }

    pub fn sprites(&self) -> &SpriteAtlas {
        &self.0.sprites
    }
}
