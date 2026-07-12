use serde_json::Value;
use tsukumo_theater::{
    parse_presentation_pack, PackDocuments, PresentationPackError, PACK_SCHEMA_VERSION,
};

#[path = "support/presentation_pack_fixture.rs"]
mod fixture;
use fixture::*;

#[test]
fn valid_pack_when_documents_match_returns_typed_presentation() {
    // Given: a schema-v1 manifest and matching scene and sprite documents.
    let documents = valid_documents();

    // When: the pure theater boundary parses and validates the documents.
    let pack = parse_presentation_pack(documents).expect("valid pack");

    // Then: presentation identity and asset references are available as typed data.
    assert_eq!(pack.manifest().schema_version, PACK_SCHEMA_VERSION);
    assert_eq!(pack.manifest().id.as_str(), "default-shiori");
    assert_eq!(pack.companion().actor_id.as_str(), "shiori");
    assert_eq!(pack.scene().width, 8);
    assert_eq!(pack.sprites().frames.len(), 5);
}

#[test]
fn unsupported_schema_when_manifest_is_newer_returns_typed_error() {
    // Given: a structurally valid manifest with an unsupported schema version.
    let manifest = manifest_with(|value| value["schema_version"] = Value::from(99));
    let documents = PackDocuments::new(&manifest, VALID_SCENE, VALID_SPRITE);

    // When: the pack is parsed.
    let error = parse_presentation_pack(documents).expect_err("unsupported schema must fail");

    // Then: callers can distinguish version drift from malformed JSON.
    assert!(matches!(
        error,
        PresentationPackError::UnsupportedSchema { found: 99 }
    ));
}

#[test]
fn parent_path_when_manifest_references_asset_returns_invalid_path() {
    // Given: a manifest that attempts to escape the selected pack root.
    let manifest = manifest_with(|value| {
        value["assets"]["sprite"] = Value::from("../secrets.json");
    });
    let documents = PackDocuments::new(&manifest, VALID_SCENE, VALID_SPRITE);

    // When: the pack is parsed.
    let error = parse_presentation_pack(documents).expect_err("parent path must fail");

    // Then: the path violation remains a typed boundary error.
    assert!(matches!(error, PresentationPackError::InvalidPath { .. }));
}

#[test]
fn palette_index_when_scene_uses_missing_color_returns_typed_error() {
    // Given: a scene layer that references palette entry 3 while only 0..=2 exist.
    let scene = scene_with(|value| {
        value["layers"][0]["rows"][0] = Value::from("33333333");
    });
    let documents = PackDocuments::new(VALID_MANIFEST, &scene, VALID_SPRITE);

    // When: cross-document validation runs.
    let error = parse_presentation_pack(documents).expect_err("palette index must fail");

    // Then: the invalid index is reported without filesystem or renderer access.
    assert!(matches!(
        error,
        PresentationPackError::InvalidPaletteIndex { index: 3 }
    ));
}

#[test]
fn semantic_palette_role_when_out_of_range_returns_typed_error() {
    // Given: a chrome role references a palette entry that does not exist.
    let manifest = manifest_with(|value| value["palette"]["roles"]["accent"] = Value::from(9));
    let documents = PackDocuments::new(&manifest, VALID_SCENE, VALID_SPRITE);

    // When: the semantic theme is validated.
    let error = parse_presentation_pack(documents).expect_err("invalid role index must fail");

    // Then: renderers never receive an unchecked role index.
    assert!(matches!(
        error,
        PresentationPackError::InvalidPaletteIndex { index: 9 }
    ));
}

#[test]
fn text_role_when_equal_to_backdrop_is_rejected() {
    // Given: primary text is assigned to the same palette entry as the backdrop.
    let manifest = manifest_with(|value| {
        value["palette"]["roles"]["text_primary"] = Value::from(0);
    });
    let documents = PackDocuments::new(&manifest, VALID_SCENE, VALID_SPRITE);

    // When: the semantic theme is validated.
    let error = parse_presentation_pack(documents).expect_err("unreadable text role must fail");

    // Then: an external pack cannot silently erase product facts.
    assert!(matches!(
        error,
        PresentationPackError::InvalidModel {
            field: "palette.roles",
            ..
        }
    ));
}

#[test]
fn control_character_when_present_in_copy_is_rejected() {
    // Given: pack copy contains an ANSI terminal control sequence.
    let manifest = manifest_with(|value| {
        value["display_name"] = Value::from("unsafe\u{1b}[2Jcopy");
    });
    let documents = PackDocuments::new(&manifest, VALID_SCENE, VALID_SPRITE);

    // When: the inert presentation boundary validates the copy.
    let error = parse_presentation_pack(documents).expect_err("control copy must fail");

    // Then: external content cannot issue terminal control sequences.
    assert!(matches!(
        error,
        PresentationPackError::InvalidModel {
            field: "manifest.display_name",
            ..
        }
    ));
}

