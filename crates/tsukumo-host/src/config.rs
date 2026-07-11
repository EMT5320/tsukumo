//! Typed limits and time budgets for owned runtime processes.

use std::time::Duration;
use thiserror::Error;

const DEFAULT_STDOUT_LINE_BYTES: usize = 1_048_576;
const DEFAULT_STDERR_TOTAL_BYTES: usize = 65_536;
const DEFAULT_CHANNEL_CAPACITY: usize = 32;
const MAX_STDOUT_LINE_BYTES: usize = 16_777_216;
const MAX_STDERR_TOTAL_BYTES: usize = 1_048_576;
const MAX_CHANNEL_CAPACITY: usize = 4_096;
const DEFAULT_EXECUTION_TIMEOUT: Duration = Duration::from_secs(600);
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(20);
const MAX_EXECUTION_TIMEOUT: Duration = Duration::from_secs(86_400);

/// Validated memory and buffering limits for one child process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessLimits {
    stdout_line_bytes: usize,
    stderr_total_bytes: usize,
    channel_capacity: usize,
}

impl ProcessLimits {
    /// Creates non-zero limits before any child resources are allocated.
    pub fn new(
        stdout_line_bytes: usize,
        stderr_total_bytes: usize,
        channel_capacity: usize,
    ) -> Result<Self, ProcessConfigError> {
        for (field, value, maximum) in [
            (
                "stdout_line_bytes",
                stdout_line_bytes,
                MAX_STDOUT_LINE_BYTES,
            ),
            (
                "stderr_total_bytes",
                stderr_total_bytes,
                MAX_STDERR_TOTAL_BYTES,
            ),
            ("channel_capacity", channel_capacity, MAX_CHANNEL_CAPACITY),
        ] {
            if value == 0 {
                return Err(ProcessConfigError::ZeroLimit { field });
            }
            if value > maximum {
                return Err(ProcessConfigError::LimitTooLarge { field, maximum });
            }
        }
        Ok(Self {
            stdout_line_bytes,
            stderr_total_bytes,
            channel_capacity,
        })
    }

    /// Returns the maximum raw bytes accepted for one stdout line.
    pub const fn stdout_line_bytes(self) -> usize {
        self.stdout_line_bytes
    }

    /// Returns the cumulative raw stderr budget for one process.
    pub const fn stderr_total_bytes(self) -> usize {
        self.stderr_total_bytes
    }

    /// Returns the bounded cross-thread signal capacity.
    pub const fn channel_capacity(self) -> usize {
        self.channel_capacity
    }
}

impl Default for ProcessLimits {
    fn default() -> Self {
        Self {
            stdout_line_bytes: DEFAULT_STDOUT_LINE_BYTES,
            stderr_total_bytes: DEFAULT_STDERR_TOTAL_BYTES,
            channel_capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }
}

/// Wall-time and polling policy for one owned execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionPolicy {
    timeout: Duration,
    poll_interval: Duration,
    process_limits: ProcessLimits,
}

impl ExecutionPolicy {
    /// Creates a bounded execution policy with default process memory limits.
    pub fn new(timeout: Duration, poll_interval: Duration) -> Result<Self, ProcessConfigError> {
        if timeout.is_zero() {
            return Err(ProcessConfigError::ZeroDuration { field: "timeout" });
        }
        if poll_interval.is_zero() {
            return Err(ProcessConfigError::ZeroDuration {
                field: "poll_interval",
            });
        }
        if timeout > MAX_EXECUTION_TIMEOUT {
            return Err(ProcessConfigError::DurationTooLarge {
                field: "timeout",
                maximum: MAX_EXECUTION_TIMEOUT,
            });
        }
        Ok(Self {
            timeout,
            poll_interval,
            process_limits: ProcessLimits::default(),
        })
    }

    /// Returns the maximum wall time for one execution.
    pub const fn timeout(self) -> Duration {
        self.timeout
    }

    /// Returns the maximum duration of one process poll.
    pub const fn poll_interval(self) -> Duration {
        self.poll_interval
    }

    /// Returns validated output and channel limits.
    pub const fn process_limits(self) -> ProcessLimits {
        self.process_limits
    }
    /// Replaces process memory limits after their independent validation.
    pub const fn with_process_limits(mut self, process_limits: ProcessLimits) -> Self {
        self.process_limits = process_limits;
        self
    }
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_EXECUTION_TIMEOUT,
            poll_interval: DEFAULT_POLL_INTERVAL,
            process_limits: ProcessLimits::default(),
        }
    }
}

/// Invalid process or execution limits rejected before spawn.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProcessConfigError {
    #[error("process limit {field} must be greater than zero")]
    ZeroLimit { field: &'static str },
    #[error("process limit {field} exceeds {maximum}")]
    LimitTooLarge { field: &'static str, maximum: usize },
    #[error("execution duration {field} must be greater than zero")]
    ZeroDuration { field: &'static str },
    #[error("execution duration {field} exceeds {maximum:?}")]
    DurationTooLarge {
        field: &'static str,
        maximum: Duration,
    },
}
