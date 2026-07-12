//! Versioned, inert presentation data for the terminal theater.

mod error;
mod model;
mod palette;
mod raw;
mod rules;
mod scene;
mod sprites;
mod validation;

pub use error::PresentationPackError;
#[cfg(test)]
pub(crate) use model::ContentId;
pub use model::{
    AnimationDefinition, CompanionPresentation, FacilityDefinition, MonochromeTone, PackAssets,
    PackId, PackManifest, Palette, PaletteColor, PaletteIndex, PaletteRoles, PixelGrid,
    PresentationActorId, PresentationPack, RelativeAssetPath, SceneDefinition, SceneLayer,
    SemanticPose, SpriteAtlas, SpriteFrame, Terminology, ValidatedPresentationPack, WalkBounds,
    WorldPresentation, PACK_SCHEMA_VERSION,
};
pub use validation::{parse_presentation_pack, presentation_pack_assets};

/// Raw JSON documents already read and bounded by the host.
#[derive(Debug, Clone, Copy)]
pub struct PackDocuments<'a> {
    pub manifest: &'a str,
    pub scene: &'a str,
    pub sprite: &'a str,
}

impl<'a> PackDocuments<'a> {
    /// Builds one document bundle for the pure parser.
    pub const fn new(manifest: &'a str, scene: &'a str, sprite: &'a str) -> Self {
        Self {
            manifest,
            scene,
            sprite,
        }
    }
}
