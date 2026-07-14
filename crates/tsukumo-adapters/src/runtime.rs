//! Host-facing runtime profile, command, and incremental decoder contracts.

use crate::stream_json::AdapterError;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;
use tsukumo_kernel::{KernelEventPayload, RuntimeBinding};

/// How a host delivers one projection to a runtime process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptDelivery {
    None,
    Stdin,
}

/// Safety capability exposed by one adapter-owned command profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSafetyCapability {
    DenyUnapproved,
    PermissionPromptTool,
}

/// Runtime executable and process-environment inputs supplied by the host.
#[derive(Clone, PartialEq, Eq)]
pub struct RuntimeLaunchConfig {
    pub executable: PathBuf,
    working_directory: PathBuf,
    environment_overrides: Vec<(OsString, OsString)>,
    inherit_environment: bool,
}

impl RuntimeLaunchConfig {
    /// Creates launch inputs without embedding a rendered prompt.
    pub fn new(executable: PathBuf, working_directory: PathBuf) -> Self {
        Self {
            executable,
            working_directory,
            environment_overrides: Vec::new(),
            inherit_environment: true,
        }
    }

    /// Adds one in-memory override whose value stays out of diagnostics.
    pub fn with_environment_override(mut self, key: OsString, value: impl Into<OsString>) -> Self {
        self.environment_overrides.push((key, value.into()));
        self
    }

    /// Disables inherited environment state for deterministic fixture children.
    pub fn without_inherited_environment(mut self) -> Self {
        self.inherit_environment = false;
        self
    }
}

impl fmt::Debug for RuntimeLaunchConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = self
            .environment_overrides
            .iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>();
        formatter
            .debug_struct("RuntimeLaunchConfig")
            .field(
                "executable_configured",
                &!self.executable.as_os_str().is_empty(),
            )
            .field(
                "working_directory_configured",
                &!self.working_directory.as_os_str().is_empty(),
            )
            .field("environment_keys", &keys)
            .field("inherit_environment", &self.inherit_environment)
            .finish()
    }
}

/// Adapter-owned process command with no rendered prompt field.
#[derive(Clone, PartialEq, Eq)]
pub struct RuntimeCommandSpec {
    program: PathBuf,
    args: Vec<OsString>,
    working_directory: PathBuf,
    environment_overrides: Vec<(OsString, OsString)>,
    inherit_environment: bool,
    prompt_delivery: PromptDelivery,
}

impl RuntimeCommandSpec {
    /// Returns the executable selected by the validated profile.
    pub fn program(&self) -> &std::path::Path {
        &self.program
    }

    /// Returns prompt-free command arguments.
    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    /// Returns the redacted-from-diagnostics working directory.
    pub fn working_directory(&self) -> &std::path::Path {
        &self.working_directory
    }

    /// Returns environment overrides whose values remain in memory only.
    pub fn environment_overrides(&self) -> &[(OsString, OsString)] {
        &self.environment_overrides
    }

    /// Reports whether the child inherits the parent environment.
    pub const fn inherit_environment(&self) -> bool {
        self.inherit_environment
    }

    /// Reports the validated prompt delivery channel.
    pub const fn prompt_delivery(&self) -> PromptDelivery {
        self.prompt_delivery
    }
    /// Builds a validated prompt-free command for a runtime or fixture profile.
    pub fn new(
        launch: &RuntimeLaunchConfig,
        args: Vec<OsString>,
        prompt_delivery: PromptDelivery,
    ) -> Result<Self, RuntimeProfileError> {
        validate_launch(launch)?;
        Ok(Self {
            program: launch.executable.clone(),
            args,
            working_directory: launch.working_directory.clone(),
            environment_overrides: launch.environment_overrides.clone(),
            inherit_environment: launch.inherit_environment,
            prompt_delivery,
        })
    }
}

impl fmt::Debug for RuntimeCommandSpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keys = self
            .environment_overrides
            .iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>();
        formatter
            .debug_struct("RuntimeCommandSpec")
            .field("program_configured", &!self.program.as_os_str().is_empty())
            .field("args", &self.args)
            .field("working_directory", &"[REDACTED]")
            .field("environment_keys", &keys)
            .field("inherit_environment", &self.inherit_environment)
            .field("prompt_delivery", &self.prompt_delivery)
            .finish()
    }
}

/// Observable classification for one syntactically valid vendor line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeDisposition {
    Emitted,
    KnownIgnored,
    UnknownSkipped,
}
/// One decoded vendor line with exact incremental position.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedRuntimeLine {
    pub line_number: usize,
    pub disposition: DecodeDisposition,
    pub payloads: Vec<KernelEventPayload>,
}

/// Stateful decoder used identically by fixture and live process streams.
pub trait RuntimeEventDecoder {
    fn decode_line(&mut self, line: &str) -> Result<DecodedRuntimeLine, AdapterError>;
    fn finish(&self) -> Result<(), AdapterError>;
}

/// Adapter-owned runtime command and normalization profile.
pub trait RuntimeProfile {
    fn binding(&self) -> RuntimeBinding;
    fn version_command(
        &self,
        launch: &RuntimeLaunchConfig,
    ) -> Result<RuntimeCommandSpec, RuntimeProfileError>;
    fn parse_version_lines(&self, lines: &[String]) -> Result<String, RuntimeProfileError>;
    fn command(
        &self,
        launch: &RuntimeLaunchConfig,
    ) -> Result<RuntimeCommandSpec, RuntimeProfileError>;
    fn decoder(&self) -> Box<dyn RuntimeEventDecoder>;
    fn safety_capability(&self) -> RuntimeSafetyCapability;
}

/// Invalid runtime profile configuration rejected before process spawn.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RuntimeProfileError {
    #[error("runtime executable is empty")]
    EmptyExecutable,
    #[error("runtime working directory is empty")]
    EmptyWorkingDirectory,
    #[error("runtime environment key is invalid")]
    InvalidEnvironmentKey,
    #[error("permission callback tool name is invalid")]
    InvalidPermissionTool,
    #[error("runtime version output does not match the selected profile")]
    InvalidVersionOutput,
}

pub(crate) fn validate_launch(launch: &RuntimeLaunchConfig) -> Result<(), RuntimeProfileError> {
    if launch.executable.as_os_str().is_empty() {
        return Err(RuntimeProfileError::EmptyExecutable);
    }
    if launch.working_directory.as_os_str().is_empty() {
        return Err(RuntimeProfileError::EmptyWorkingDirectory);
    }
    if launch.environment_overrides.iter().any(|(key, _)| {
        let key = key.to_string_lossy();
        key.is_empty() || key.contains('=') || key.contains('\0')
    }) {
        return Err(RuntimeProfileError::InvalidEnvironmentKey);
    }
    Ok(())
}
