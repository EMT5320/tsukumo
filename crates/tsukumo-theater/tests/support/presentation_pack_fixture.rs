use serde_json::Value;
use tsukumo_theater::PackDocuments;

pub const VALID_MANIFEST: &str = include_str!("presentation-pack-valid/pack.json");

pub const VALID_SCENE: &str = include_str!("presentation-pack-valid/scene.json");

pub const VALID_SPRITE: &str = include_str!("presentation-pack-valid/sprite.json");

pub fn valid_documents() -> PackDocuments<'static> {
    PackDocuments::new(VALID_MANIFEST, VALID_SCENE, VALID_SPRITE)
}

pub fn manifest_with(mut edit: impl FnMut(&mut Value)) -> String {
    let mut manifest: Value = serde_json::from_str(VALID_MANIFEST).expect("valid manifest fixture");
    edit(&mut manifest);
    serde_json::to_string(&manifest).expect("serialize manifest fixture")
}

pub fn scene_with(mut edit: impl FnMut(&mut Value)) -> String {
    let mut scene: Value = serde_json::from_str(VALID_SCENE).expect("valid scene fixture");
    edit(&mut scene);
    serde_json::to_string(&scene).expect("serialize scene fixture")
}
pub fn sprite_with(mut edit: impl FnMut(&mut Value)) -> String {
    let mut sprite: Value = serde_json::from_str(VALID_SPRITE).expect("valid sprite fixture");
    edit(&mut sprite);
    serde_json::to_string(&sprite).expect("serialize sprite fixture")
}
