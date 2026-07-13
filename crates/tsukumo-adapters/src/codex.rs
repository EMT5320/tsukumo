//! Codex CLI command profile with explicit fail-closed safety.

use crate::codex_json::CodexJsonDecoder;
use crate::runtime::{
    PromptDelivery, RuntimeCommandSpec, RuntimeEventDecoder, RuntimeLaunchConfig, RuntimeProfile,
    RuntimeProfileError, RuntimeSafetyCapability,
};
use std::ffi::OsString;
use tsukumo_kernel::{RuntimeBinding, RuntimeKind, RuntimeMode};

/// Returns the reviewed Codex 0.135.0 fixture consumed by the production decoder.
pub const fn codex_0_135_0_success_fixture() -> &'static str {
    include_str!("../fixtures/codex_0_135_0_success.jsonl")
}

/// Returns the reviewed GNU with-state Codex 0.135.0 capture.
pub const fn codex_0_135_0_gnu_with_state_fixture() -> &'static str {
    include_str!("../fixtures/codex_0_135_0_gnu_with_state.jsonl")
}

/// Returns the reviewed GNU without-state Codex 0.135.0 capture.
pub const fn codex_0_135_0_gnu_without_state_fixture() -> &'static str {
    include_str!("../fixtures/codex_0_135_0_gnu_without_state.jsonl")
}

/// Filesystem capability requested from the Codex command sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexSandboxMode {
    ReadOnly,
    WorkspaceWrite,
}

impl CodexSandboxMode {
    const fn argument(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodexInvocationMode {
    Standard,
    IsolatedSmoke,
}

/// Owned-process Codex profile shared by fixture and live Host paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodexRuntimeProfile {
    sandbox: CodexSandboxMode,
    invocation: CodexInvocationMode,
}

impl CodexRuntimeProfile {
    /// Creates the least-capable default profile.
    pub const fn read_only() -> Self {
        Self {
            sandbox: CodexSandboxMode::ReadOnly,
            invocation: CodexInvocationMode::Standard,
        }
    }

    /// Creates the explicit code-editing profile.
    pub const fn workspace_write() -> Self {
        Self {
            sandbox: CodexSandboxMode::WorkspaceWrite,
            invocation: CodexInvocationMode::Standard,
        }
    }

    /// Creates a context-reduced, read-only profile for billed live smoke.
    pub const fn isolated_smoke() -> Self {
        Self {
            sandbox: CodexSandboxMode::ReadOnly,
            invocation: CodexInvocationMode::IsolatedSmoke,
        }
    }

    /// Builds a prompt-free command used by opt-in prerequisite probes.
    pub fn version_command(
        &self,
        launch: &RuntimeLaunchConfig,
    ) -> Result<RuntimeCommandSpec, RuntimeProfileError> {
        RuntimeCommandSpec::new(
            launch,
            vec![OsString::from("--version")],
            PromptDelivery::None,
        )
    }
}

impl Default for CodexRuntimeProfile {
    fn default() -> Self {
        Self::read_only()
    }
}

impl RuntimeProfile for CodexRuntimeProfile {
    fn binding(&self) -> RuntimeBinding {
        RuntimeBinding::new(RuntimeKind::CodexCli, RuntimeMode::OwnedProcess)
    }

    fn command(
        &self,
        launch: &RuntimeLaunchConfig,
    ) -> Result<RuntimeCommandSpec, RuntimeProfileError> {
        let mut args = [
            "exec",
            "--json",
            "--ephemeral",
            "--color",
            "never",
            "--sandbox",
            self.sandbox.argument(),
            "-c",
            "approval_policy=\"never\"",
        ]
        .into_iter()
        .map(OsString::from)
        .collect::<Vec<_>>();
        if self.invocation == CodexInvocationMode::IsolatedSmoke {
            args.extend(
                [
                    "--ignore-user-config",
                    "--skip-git-repo-check",
                    "--disable",
                    "apps",
                    "--disable",
                    "remote_plugin",
                    "--disable",
                    "multi_agent",
                    "--disable",
                    "memories",
                    "-c",
                    "web_search=\"disabled\"",
                ]
                .map(OsString::from),
            );
        }
        args.push(OsString::from("-"));
        RuntimeCommandSpec::new(launch, args, PromptDelivery::Stdin)
    }

    fn decoder(&self) -> Box<dyn RuntimeEventDecoder> {
        Box::new(CodexJsonDecoder::new())
    }

    fn safety_capability(&self) -> RuntimeSafetyCapability {
        RuntimeSafetyCapability::DenyUnapproved
    }
}
