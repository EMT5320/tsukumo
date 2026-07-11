//! Claude CLI command profile with fail-closed permission defaults.

use crate::runtime::{
    PromptDelivery, RuntimeCommandSpec, RuntimeEventDecoder, RuntimeLaunchConfig, RuntimeProfile,
    RuntimeProfileError, RuntimeSafetyCapability,
};
use crate::stream_json::ClaudeStreamDecoder;
use std::ffi::OsString;
use std::fmt;
use tsukumo_kernel::{contains_sensitive_material, RuntimeBinding, RuntimeKind, RuntimeMode};

/// Returns the reviewed C1 fixture consumed by the production decoder.
pub const fn claude_c1_success_fixture() -> &'static str {
    include_str!("../fixtures/claude_c1_success.jsonl")
}

/// Non-interactive permission behavior selected explicitly for Claude CLI.
#[derive(Clone, PartialEq, Eq)]
pub enum ClaudeSafetyMode {
    DenyUnapproved,
    PermissionPromptTool { tool: String },
}

impl fmt::Debug for ClaudeSafetyMode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DenyUnapproved => formatter.write_str("DenyUnapproved"),
            Self::PermissionPromptTool { .. } => formatter
                .debug_struct("PermissionPromptTool")
                .field("tool", &"[REDACTED]")
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClaudeInvocationMode {
    Standard,
    IsolatedSmoke,
}

/// Owned-process Claude profile shared by fake, fixture, and live host paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeRuntimeProfile {
    safety_mode: ClaudeSafetyMode,
    invocation_mode: ClaudeInvocationMode,
}

impl ClaudeRuntimeProfile {
    /// Creates the fail-closed non-interactive profile.
    pub const fn deny_unapproved() -> Self {
        Self {
            safety_mode: ClaudeSafetyMode::DenyUnapproved,
            invocation_mode: ClaudeInvocationMode::Standard,
        }
    }

    /// Creates a context-free, tool-free profile for an explicitly billed smoke.
    pub const fn isolated_smoke() -> Self {
        Self {
            safety_mode: ClaudeSafetyMode::DenyUnapproved,
            invocation_mode: ClaudeInvocationMode::IsolatedSmoke,
        }
    }

    /// Creates the explicit MCP permission callback profile.
    pub fn with_permission_tool(tool: impl Into<String>) -> Result<Self, RuntimeProfileError> {
        let tool = tool.into();
        if tool.trim().is_empty()
            || tool.chars().count() > 256
            || tool.chars().any(char::is_control)
            || contains_sensitive_material(&tool)
        {
            return Err(RuntimeProfileError::InvalidPermissionTool);
        }
        Ok(Self {
            safety_mode: ClaudeSafetyMode::PermissionPromptTool { tool },
            invocation_mode: ClaudeInvocationMode::Standard,
        })
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

impl Default for ClaudeRuntimeProfile {
    fn default() -> Self {
        Self::deny_unapproved()
    }
}

impl RuntimeProfile for ClaudeRuntimeProfile {
    fn binding(&self) -> RuntimeBinding {
        RuntimeBinding::new(RuntimeKind::ClaudeCli, RuntimeMode::OwnedProcess)
    }

    fn command(
        &self,
        launch: &RuntimeLaunchConfig,
    ) -> Result<RuntimeCommandSpec, RuntimeProfileError> {
        let mut args = [
            "-p",
            "--input-format",
            "text",
            "--output-format",
            "stream-json",
            "--verbose",
            "--no-session-persistence",
        ]
        .into_iter()
        .map(OsString::from)
        .collect::<Vec<_>>();
        match self.invocation_mode {
            ClaudeInvocationMode::Standard => {}
            ClaudeInvocationMode::IsolatedSmoke => {
                args.extend(
                    [
                        "--safe-mode",
                        "--tools",
                        "",
                        "--max-budget-usd",
                        "0.05",
                        "--system-prompt",
                        "Return only the exact token requested by the user.",
                        "--no-chrome",
                        "--prompt-suggestions",
                        "false",
                    ]
                    .map(OsString::from),
                );
            }
        }
        match &self.safety_mode {
            ClaudeSafetyMode::DenyUnapproved => {
                args.extend(["--permission-mode", "dontAsk"].map(OsString::from));
            }
            ClaudeSafetyMode::PermissionPromptTool { tool } => {
                args.extend(["--permission-mode", "default"].map(OsString::from));
                args.push(OsString::from("--permission-prompt-tool"));
                args.push(OsString::from(tool));
            }
        }
        RuntimeCommandSpec::new(launch, args, PromptDelivery::Stdin)
    }

    fn decoder(&self) -> Box<dyn RuntimeEventDecoder> {
        Box::new(ClaudeStreamDecoder::new())
    }

    fn safety_capability(&self) -> RuntimeSafetyCapability {
        match self.safety_mode {
            ClaudeSafetyMode::DenyUnapproved => RuntimeSafetyCapability::DenyUnapproved,
            ClaudeSafetyMode::PermissionPromptTool { .. } => {
                RuntimeSafetyCapability::PermissionPromptTool
            }
        }
    }
}
