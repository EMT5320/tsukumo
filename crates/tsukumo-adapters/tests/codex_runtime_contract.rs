//! Contract tests for the Codex owned-process profile and JSONL normalization.

use std::path::PathBuf;
use tsukumo_adapters::{
    claude_c1_success_fixture, codex_0_135_0_success_fixture, ClaudeRuntimeProfile,
    CodexRuntimeProfile, PromptDelivery, RuntimeEventDecoder, RuntimeLaunchConfig, RuntimeProfile,
    RuntimeSafetyCapability,
};
use tsukumo_kernel::{KernelEventPayload, OutcomeStatus, RuntimeKind, RuntimeMode};

#[test]
fn codex_command_uses_stdin_jsonl_and_fail_closed_sandboxing() {
    // Given: the least-capable Codex owned-process profile.
    let profile = CodexRuntimeProfile::read_only();
    let config = RuntimeLaunchConfig::new(PathBuf::from("codex.cmd"), PathBuf::from("D:/work"));

    // When: the profile constructs the prompt-free process command.
    let spec = profile.command(&config).expect("construct Codex command");
    let arguments = spec
        .args()
        .iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let version = profile
        .version_command(&config)
        .expect("construct Codex version probe");

    // Then: prompts stay on stdin and the process cannot escalate interactively.
    assert_eq!(profile.binding().kind, RuntimeKind::CodexCli);
    assert_eq!(profile.binding().mode, RuntimeMode::OwnedProcess);
    assert_eq!(spec.prompt_delivery(), PromptDelivery::Stdin);
    assert_eq!(
        profile.safety_capability(),
        RuntimeSafetyCapability::DenyUnapproved
    );
    assert_eq!(arguments.first().map(String::as_str), Some("exec"));
    assert_eq!(arguments.last().map(String::as_str), Some("-"));
    assert!(arguments.iter().any(|argument| argument == "--json"));
    assert!(arguments
        .windows(2)
        .any(|pair| pair == ["--sandbox", "read-only"]));
    assert!(arguments
        .windows(2)
        .any(|pair| pair == ["-c", "approval_policy=\"never\""]));
    assert!(!arguments
        .iter()
        .any(|argument| argument.contains("dangerously")));
    assert_eq!(version.prompt_delivery(), PromptDelivery::None);
    assert_eq!(version.args(), ["--version"]);
}

#[test]
fn workspace_and_isolated_profiles_keep_capabilities_explicit() {
    // Given: one editing profile and one context-reduced smoke profile.
    let config = RuntimeLaunchConfig::new(PathBuf::from("codex"), PathBuf::from("D:/work"));

    // When: both command specifications are materialized.
    let editing = command_arguments(&CodexRuntimeProfile::workspace_write(), &config);
    let isolated = command_arguments(&CodexRuntimeProfile::isolated_smoke(), &config);

    // Then: editing is opt-in and the smoke disables ambient extension surfaces.
    assert!(editing
        .windows(2)
        .any(|pair| pair == ["--sandbox", "workspace-write"]));
    assert!(isolated
        .windows(2)
        .any(|pair| pair == ["--sandbox", "read-only"]));
    assert!(isolated
        .iter()
        .any(|argument| argument == "--ignore-user-config"));
    assert!(isolated
        .iter()
        .any(|argument| argument == "--skip-git-repo-check"));
    for feature in ["apps", "remote_plugin", "multi_agent", "memories"] {
        assert!(isolated
            .windows(2)
            .any(|pair| pair[0] == "--disable" && pair[1] == feature));
    }
    assert!(isolated
        .windows(2)
        .any(|pair| pair == ["-c", "web_search=\"disabled\""]));
}

#[test]
fn reviewed_codex_fixture_uses_the_production_decoder() {
    // Given: the sanitized Codex 0.135.0 success fixture.
    let fixture = codex_0_135_0_success_fixture();
    let mut decoder = CodexRuntimeProfile::read_only().decoder();

    // When: every stdout JSONL line is validated and crosses the incremental decoder.
    for line in fixture.lines() {
        serde_json::from_str::<serde_json::Value>(line).expect("fixture line is JSON");
    }
    let payloads = decode_fixture(&mut *decoder, fixture);

    // Then: one paired command and one successful terminal outcome are emitted.
    decoder.finish().expect("fixture has a terminal turn");
    for forbidden in ["C:\\Users\\", "/home/", "auth.json", "SENTINEL", "api_key"] {
        assert!(!fixture.contains(forbidden), "fixture contains {forbidden}");
    }
    assert_eq!(
        payload_kinds(&payloads),
        ["tool_start", "tool_end", "outcome"]
    );
    assert!(matches!(
        payloads.as_slice(),
        [
            KernelEventPayload::ToolStart { vendor_call, .. },
            KernelEventPayload::ToolEnd {
                vendor_call: end_call,
                is_error: false,
                ..
            },
            KernelEventPayload::Outcome {
                status: OutcomeStatus::Succeeded,
                ..
            }
        ] if vendor_call == end_call
            && vendor_call.namespace == "codex_cli"
            && vendor_call.id == "fixture-thread-0-135-0:item_0"
    ));
}

#[test]
fn codex_and_claude_fixtures_share_the_kernel_progress_shape() {
    // Given: one reviewed successful tool episode from each runtime.
    let mut codex = CodexRuntimeProfile::read_only().decoder();
    let mut claude = ClaudeRuntimeProfile::deny_unapproved().decoder();

    // When: both vendor streams are normalized independently.
    let codex_payloads = decode_fixture(&mut *codex, codex_0_135_0_success_fixture());
    let claude_payloads = decode_fixture(&mut *claude, claude_c1_success_fixture());

    // Then: vendor syntax converges on the same progress and terminal variants.
    codex.finish().expect("Codex fixture complete");
    claude.finish().expect("Claude fixture complete");
    assert_eq!(
        payload_kinds(&codex_payloads),
        payload_kinds(&claude_payloads)
    );
    assert_eq!(
        payload_kinds(&codex_payloads),
        ["tool_start", "tool_end", "outcome"]
    );
}

fn command_arguments(profile: &CodexRuntimeProfile, config: &RuntimeLaunchConfig) -> Vec<String> {
    profile
        .command(config)
        .expect("construct Codex command")
        .args()
        .iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect()
}

fn decode_fixture(decoder: &mut dyn RuntimeEventDecoder, fixture: &str) -> Vec<KernelEventPayload> {
    let mut payloads = Vec::new();
    for line in fixture.lines() {
        payloads.extend(
            decoder
                .decode_line(line)
                .expect("decode fixture line")
                .payloads,
        );
    }
    payloads
}

fn payload_kinds(payloads: &[KernelEventPayload]) -> Vec<&'static str> {
    payloads
        .iter()
        .map(|payload| match payload {
            KernelEventPayload::ToolStart { .. } => "tool_start",
            KernelEventPayload::ToolEnd { .. } => "tool_end",
            KernelEventPayload::Outcome { .. } => "outcome",
            _ => "other",
        })
        .collect()
}
