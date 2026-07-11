//! Typed execution reports and sanitized diagnostic categories.

use crate::process::{ProcessError, ProcessExit, ProcessTreeCapability};
use tsukumo_adapters::AdapterError;
use tsukumo_kernel::OutcomeStatus;

/// How child resources reached their terminal state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupStatus {
    NotStarted,
    Natural(ProcessExit),
    Cancelled(ProcessExit),
    Failed,
}

/// Stable failure classification used by operators and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionFailure {
    LaunchFailed,
    Cancelled,
    TimedOut,
    MalformedOutput,
    TruncatedStream,
    NonZeroExit,
    ProcessFailure,
    SafetyUnsupported,
    VendorFailure,
}

/// Redacted typed detail retained outside Chronicle presentation text.
#[derive(Debug)]
pub enum FailureDetail {
    Adapter(AdapterError),
    Process(ProcessError),
}

/// One terminal report for exactly one prepared projection execution.
#[derive(Debug)]
pub struct ExecutionReport {
    pub status: OutcomeStatus,
    pub process_tree: ProcessTreeCapability,
    pub failure: Option<ExecutionFailure>,
    pub detail: Option<FailureDetail>,
    pub cleanup: CleanupStatus,
    pub cleanup_error: Option<ProcessError>,
    pub exit: Option<ProcessExit>,
    pub committed_events: usize,
    pub stderr_lines: usize,
    pub known_ignored_lines: usize,
    pub unknown_skipped_lines: usize,
}
