//! Race-focused tests for the local directory capability.

#[cfg(windows)]
#[test]
fn sidecar_inserted_after_tree_validation_fails_before_sqlite_open() {
    // Given: an attacker inserts a hard-linked WAL name after the tree walk.
    let directory = tempfile::tempdir().expect("temporary guarded root");
    let data = directory.path().join("data");
    let outside = directory.path().join("outside-wal");
    std::fs::write(&outside, "outside sentinel").expect("write outside fixture");
    let mut guard = super::LocalDirectoryGuard::prepare(&data).expect("prepare guarded root");
    guard.validate_tree().expect("validate empty tree");
    let wal = data.join("soul.db-wal");
    std::fs::hard_link(&outside, &wal).expect("insert sidecar after validation");

    // When: the fixed sidecar name is atomically opened and validated.
    let error = guard
        .ensure_guarded_file(std::path::Path::new("soul.db-wal"), b"")
        .expect_err("hard-linked sidecar must fail");

    // Then: SQLite is never opened and the outside target remains unchanged.
    assert!(error.to_string().contains("hard-linked"));
    assert_eq!(
        std::fs::read_to_string(outside).expect("read outside fixture"),
        "outside sentinel"
    );
}
