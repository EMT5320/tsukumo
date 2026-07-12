use serde_json::Value;
use tsukumo_theater::{parse_presentation_pack, PackDocuments, PresentationPackError};

const VALID_MANIFEST: &str = include_str!("support/presentation-pack-valid/pack.json");
const VALID_SCENE: &str = include_str!("support/presentation-pack-valid/scene.json");
const VALID_SPRITE: &str = include_str!("support/presentation-pack-valid/sprite.json");

fn manifest_with(mut edit: impl FnMut(&mut Value)) -> String {
    let mut value: Value = serde_json::from_str(VALID_MANIFEST).expect("valid manifest fixture");
    edit(&mut value);
    serde_json::to_string(&value).expect("serialize manifest fixture")
}

#[test]
fn palette_aliases_and_near_equal_truecolor_values_fail_contrast() {
    // Given: semantic colors that differ numerically yet render identically or almost identically.
    let ansi_black_alias = manifest_with(|value| {
        value["palette"]["colors"][0]["rgb"] = serde_json::json!([0, 0, 0]);
        value["palette"]["colors"][0]["ansi256"] = Value::from(0);
        value["palette"]["colors"][1]["ansi256"] = Value::from(16);
    });
    let ansi_white_alias = manifest_with(|value| {
        value["palette"]["colors"][0]["ansi256"] = Value::from(15);
        value["palette"]["colors"][1]["ansi256"] = Value::from(231);
    });
    let near_truecolor = manifest_with(|value| {
        value["palette"]["colors"][0]["rgb"] = serde_json::json!([0, 0, 0]);
        value["palette"]["colors"][1]["rgb"] = serde_json::json!([0, 0, 1]);
    });

    // When/Then: resolved-color contrast validation rejects every unsafe palette.
    for manifest in [ansi_black_alias, ansi_white_alias, near_truecolor] {
        let documents = PackDocuments::new(&manifest, VALID_SCENE, VALID_SPRITE);
        assert!(matches!(
            parse_presentation_pack(documents),
            Err(PresentationPackError::InvalidModel {
                field: "palette.roles",
                ..
            })
        ));
    }
}

#[test]
fn superscript_dos_device_alias_in_asset_path_is_rejected() {
    // Given: Win32-reserved COM and LPT aliases written with superscript digits.
    for asset in ["COM\u{00b9}.json", "LPT\u{00b2}.json", "com\u{00b3}"] {
        let manifest = manifest_with(|value| {
            value["assets"]["scene"] = Value::from(asset);
        });

        // When/Then: the inert pack boundary rejects the device path before host I/O.
        assert!(matches!(
            parse_presentation_pack(PackDocuments::new(&manifest, VALID_SCENE, VALID_SPRITE,)),
            Err(PresentationPackError::InvalidPath { .. })
        ));
    }
}
