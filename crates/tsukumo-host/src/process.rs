//! Owned child-process mechanics with bounded concurrent output.

use crate::config::ProcessLimits;
use crate::process_handle::StandardRuntimeHandle;
use crate::process_reader::{spawn_reader, StreamKind};
use std::fmt;
use std::io::Write;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use thiserror::Error;
use tsukumo_adapters::{PromptDelivery, RuntimeCommandSpec};
use tsukumo_kernel::SensitiveText;

/// One prompt-free command plus its in-memory stdin value and limits.
pub struct ProcessLaunch {
    pub command: RuntimeCommandSpec,
    pub prompt: Option<SensitiveText>,
    pub limits: ProcessLimits,
}

impl ProcessLaunch {
    /// Groups all inputs for one owned-process allocation.
    pub const fn new(
        command: RuntimeCommandSpec,
        prompt: Option<SensitiveText>,
        limits: ProcessLimits,
    ) -> Self {
        Self {
            command,
            prompt,
            limits,
        }
    }
}

impl fmt::Debug for ProcessLaunch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProcessLaunch")
            .field("command", &self.command)
            .field("prompt", &self.prompt.as_ref().map(|_| "[REDACTED]"))
            .field("limits", &self.limits)
            .finish()
    }
}

/// Portable child exit evidence retained after OS reaping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessExit {
    pub code: Option<i32>,
    pub success: bool,
}

impl From<ExitStatus> for ProcessExit {
    fn from(status: ExitStatus) -> Self {
        Self {
            code: status.code(),
            success: status.success(),
        }
    }
}

/// Incremental child output whose diagnostic form never exposes content.
pub enum RuntimeOutput {
    StdoutLine(String),
    StderrLine(String),
    Idle,
    Exited(ProcessExit),
}

impl fmt::Debug for RuntimeOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StdoutLine(_) => formatter.write_str("StdoutLine([REDACTED])"),
            Self::StderrLine(_) => formatter.write_str("StderrLine([REDACTED])"),
            Self::Idle => formatter.write_str("Idle"),
            Self::Exited(exit) => formatter.debug_tuple("Exited").field(exit).finish(),
        }
    }
}

/// Process-tree ownership guaranteed by one runner implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessTreeCapability {
    DirectChildOnly,
    ManagedTree,
}

/// Process allocation port used by real and deterministic fake runners.
pub trait ProcessRunner {
    fn spawn(&self, launch: ProcessLaunch) -> Result<Box<dyn RuntimeHandle>, ProcessError>;

    /// Reports whether cancellation covers descendants or only the direct child.
    fn process_tree_capability(&self) -> ProcessTreeCapability {
        ProcessTreeCapability::DirectChildOnly
    }
}

/// Incremental output and idempotent cleanup port for one child.
pub trait RuntimeHandle: fmt::Debug {
    fn next(&mut self, wait: Duration) -> Result<RuntimeOutput, ProcessError>;
    fn cancel_and_reap(&mut self) -> Result<ProcessExit, ProcessError>;
}

/// Standard-library child runner with piped stdin, stdout, and stderr.
#[derive(Debug, Clone, Copy, Default)]
pub struct StandardProcessRunner;

impl ProcessRunner for StandardProcessRunner {
    fn spawn(&self, launch: ProcessLaunch) -> Result<Box<dyn RuntimeHandle>, ProcessError> {
        let mut command = configured_command(&launch.command);
        let mut child = command.spawn().map_err(ProcessError::Spawn)?;
        let stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                return Err(cleanup_unusable_child(
                    child,
                    ProcessError::MissingPipe { stream: "stdin" },
                ))
            }
        };
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                return Err(cleanup_unusable_child(
                    child,
                    ProcessError::MissingPipe { stream: "stdout" },
                ))
            }
        };
        let stderr = match child.stderr.take() {
            Some(stderr) => stderr,
            None => {
                return Err(cleanup_unusable_child(
                    child,
                    ProcessError::MissingPipe { stream: "stderr" },
                ))
            }
        };
        let (sender, receiver) = mpsc::sync_channel(launch.limits.channel_capacity());
        let readers = vec![
            spawn_reader(stdout, StreamKind::Stdout, launch.limits, sender.clone()),
            spawn_reader(stderr, StreamKind::Stderr, launch.limits, sender),
        ];
        let mut handle = StandardRuntimeHandle::new(child, receiver, readers);
        if let Err(input) = deliver_input(stdin, launch.command.prompt_delivery(), launch.prompt) {
            return match handle.cancel_and_reap() {
                Ok(_) => Err(input),
                Err(cleanup) => Err(ProcessError::InputCleanup {
                    input: Box::new(input),
                    cleanup: Box::new(cleanup),
                }),
            };
        }
        Ok(Box::new(handle))
    }
}

