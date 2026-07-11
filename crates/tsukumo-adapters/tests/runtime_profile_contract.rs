//! Contract tests for Claude command safety and incremental runtime decoding.

use std::ffi::OsString;
use std::path::PathBuf;
use tsukumo_adapters::{
    claude_c1_success_fixture, AdapterError, ClaudeRuntimeProfile, ClaudeSafetyMode,
    DecodeDisposition, RuntimeLaunchConfig, RuntimeProfile, RuntimeSafetyCapability,
};
use tsukumo_kernel::{KernelEventPayload, OutcomeStatus, RuntimeKind, RuntimeMode};

const ENV_SECRET: &str = "SENTINEL-host-profile-secret";

#[test]
fn claude_command_keeps_prompt_and_environment_values_out_of_diagnostics() {
    // Given: a locked-down Claude profile and a secret-bearing environment override.
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let config =
        RuntimeLaunchConfig::new(PathBuf::from("claude"), PathBuf::from("D:/work/tsukumo"))
            .with_environment_override(OsString::from("CLAUDE_PROFILE_TOKEN"), ENV_SECRET);

    // When: the adapter constructs the owned-process command.
    let spec = profile.command(&config).expect("construct Claude command");
    let arguments = spec
        .args()
        .iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let diagnostic = format!("{config:?} {spec:?}");

    // Then: stdin is the prompt channel and the command uses fail-closed permissions.
    assert_eq!(profile.binding().kind, RuntimeKind::ClaudeCli);
    assert_eq!(profile.binding().mode, RuntimeMode::OwnedProcess);
    assert_eq!(
        profile.safety_capability(),
        RuntimeSafetyCapability::DenyUnapproved
    );
    assert!(arguments
        .windows(2)
        .any(|pair| pair == ["--output-format", "stream-json"]));
    assert!(arguments
        .windows(2)
        .any(|pair| pair == ["--permission-mode", "dontAsk"]));
    assert!(arguments.iter().any(|argument| argument == "--verbose"));
    assert!(!arguments
        .iter()
        .any(|argument| argument.contains("dangerously-skip-permissions")));
    assert!(!diagnostic.contains(ENV_SECRET));
    assert!(diagnostic.contains("CLAUDE_PROFILE_TOKEN"));
}

#[test]
fn isolated_smoke_profile_disables_workspace_context_tools_and_unbounded_cost() {
    // Given: the dedicated profile for an externally billed live smoke.
    let profile = ClaudeRuntimeProfile::isolated_smoke();
    let config = RuntimeLaunchConfig::new(PathBuf::from("claude"), PathBuf::from("D:/empty"));

    // When: the adapter constructs the reviewed smoke command.
    let spec = profile
        .command(&config)
        .expect("construct isolated smoke command");
    let arguments = spec
        .args()
        .iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    // Then: custom project context and tools are disabled under a hard cost cap.
    assert!(arguments.iter().any(|argument| argument == "--safe-mode"));
    assert!(arguments.windows(2).any(|pair| pair == ["--tools", ""]));
    assert!(arguments
        .windows(2)
        .any(|pair| pair == ["--max-budget-usd", "0.05"]));
    assert!(arguments.windows(2).any(|pair| {
        pair == [
            "--system-prompt",
            "Return only the exact token requested by the user.",
        ]
    }));
    assert!(arguments.iter().any(|argument| argument == "--no-chrome"));
    assert!(arguments
        .windows(2)
        .any(|pair| pair == ["--prompt-suggestions", "false"]));
    assert!(arguments
        .windows(2)
        .any(|pair| pair == ["--permission-mode", "dontAsk"]));
}

#[test]
fn permission_tool_profile_is_explicit_and_rejects_sensitive_tool_names() {
    // Given: an explicitly configured permission callback tool.
    let profile = ClaudeRuntimeProfile::with_permission_tool("mcp__tsukumo__permission")
        .expect("construct permission-tool profile");
    let config = RuntimeLaunchConfig::new(PathBuf::from("claude"), PathBuf::from("."));

    // When: the profile creates its command specification.
    let spec = profile
        .command(&config)
        .expect("construct callback command");
    let arguments = spec
        .args()
        .iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    // Then: callback capability is explicit and dangerous bypass remains absent.
    assert_eq!(
        profile.safety_capability(),
        RuntimeSafetyCapability::PermissionPromptTool
    );
    assert!(arguments
        .windows(2)
        .any(|pair| pair == ["--permission-prompt-tool", "mcp__tsukumo__permission"]));
    assert!(matches!(
        ClaudeRuntimeProfile::with_permission_tool("token=secret-value"),
        Err(tsukumo_adapters::RuntimeProfileError::InvalidPermissionTool)
    ));
}

