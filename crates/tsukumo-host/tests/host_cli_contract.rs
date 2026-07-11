use std::process::Command;

#[test]
fn host_binary_exposes_stable_help_version_and_usage_errors() {
    // Given: the packaged Host binary built by Cargo.
    let binary = env!("CARGO_BIN_EXE_tsukumo-host");

    // When/Then: version and help are stable, prompt-free local operations.
    let version = Command::new(binary)
        .arg("--version")
        .output()
        .expect("run host version");
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).expect("version is UTF-8"),
        format!("tsukumo-host {}\n", env!("CARGO_PKG_VERSION"))
    );

    let help = Command::new(binary)
        .arg("--help")
        .output()
        .expect("run host help");
    assert!(help.status.success());
    let help_text = String::from_utf8(help.stdout).expect("help is UTF-8");
    assert!(help_text.contains("receipt-first runtime composition root"));
    assert!(help_text.contains("TSUKUMO_RUN_LIVE_SMOKE"));

    let invalid = Command::new(binary)
        .arg("--unknown")
        .output()
        .expect("run invalid host command");
    assert_eq!(invalid.status.code(), Some(2));
    assert!(String::from_utf8(invalid.stderr)
        .expect("usage error is UTF-8")
        .contains("unknown argument"));
}
