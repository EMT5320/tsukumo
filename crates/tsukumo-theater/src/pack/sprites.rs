//! Conversion of sprite JSON into validated frames and semantic animations.

use super::error::PresentationPackError;
use super::model::{AnimationDefinition, ContentId, SemanticPose, SpriteAtlas, SpriteFrame};
use super::raw::RawSpriteAtlas;
use super::rules::{
    decode_grid, validate_id, MAX_FRAMES_PER_POSE, MAX_SPRITE_FRAMES, MAX_SPRITE_HEIGHT,
    MAX_SPRITE_WIDTH,
};
use std::collections::{HashMap, HashSet};

pub(super) fn validate_sprites(
    raw: RawSpriteAtlas,
    palette_len: usize,
) -> Result<SpriteAtlas, PresentationPackError> {
    if raw.frame_width == 0 || raw.frame_width > MAX_SPRITE_WIDTH {
        return Err(PresentationPackError::LimitExceeded {
            field: "sprites.frame_width",
            maximum: usize::from(MAX_SPRITE_WIDTH),
        });
    }
    if raw.frame_height == 0 || raw.frame_height > MAX_SPRITE_HEIGHT {
        return Err(PresentationPackError::LimitExceeded {
            field: "sprites.frame_height",
            maximum: usize::from(MAX_SPRITE_HEIGHT),
        });
    }
    if raw.frames.is_empty() || raw.frames.len() > MAX_SPRITE_FRAMES {
        return Err(PresentationPackError::LimitExceeded {
            field: "sprites.frames",
            maximum: MAX_SPRITE_FRAMES,
        });
    }
    if raw.animations.len() > 5 {
        return Err(PresentationPackError::LimitExceeded {
            field: "sprites.animations",
            maximum: 5,
        });
    }

    let mut frame_lookup = HashMap::with_capacity(raw.frames.len());
    let mut frames = Vec::with_capacity(raw.frames.len());
    for frame in raw.frames {
        let id = validate_id(frame.id, "sprites.frames.id")?;
        if frame_lookup.insert(id.clone(), frames.len()).is_some() {
            return Err(PresentationPackError::DuplicateId { id });
        }
        frames.push(SpriteFrame {
            id: ContentId(id),
            pixels: decode_grid(
                frame.rows,
                Some((raw.frame_width, raw.frame_height)),
                palette_len,
                "sprites.frames.rows",
            )?,
        });
    }

    let mut poses = HashSet::new();
    let mut referenced_frames = HashSet::new();
    let mut animations = Vec::with_capacity(raw.animations.len());
    for animation in raw.animations {
        if !poses.insert(animation.pose) {
            return Err(PresentationPackError::DuplicateId {
                id: format!("pose:{:?}", animation.pose),
            });
        }
        if animation.frames.is_empty() || animation.frames.len() > MAX_FRAMES_PER_POSE {
            return Err(PresentationPackError::LimitExceeded {
                field: "sprites.animations.frames",
                maximum: MAX_FRAMES_PER_POSE,
            });
        }
        if animation.frame_ticks == 0 {
            return Err(PresentationPackError::InvalidModel {
                field: "sprites.animations.frame_ticks",
                reason: "frame_ticks must be greater than zero".into(),
            });
        }
        let frame_indices = animation
            .frames
            .into_iter()
            .map(|id| {
                let id = validate_id(id, "sprites.animations.frames")?;
                let index = frame_lookup.get(&id).copied().ok_or_else(|| {
                    PresentationPackError::InvalidModel {
                        field: "sprites.animations.frames",
                        reason: format!("unknown validated frame id {id}"),
                    }
                })?;
                referenced_frames.insert(index);
                Ok(index)
            })
            .collect::<Result<Vec<_>, PresentationPackError>>()?;
        animations.push(AnimationDefinition {
            pose: animation.pose,
            frame_indices,
            frame_ticks: animation.frame_ticks,
        });
    }

    for required in [
        SemanticPose::Idle,
        SemanticPose::Work,
        SemanticPose::Wait,
        SemanticPose::Urgent,
        SemanticPose::Celebrate,
    ] {
        if !poses.contains(&required) {
            return Err(PresentationPackError::InvalidModel {
                field: "sprites.animations.pose",
                reason: format!("missing required pose {required:?}"),
            });
        }
    }
    if referenced_frames.len() != frames.len() {
        return Err(PresentationPackError::InvalidModel {
            field: "sprites.frames",
            reason: "every frame must be referenced by a semantic animation".into(),
        });
    }

    Ok(SpriteAtlas {
        frame_width: raw.frame_width,
        frame_height: raw.frame_height,
        frames,
        animations,
    })
}
