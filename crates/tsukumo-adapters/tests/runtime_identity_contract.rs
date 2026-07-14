//! Prompt-free runtime identity parsing contracts.

use tsukumo_adapters::{
    ClaudeRuntimeProfile, CodexRuntimeProfile, RuntimeProfile, RuntimeProfileError,
};

#[test]
fn reviewed_vendor_version_formats_produce_exact_identity_versions() {
    let claude = ClaudeRuntimeProfile::deny_unapproved();
    let codex = CodexRuntimeProfile::read_only();

    assert_eq!(
        claude
            .parse_version_lines(&["2.1.205 (Claude Code)".into()])
            .expect("parse Claude version"),
        "2.1.205"
    );
    assert_eq!(
        codex
            .parse_version_lines(&["codex-cli 0.135.0".into()])
            .expect("parse Codex version"),
        "0.135.0"
    );
}

#[test]
fn cross_family_ambiguous_and_terminal_unsafe_versions_fail_closed() {
    let claude = ClaudeRuntimeProfile::deny_unapproved();
    let codex = CodexRuntimeProfile::read_only();

    assert_eq!(
        claude.parse_version_lines(&["codex-cli 0.135.0".into()]),
        Err(RuntimeProfileError::InvalidVersionOutput)
    );
    assert_eq!(
        codex.parse_version_lines(&["codex-cli 0.135.0".into(), "codex-cli 0.136.0".into(),]),
        Err(RuntimeProfileError::InvalidVersionOutput)
    );
    let unsafe_line = format!(
        "codex-cli 0.135.0{}",
        char::from_u32(0x202e).expect("valid bidi control scalar")
    );
    assert_eq!(
        codex.parse_version_lines(&[unsafe_line]),
        Err(RuntimeProfileError::InvalidVersionOutput)
    );
}
