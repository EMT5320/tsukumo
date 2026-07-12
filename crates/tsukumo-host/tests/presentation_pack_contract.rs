use std::path::PathBuf;
use tsukumo_host::{load_presentation_pack, PresentationPackLoadError, PresentationPackSource};

#[test]
fn embedded_default_when_no_directory_is_selected_loads_shiori() {
    // Given: the bundled source selected without filesystem installation.
    let source = PresentationPackSource::EmbeddedDefault;

    // When: the host loads presentation data before terminal entry.
    let pack = load_presentation_pack(&source).expect("embedded default pack");

    // Then: the approved default identity is available through validated data.
    assert_eq!(pack.manifest().id.as_str(), "default-shiori");
    assert_eq!(pack.companion().actor_id.as_str(), "shiori");
    assert_eq!(pack.companion().display_name, "栞");
}

#[test]
fn external_directory_when_explicitly_selected_changes_presentation() {
    // Given: a valid external fixture with a different companion.
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/presentation-pack-minimal");
    let source = PresentationPackSource::Directory(root);

    // When: the explicit directory is loaded.
    let pack = load_presentation_pack(&source).expect("external presentation pack");

    // Then: no recompilation or durable event change is needed for new presentation data.
    assert_eq!(pack.manifest().id.as_str(), "fixture-clerk");
    assert_eq!(pack.companion().actor_id.as_str(), "fixture-clerk");
    assert_eq!(pack.companion().display_name, "试作书记官");
}

#[test]
fn missing_directory_when_explicitly_selected_does_not_fall_back() {
    // Given: an explicit directory that cannot be opened.
    let source = PresentationPackSource::Directory(PathBuf::from("tests/fixtures/does-not-exist"));

    // When: loading is attempted.
    let error = load_presentation_pack(&source).expect_err("missing explicit pack must fail");

    // Then: the error remains a host I/O failure and the embedded identity is never returned.
    assert!(matches!(error, PresentationPackLoadError::LocalPath(_)));
    assert!(!error.to_string().contains("default-shiori"));
}

#[test]
fn embedded_shiori_when_normalized_preserves_identity_and_distinct_poses() {
    // Given: the approved default Shiori atlas parsed through the production boundary.
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)
        .expect("embedded default pack");
    let frames = &pack.sprites().frames;

    // Then: every semantic frame retains the side braid, flame, and wax accent anchors.
    for frame in frames {
        let color_name = |x, y| {
            frame
                .pixels
                .pixel(x, y)
                .map(|index| pack.palette().color(index).name.as_str())
        };
        assert_eq!(
            color_name(1, 10),
            Some("silver"),
            "frame {}",
            frame.id.as_str()
        );
        assert_eq!(
            color_name(13, 0),
            Some("spirit_cyan"),
            "frame {}",
            frame.id.as_str()
        );
        assert_eq!(
            color_name(7, 9),
            Some("vermilion"),
            "frame {}",
            frame.id.as_str()
        );
    }

    // And: all five large-shape contracts remain independently identifiable.
    for left in 0..frames.len() {
        for right in left + 1..frames.len() {
            assert_ne!(
                frames[left].pixels,
                frames[right].pixels,
                "frames {} and {} must differ",
                frames[left].id.as_str(),
                frames[right].id.as_str()
            );
        }
    }
}
