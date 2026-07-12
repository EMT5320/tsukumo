//! Pure pack parsing and cross-document validation.

use super::error::PresentationPackError;
use super::model::{
    CompanionPresentation, PackAssets, PackId, PackManifest, PresentationActorId, PresentationPack,
    Terminology, ValidatedPresentationPack, WorldPresentation, PACK_SCHEMA_VERSION,
};
use super::palette::validate_palette;
use super::raw::{RawPackManifest, RawSceneDefinition, RawSpriteAtlas};
use super::rules::{validate_copy, validate_id, validate_optional_copy, validate_path};
use super::scene::validate_scene;
use super::sprites::validate_sprites;
use super::PackDocuments;
use crate::director::LineBook;
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};

const MAX_PACK_BYTES: usize = 4 * 1024 * 1024;

impl TryFrom<&str> for PresentationActorId {
    type Error = PresentationPackError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(validate_id(value.to_owned(), "actor_id")?))
    }
}

impl<'de> serde::Deserialize<'de> for PresentationActorId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        Self::try_from(value.as_str()).map_err(serde::de::Error::custom)
    }
}

/// Returns the validated relative asset references declared by a manifest.
pub fn presentation_pack_assets(input: &str) -> Result<PackAssets, PresentationPackError> {
    Ok(parse_manifest(input)?.assets)
}

/// Parses three bounded JSON documents into one immutable presentation pack.
pub fn parse_presentation_pack(
    documents: PackDocuments<'_>,
) -> Result<ValidatedPresentationPack, PresentationPackError> {
    let total_bytes = documents
        .manifest
        .len()
        .checked_add(documents.scene.len())
        .and_then(|total| total.checked_add(documents.sprite.len()))
        .ok_or(PresentationPackError::LimitExceeded {
            field: "pack.bytes",
            maximum: MAX_PACK_BYTES,
        })?;
    if total_bytes > MAX_PACK_BYTES {
        return Err(PresentationPackError::LimitExceeded {
            field: "pack.bytes",
            maximum: MAX_PACK_BYTES,
        });
    }

    let manifest = parse_manifest(documents.manifest)?;
    let scene_path = manifest.assets.scene.as_path().to_path_buf();
    let sprite_path = manifest.assets.sprite.as_path().to_path_buf();
    let raw_scene: RawSceneDefinition = parse_json(documents.scene, &scene_path)?;
    let raw_sprites: RawSpriteAtlas = parse_json(documents.sprite, &sprite_path)?;
    let palette_len = manifest.palette.colors.len();
    let scene = validate_scene(raw_scene, palette_len)?;
    let sprites = validate_sprites(raw_sprites, palette_len)?;

    Ok(ValidatedPresentationPack(PresentationPack {
        manifest,
        scene,
        sprites,
    }))
}

#[derive(serde::Deserialize)]
struct SchemaHeader {
    schema_version: u16,
}

fn parse_manifest(input: &str) -> Result<PackManifest, PresentationPackError> {
    let path = Path::new("pack.json");
    let header: SchemaHeader = parse_json(input, path)?;
    if header.schema_version != PACK_SCHEMA_VERSION {
        return Err(PresentationPackError::UnsupportedSchema {
            found: header.schema_version,
        });
    }
    let raw: RawPackManifest = parse_json(input, path)?;
    validate_manifest(raw)
}
fn parse_json<T: DeserializeOwned>(input: &str, path: &Path) -> Result<T, PresentationPackError> {
    serde_json::from_str(input).map_err(|source| PresentationPackError::InvalidJson {
        path: PathBuf::from(path),
        source,
    })
}

fn validate_manifest(raw: RawPackManifest) -> Result<PackManifest, PresentationPackError> {
    let palette = validate_palette(raw.palette)?;
    Ok(PackManifest {
        schema_version: raw.schema_version,
        id: PackId(validate_id(raw.id, "manifest.id")?),
        display_name: validate_copy(raw.display_name, "manifest.display_name")?,
        world: WorldPresentation {
            name: validate_copy(raw.world.name, "manifest.world.name")?,
            subtitle: validate_copy(raw.world.subtitle, "manifest.world.subtitle")?,
        },
        companion: CompanionPresentation {
            actor_id: PresentationActorId(validate_id(
                raw.companion.actor_id,
                "manifest.companion.actor_id",
            )?),
            display_name: validate_copy(
                raw.companion.display_name,
                "manifest.companion.display_name",
            )?,
            romanized_name: validate_copy(
                raw.companion.romanized_name,
                "manifest.companion.romanized_name",
            )?,
            title: validate_copy(raw.companion.title, "manifest.companion.title")?,
            owner_address: validate_copy(
                raw.companion.owner_address,
                "manifest.companion.owner_address",
            )?,
        },
        terminology: Terminology {
            workshop: validate_copy(raw.terminology.workshop, "terminology.workshop")?,
            quest_board: validate_copy(raw.terminology.quest_board, "terminology.quest_board")?,
            runtime_portal: validate_copy(
                raw.terminology.runtime_portal,
                "terminology.runtime_portal",
            )?,
            memory_cabinet: validate_copy(
                raw.terminology.memory_cabinet,
                "terminology.memory_cabinet",
            )?,
            projection_desk: validate_copy(
                raw.terminology.projection_desk,
                "terminology.projection_desk",
            )?,
            permission_station: validate_copy(
                raw.terminology.permission_station,
                "terminology.permission_station",
            )?,
        },
        line_book: LineBook {
            tool_start: validate_optional_copy(raw.line_book.tool_start, "line_book.tool_start")?,
            tool_end_ok: validate_optional_copy(
                raw.line_book.tool_end_ok,
                "line_book.tool_end_ok",
            )?,
            tool_end_err: validate_optional_copy(
                raw.line_book.tool_end_err,
                "line_book.tool_end_err",
            )?,
            waiting: validate_optional_copy(raw.line_book.waiting, "line_book.waiting")?,
            outcome: validate_optional_copy(raw.line_book.outcome, "line_book.outcome")?,
            error: validate_optional_copy(raw.line_book.error, "line_book.error")?,
        },
        palette,
        assets: PackAssets {
            scene: validate_path(raw.assets.scene)?,
            sprite: validate_path(raw.assets.sprite)?,
        },
    })
}