fn configured_command(spec: &RuntimeCommandSpec) -> Command {
    let mut command = Command::new(spec.program());
    command
        .args(spec.args())
        .current_dir(spec.working_directory())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if !spec.inherit_environment() {
        command.env_clear();
    }
    command.envs(
        spec.environment_overrides()
            .iter()
            .map(|(key, value)| (key, value)),
    );
    command
}

fn deliver_input(
    mut stdin: std::process::ChildStdin,
    delivery: PromptDelivery,
    prompt: Option<SensitiveText>,
) -> Result<(), ProcessError> {
    match (delivery, prompt) {
        (PromptDelivery::None, None) => Ok(()),
        (PromptDelivery::None, Some(_)) => Err(ProcessError::UnexpectedPrompt),
        (PromptDelivery::Stdin, None) => Err(ProcessError::MissingPrompt),
        (PromptDelivery::Stdin, Some(prompt)) => {
            stdin
                .write_all(prompt.expose().as_bytes())
                .map_err(ProcessError::WriteStdin)?;
            stdin.flush().map_err(ProcessError::WriteStdin)
        }
    }
}

fn cleanup_unusable_child(mut child: Child, input: ProcessError) -> ProcessError {
    let cleanup = match child.try_wait().map_err(ProcessError::Wait) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => child
            .kill()
            .map_err(ProcessError::Kill)
            .and_then(|()| child.wait().map(|_| ()).map_err(ProcessError::Wait)),
        Err(error) => Err(error),
    };
    match cleanup {
        Ok(()) => input,
        Err(cleanup) => ProcessError::InputCleanup {
            input: Box::new(input),
            cleanup: Box::new(cleanup),
        },
    }
}

/// Sanitized process failures that never retain stdin or output bytes.
#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("failed to spawn runtime process: {0}")]
    Spawn(std::io::Error),
    #[error("runtime process did not expose its {stream} pipe")]
    MissingPipe { stream: &'static str },
    #[error("runtime command does not accept a prompt")]
    UnexpectedPrompt,
    #[error("runtime command requires a prompt")]
    MissingPrompt,
    #[error("failed to write runtime stdin: {0}")]
    WriteStdin(std::io::Error),
    #[error("failed to read runtime {stream}: {source}")]
    Read {
        stream: &'static str,
        source: std::io::Error,
    },
    #[error("runtime {stream} line exceeded {maximum} bytes")]
    LineLimitExceeded {
        stream: &'static str,
        maximum: usize,
    },
    #[error("runtime stderr exceeded {maximum} bytes")]
    StderrLimitExceeded { maximum: usize },
    #[error("runtime {stream} was not valid UTF-8")]
    InvalidUtf8 { stream: &'static str },
    #[error("runtime output wait duration is too large")]
    WaitDurationTooLarge,
    #[error("runtime output channel closed before both readers completed")]
    OutputChannelClosed,
    #[error("runtime reader thread panicked")]
    ReaderThreadPanicked,
    #[error("runtime reader shutdown timed out")]
    ReaderShutdownTimedOut,
    #[error("failed to wait for runtime process: {0}")]
    Wait(std::io::Error),
    #[error("failed to terminate runtime process: {0}")]
    Kill(std::io::Error),
    #[error("runtime input failed and cleanup also failed")]
    InputCleanup {
        input: Box<ProcessError>,
        cleanup: Box<ProcessError>,
    },
}
