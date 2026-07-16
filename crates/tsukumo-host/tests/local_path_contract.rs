use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[cfg(windows)]
use std::path::PathBuf;
use tempfile::tempdir;
#[cfg(windows)]
use tsukumo_host::PresentationPackLoadError;
use tsukumo_host::{load_presentation_pack, HostProductController, PresentationPackSource};

#[cfg(windows)]
#[test]
fn remote_or_device_pack_roots_fail_before_filesystem_access() {
    // Given: Windows paths that could trigger SMB, WebDAV, or device I/O.
    let roots = [
        PathBuf::from(r"\\server\share\pack"),
        PathBuf::from(r"\\?\UNC\server\share\pack"),
        PathBuf::from(r"\\.\pipe\tsukumo-pack"),
    ];

    // When/Then: lexical local-only validation rejects each root without opening it.
    for root in roots {
        let error = load_presentation_pack(&PresentationPackSource::Directory(root))
            .expect_err("remote or device root must fail");
        assert!(matches!(error, PresentationPackLoadError::LocalPath(_)));
    }
}

#[cfg(windows)]
#[test]
fn reserved_or_ads_data_paths_fail_before_store_creation() {
    // Given: local-looking paths with a DOS device name or NTFS stream syntax.
    let directory = tempdir().expect("temporary parent");
    let pack =
        load_presentation_pack(&PresentationPackSource::EmbeddedDefault).expect("embedded pack");
    let paths = [
        directory.path().join("NUL"),
        directory.path().join("COM\u{00b9}.txt"),
        directory.path().join("LPT\u{00b2}"),
        directory.path().join("data:stream"),
    ];

    // When/Then: the product boundary rejects both before SQLite or snapshots are created.
    for path in paths {
        let error = HostProductController::open(&path, &pack)
            .err()
            .expect("device-like data path must fail");
        assert!(error.to_string().contains("local path rejected"));
        assert!(!path.exists());
    }
}

#[test]
fn hard_link_inside_data_tree_when_opened_is_rejected() {
    // Given: a product data tree containing a hard link to a file outside that tree.
    let directory = tempdir().expect("temporary data parent");
    let data_path = directory.path().join("product-data");
    fs::create_dir(&data_path).expect("create product data tree");
    let outside = directory.path().join("outside.txt");
    fs::write(&outside, "outside sentinel").expect("write outside fixture");
    fs::hard_link(&outside, data_path.join("linked.txt")).expect("create hard-link fixture");
    let pack =
        load_presentation_pack(&PresentationPackSource::EmbeddedDefault).expect("embedded pack");

    // When: the host preflights the existing data tree.
    let error = HostProductController::open(&data_path, &pack)
        .err()
        .expect("hard-linked data tree must fail");

    // Then: storage remains unopened and the outside file retains its content.
    assert!(error.to_string().contains("local path rejected"));
    assert!(!data_path.join("soul.db").exists());
    assert_eq!(
        fs::read_to_string(outside).expect("read outside fixture"),
        "outside sentinel"
    );
}

#[test]
fn ordinary_local_data_directory_still_opens() {
    // Given: a normal local directory and the embedded inert pack.
    let directory = tempdir().expect("temporary data root");
    let path = directory.path().join("product-data");
    let pack =
        load_presentation_pack(&PresentationPackSource::EmbeddedDefault).expect("embedded pack");

    // When: the host preflights and opens its local authority.
    let controller = HostProductController::open(&path, &pack).expect("open local product");

    // Then: the local store is created without touching any remote source.
    drop(controller);
    assert!(path.join("soul.db").is_file());
}

#[cfg(unix)]
#[test]
fn arbitrary_symlinked_data_ancestor_remains_rejected() {
    let directory = tempdir().expect("temporary symlink parent");
    let real_parent = directory.path().join("real");
    fs::create_dir(&real_parent).expect("create real parent");
    let alias_parent = directory.path().join("alias");
    symlink(&real_parent, &alias_parent).expect("create arbitrary parent alias");
    let data_path = alias_parent.join("product-data");
    let pack =
        load_presentation_pack(&PresentationPackSource::EmbeddedDefault).expect("embedded pack");

    let error = HostProductController::open(&data_path, &pack)
        .err()
        .expect("arbitrary symlink ancestor must fail");

    assert!(error.to_string().contains("local path rejected"));
    assert!(!real_parent.join("product-data/soul.db").exists());
}

#[cfg(windows)]
#[test]
fn opened_data_root_blocks_concurrent_directory_replacement() {
    // Given: the product holds its guarded data root and critical files open.
    let directory = tempdir().expect("temporary data parent");
    let data_path = directory.path().join("guarded-data");
    let moved_path = directory.path().join("replaced-data");
    let pack =
        load_presentation_pack(&PresentationPackSource::EmbeddedDefault).expect("embedded pack");
    let controller = HostProductController::open(&data_path, &pack).expect("open guarded product");

    // When: a concurrent writer tries to replace the validated directory by name.
    let replacement = fs::rename(&data_path, &moved_path);

    // Then: no-delete component handles keep every later path traversal on the same tree.
    assert!(replacement.is_err(), "guarded data root was replaceable");
    assert!(data_path.join("soul.db").is_file());
    drop(controller);
    fs::rename(&data_path, &moved_path).expect("rename after guard release");
}

#[cfg(windows)]
#[test]
fn reparse_component_inside_pack_path_is_rejected_without_following_target() {
    // Given: a local pack root whose sprite directory is a reparse-point alias.
    let directory = tempdir().expect("temporary pack parent");
    let fixture =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/presentation-pack-minimal");
    let pack_root = directory.path().join("pack");
    let outside = directory.path().join("outside");
    fs::create_dir(&pack_root).expect("create pack root");
    fs::create_dir(&outside).expect("create outside target");
    fs::copy(fixture.join("pack.json"), pack_root.join("pack.json")).expect("copy manifest");
    fs::copy(fixture.join("scene.json"), pack_root.join("scene.json")).expect("copy scene");
    let source_sprite = fixture.join("sprites/companion.json");
    fs::copy(source_sprite, outside.join("companion.json")).expect("copy sprite target");
    // PowerShell creates a junction without requiring the symbolic-link privilege.
    let junction = std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "New-Item -ItemType Junction -Path $env:TSUKUMO_LINK -Target $env:TSUKUMO_TARGET | Out-Null",
        ])
        .env("TSUKUMO_LINK", pack_root.join("sprites"))
        .env("TSUKUMO_TARGET", &outside)
        .status()
        .expect("run junction helper");
    assert!(junction.success(), "create reparse fixture");

    // When: the host opens the pack through its component-handle boundary.
    let error = load_presentation_pack(&PresentationPackSource::Directory(pack_root))
        .expect_err("reparse component must fail");

    // Then: the target is never accepted as inert pack content.
    assert!(matches!(error, PresentationPackLoadError::LocalPath(_)));
}
