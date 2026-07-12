//! Conversion of scene JSON into bounded logical-pixel data.

use super::error::PresentationPackError;
use super::model::{ContentId, FacilityDefinition, SceneDefinition, SceneLayer, WalkBounds};
use super::raw::RawSceneDefinition;
use super::rules::{
    decode_grid, validate_copy, validate_id, MAX_SCENE_FACILITIES, MAX_SCENE_HEIGHT,
    MAX_SCENE_LAYERS, MAX_SCENE_WIDTH,
};
use std::collections::HashSet;

const REQUIRED_FACILITY_IDS: [&str; 5] = [
    "quest_board",
    "runtime_portal",
    "memory_cabinet",
    "projection_desk",
    "permission_station",
];

pub(super) fn validate_scene(
    raw: RawSceneDefinition,
    palette_len: usize,
) -> Result<SceneDefinition, PresentationPackError> {
    if raw.width == 0 || raw.width > MAX_SCENE_WIDTH {
        return Err(PresentationPackError::LimitExceeded {
            field: "scene.width",
            maximum: usize::from(MAX_SCENE_WIDTH),
        });
    }
    if raw.height == 0 || raw.height > MAX_SCENE_HEIGHT {
        return Err(PresentationPackError::LimitExceeded {
            field: "scene.height",
            maximum: usize::from(MAX_SCENE_HEIGHT),
        });
    }
    if raw.layers.is_empty() || raw.layers.len() > MAX_SCENE_LAYERS {
        return Err(PresentationPackError::LimitExceeded {
            field: "scene.layers",
            maximum: MAX_SCENE_LAYERS,
        });
    }
    if raw.facilities.len() > MAX_SCENE_FACILITIES {
        return Err(PresentationPackError::LimitExceeded {
            field: "scene.facilities",
            maximum: MAX_SCENE_FACILITIES,
        });
    }

    let mut layer_ids = HashSet::new();
    let mut layers = Vec::with_capacity(raw.layers.len());
    for layer in raw.layers {
        let id = validate_id(layer.id, "scene.layers.id")?;
        if !layer_ids.insert(id.clone()) {
            return Err(PresentationPackError::DuplicateId { id });
        }
        let pixels = decode_grid(layer.rows, None, palette_len, "scene.layers.rows")?;
        ensure_rect_fits(
            layer.x,
            layer.y,
            pixels.width,
            pixels.height,
            raw.width,
            raw.height,
            "scene.layers",
        )?;
        layers.push(SceneLayer {
            id: ContentId(id),
            x: layer.x,
            y: layer.y,
            pixels,
        });
    }

    let mut facility_ids = HashSet::new();
    let mut facilities = Vec::with_capacity(raw.facilities.len());
    for facility in raw.facilities {
        let id = validate_id(facility.id, "scene.facilities.id")?;
        if !facility_ids.insert(id.clone()) {
            return Err(PresentationPackError::DuplicateId { id });
        }
        if facility.x >= raw.width || facility.y >= raw.height {
            return Err(PresentationPackError::InvalidModel {
                field: "scene.facilities",
                reason: "facility anchor lies outside the scene".into(),
            });
        }
        facilities.push(FacilityDefinition {
            id: ContentId(id),
            label: validate_copy(facility.label, "scene.facilities.label")?,
            x: facility.x,
            y: facility.y,
        });
    }

    // Renderer-owned semantic destinations must resolve in every accepted pack.
    for required in REQUIRED_FACILITY_IDS {
        if !facility_ids.contains(required) {
            return Err(PresentationPackError::InvalidModel {
                field: "scene.facilities",
                reason: format!("missing required facility {required}"),
            });
        }
    }
    let walk = raw.walk_bounds;
    if walk.min_x > walk.max_x || walk.max_x >= raw.width || walk.y >= raw.height {
        return Err(PresentationPackError::InvalidModel {
            field: "scene.walk_bounds",
            reason: "walk bounds lie outside the scene".into(),
        });
    }

    Ok(SceneDefinition {
        width: raw.width,
        height: raw.height,
        layers,
        facilities,
        walk_bounds: WalkBounds {
            min_x: walk.min_x,
            max_x: walk.max_x,
            y: walk.y,
        },
    })
}

fn ensure_rect_fits(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    scene_width: u16,
    scene_height: u16,
    field: &'static str,
) -> Result<(), PresentationPackError> {
    let fits = match (x.checked_add(width), y.checked_add(height)) {
        (Some(right), Some(bottom)) => right <= scene_width && bottom <= scene_height,
        (None, _) | (_, None) => false,
    };
    if !fits {
        return Err(PresentationPackError::InvalidModel {
            field,
            reason: "pixel grid lies outside the scene".into(),
        });
    }
    Ok(())
}