#[test]
fn stateful_decoder_emits_before_finish_and_requires_one_terminal_result() {
    // Given: one ignored init line, an incremental tool start, and a terminal result.
    let mut decoder = ClaudeRuntimeProfile::deny_unapproved().decoder();
    let init = r#"{"type":"system","subtype":"init"}"#;
    let tool = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"toolu_1","name":"Read","input":{"path":"DESIGN.md"}}]}}"#;
    let result = r#"{"type":"result","subtype":"success","is_error":false,"result":"done"}"#;

    // When: lines are decoded one at a time before the stream is complete.
    let ignored = decoder.decode_line(init).expect("decode init line");
    let emitted = decoder.decode_line(tool).expect("decode tool line");
    let early_finish = decoder.finish().expect_err("missing result must truncate");
    let terminal = decoder.decode_line(result).expect("decode result line");

    // Then: each line is observable and exactly one terminal outcome closes the stream.
    assert!(ignored.payloads.is_empty());
    assert_eq!(ignored.disposition, DecodeDisposition::KnownIgnored);
    assert_eq!(ignored.line_number, 1);
    assert!(matches!(
        emitted.payloads.as_slice(),
        [KernelEventPayload::ToolStart { tool, .. }] if tool == "Read"
    ));
    assert_eq!(emitted.line_number, 2);
    assert_eq!(emitted.disposition, DecodeDisposition::Emitted);
    assert!(matches!(
        early_finish,
        AdapterError::TruncatedStream { lines: 2 }
    ));
    assert!(matches!(
        terminal.payloads.as_slice(),
        [KernelEventPayload::Outcome {
            status: OutcomeStatus::Succeeded,
            ..
        }]
    ));
    decoder.finish().expect("terminal result closes stream");
    assert!(matches!(
        decoder.decode_line(result),
        Err(AdapterError::DuplicateTerminal { line: 4 })
    ));
}

#[test]
fn safety_mode_debug_never_exposes_callback_values() {
    // Given: both supported profile variants.
    let denied = ClaudeSafetyMode::DenyUnapproved;
    let callback = ClaudeSafetyMode::PermissionPromptTool {
        tool: "mcp__tsukumo__permission".into(),
    };

    // When/Then: diagnostics expose capability shape without credential material.
    assert_eq!(format!("{denied:?}"), "DenyUnapproved");
    assert!(!format!("{callback:?}").contains("mcp__tsukumo__permission"));
}

#[test]
fn reviewed_fixture_uses_the_same_stateful_decoder_contract() {
    // Given: the checked-in C1 Claude JSONL fixture.
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let mut decoder = profile.decoder();
    let mut payloads = Vec::new();

    // When: every recorded line crosses the live incremental decoder.
    for line in claude_c1_success_fixture().lines() {
        payloads.extend(
            decoder
                .decode_line(line)
                .expect("decode fixture line")
                .payloads,
        );
    }

    // Then: fixture completion and tool progress satisfy the production stream contract.
    decoder.finish().expect("fixture has one terminal result");
    assert!(payloads.iter().any(
        |payload| matches!(payload, KernelEventPayload::ToolStart { tool, .. } if tool == "Read")
    ));
    assert!(matches!(
        payloads.last(),
        Some(KernelEventPayload::Outcome {
            status: OutcomeStatus::Succeeded,
            ..
        })
    ));
}

#[test]
fn unknown_vendor_events_are_observable_without_fabricated_payloads() {
    // Given: a valid future vendor event outside the documented C1 set.
    let mut decoder = ClaudeRuntimeProfile::deny_unapproved().decoder();

    // When: the incremental decoder encounters that compatibility event.
    let decoded = decoder
        .decode_line(r#"{"type":"future_vendor_event","value":1}"#)
        .expect("skip future event");

    // Then: Host can count the skip while no durable product fact is invented.
    assert!(decoded.payloads.is_empty());
    assert_eq!(decoded.disposition, DecodeDisposition::UnknownSkipped);
}