#[test]
fn required_facility_when_missing_is_rejected() {
    // Given: an otherwise valid scene omits the contract-station anchor.
    let scene = scene_with(|value| {
        value["facilities"]
            .as_array_mut()
            .expect("facility array")
            .retain(|facility| facility["id"] != "permission_station");
    });
    let documents = PackDocuments::new(VALID_MANIFEST, &scene, VALID_SPRITE);

    // When: semantic scene validation runs.
    let error = parse_presentation_pack(documents).expect_err("missing facility must fail");

    // Then: renderers never receive an unresolved semantic destination.
    assert!(matches!(
        error,
        PresentationPackError::InvalidModel {
            field: "scene.facilities",
            ..
        }
    ));
}
#[test]
fn duplicate_sprite_frame_when_parsed_returns_duplicate_id() {
    // Given: two deterministic sprite frames share one identifier.
    let sprite = sprite_with(|value| {
        let duplicate = value["frames"][0].clone();
        value["frames"]
            .as_array_mut()
            .expect("frames array")
            .push(duplicate);
    });
    let documents = PackDocuments::new(VALID_MANIFEST, VALID_SCENE, &sprite);

    // When: the cross-file pack is validated.
    let error = parse_presentation_pack(documents).expect_err("duplicate frame must fail");

    // Then: frame lookup remains deterministic.
    assert!(matches!(error, PresentationPackError::DuplicateId { .. }));
}

#[test]
fn malformed_manifest_when_parsed_identifies_pack_document() {
    // Given: a malformed manifest with otherwise valid assets.
    let documents = PackDocuments::new("{", VALID_SCENE, VALID_SPRITE);

    // When: the pack parser reads the boundary.
    let error = parse_presentation_pack(documents).expect_err("malformed JSON must fail");

    // Then: the actionable error retains the failing document path.
    assert!(matches!(
        error,
        PresentationPackError::InvalidJson { ref path, .. } if path.ends_with("pack.json")
    ));
}

#[test]
fn structurally_newer_manifest_returns_unsupported_schema_before_shape_errors() {
    // Given: a future manifest whose only stable field is its schema header.
    let documents = PackDocuments::new(r#"{"schema_version":99}"#, VALID_SCENE, VALID_SPRITE);

    // When: the schema header is inspected before the v1 body.
    let error = parse_presentation_pack(documents).expect_err("future schema must fail early");

    // Then: callers receive version drift instead of a misleading v1 JSON error.
    assert!(matches!(
        error,
        PresentationPackError::UnsupportedSchema { found: 99 }
    ));
}

#[test]
fn semantic_colors_with_distinct_indices_but_equal_values_are_rejected() {
    // Given: each capability has a foreground that resolves to the backdrop value.
    let mut cases = Vec::new();
    cases.push(manifest_with(|value| {
        value["palette"]["colors"]
            .as_array_mut()
            .expect("palette colors")
            .push(serde_json::json!({
                "name": "duplicate_ink",
                "rgb": [1, 6, 12],
                "ansi256": 232,
                "monochrome": "black"
            }));
        value["palette"]["roles"]["text_primary"] = Value::from(3);
    }));
    cases.push(manifest_with(|value| {
        value["palette"]["colors"][2]["ansi256"] = Value::from(232);
    }));
    cases.push(manifest_with(|value| {
        value["palette"]["colors"][1]["monochrome"] = Value::from("black");
    }));

    // When/Then: truecolor, ANSI-256, and monochrome collisions all fail closed.
    for manifest in cases {
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
fn terminal_format_character_in_copy_or_path_is_rejected() {
    // Given: invisible bidi and zero-width controls in presentation inputs.
    let copy = manifest_with(|value| {
        value["display_name"] = Value::from("safe\u{202e}txt");
    });
    let path = manifest_with(|value| {
        value["assets"]["scene"] = Value::from("scene\u{200b}.json");
    });

    // When/Then: neither input reaches terminal copy or host path handling.
    for manifest in [copy, path] {
        let documents = PackDocuments::new(&manifest, VALID_SCENE, VALID_SPRITE);
        assert!(matches!(
            parse_presentation_pack(documents),
            Err(PresentationPackError::InvalidModel { .. })
        ));
    }
}

#[test]
fn excessive_scene_layers_and_unreferenced_frames_are_rejected() {
    // Given: valid-looking data that expands aggregate model counts.
    let scene = scene_with(|value| {
        let template = value["layers"][0].clone();
        let layers = value["layers"].as_array_mut().expect("layers");
        for index in 1..17 {
            let mut layer = template.clone();
            layer["id"] = Value::from(format!("layer_{index}"));
            layers.push(layer);
        }
    });
    let sprite = sprite_with(|value| {
        value["frames"]
            .as_array_mut()
            .expect("frames")
            .push(serde_json::json!({
                "id": "unused_0",
                "rows": [".00.", "0110", "0220", ".00."]
            }));
    });

    // When/Then: count inflation and inert retained frames fail explicitly.
    assert!(matches!(
        parse_presentation_pack(PackDocuments::new(VALID_MANIFEST, &scene, VALID_SPRITE)),
        Err(PresentationPackError::LimitExceeded {
            field: "scene.layers",
            ..
        })
    ));
    assert!(matches!(
        parse_presentation_pack(PackDocuments::new(VALID_MANIFEST, VALID_SCENE, &sprite)),
        Err(PresentationPackError::InvalidModel {
            field: "sprites.frames",
            ..
        })
    ));
}
