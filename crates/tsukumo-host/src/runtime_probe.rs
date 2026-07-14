//! Prompt-free runtime identity preflight for reviewed episode execution.

use crate::{ProcessLaunch, ProcessLimits, ProcessRunner, RuntimeOutput, StandardProcessRunner};
use std::time::{Duration, Instant};
use thiserror::Error;
use tsukumo_adapters::{RuntimeLaunchConfig, RuntimeProfile, RuntimeProfileError};
use tsukumo_kernel::RuntimeBinding;

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const PROBE_POLL: Duration = Duration::from_millis(50);
const MAX_PROBE_OUTPUT_CHARS: usize = 4_096;

/// Runtime identity observed from the selected executable without delivering a prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeIdentity {
    pub binding: RuntimeBinding,
    pub version: String,
}

/// Prompt-free identity probe kept separate from the prompt-bearing execution runner.
pub trait RuntimeProbe {
    fn probe(
        &self,
        profile: &dyn RuntimeProfile,
        launch: &RuntimeLaunchConfig,
    ) -> Result<RuntimeIdentity, RuntimeProbeError>;
}

/// Production prompt-free probe using the standard owned-process runner.
#[derive(Debug, Clone, Copy, Default)]
pub struct StandardRuntimeProbe;

impl RuntimeProbe for StandardRuntimeProbe {
    fn probe(
        &self,
        profile: &dyn RuntimeProfile,
        launch: &RuntimeLaunchConfig,
    ) -> Result<RuntimeIdentity, RuntimeProbeError> {
        probe_with_runner(&StandardProcessRunner, profile, launch)
    }
}

/// Sanitized probe failures that never retain executable paths or raw version output.
#[derive(Debug, Error)]
pub enum RuntimeProbeError {
    #[error(transparent)]
    Profile(#[from] RuntimeProfileError),
    #[error("failed to start the prompt-free runtime identity probe")]
    Spawn,
    #[error("failed while reading the prompt-free runtime identity probe")]
    Read,
    #[error("failed to clean up the prompt-free runtime identity probe")]
    Cleanup,
    #[error("prompt-free runtime identity probe timed out")]
    TimedOut,
    #[error("prompt-free runtime identity probe output exceeded its bound")]
    OutputTooLarge,
    #[error("prompt-free runtime identity probe exited unsuccessfully")]
    NonZeroExit,
}

fn probe_with_runner(
    runner: &dyn ProcessRunner,
    profile: &dyn RuntimeProfile,
    launch: &RuntimeLaunchConfig,
) -> Result<RuntimeIdentity, RuntimeProbeError> {
    let command = profile.version_command(launch)?;
    let limits =
        ProcessLimits::new(4_096, 4_096, 8).expect("runtime probe limits are valid constants");
    let mut handle = runner
        .spawn(ProcessLaunch::new(command, None, limits))
        .map_err(|_| RuntimeProbeError::Spawn)?;
    let started = Instant::now();
    let mut lines = Vec::new();
    let mut output_chars = 0usize;

    loop {
        if started.elapsed() >= PROBE_TIMEOUT {
            handle
                .cancel_and_reap()
                .map_err(|_| RuntimeProbeError::Cleanup)?;
            return Err(RuntimeProbeError::TimedOut);
        }
        match handle.next(PROBE_POLL) {
            Ok(RuntimeOutput::StdoutLine(line) | RuntimeOutput::StderrLine(line)) => {
                output_chars = output_chars.saturating_add(line.chars().count());
                if output_chars > MAX_PROBE_OUTPUT_CHARS {
                    handle
                        .cancel_and_reap()
                        .map_err(|_| RuntimeProbeError::Cleanup)?;
                    return Err(RuntimeProbeError::OutputTooLarge);
                }
                lines.push(line);
            }
            Ok(RuntimeOutput::Idle) => {}
            Ok(RuntimeOutput::Exited(exit)) if exit.success => {
                let version = profile.parse_version_lines(&lines)?;
                return Ok(RuntimeIdentity {
                    binding: profile.binding(),
                    version,
                });
            }
            Ok(RuntimeOutput::Exited(_)) => return Err(RuntimeProbeError::NonZeroExit),
            Err(_) => {
                handle
                    .cancel_and_reap()
                    .map_err(|_| RuntimeProbeError::Cleanup)?;
                return Err(RuntimeProbeError::Read);
            }
        }
    }
}
