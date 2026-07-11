//! Internal runtime terminal state and stable Chronicle summaries.

use crate::process::ProcessExit;
use crate::report::{ExecutionFailure, FailureDetail};
use tsukumo_adapters::{AdapterError, RuntimeEventDecoder};
use tsukumo_kernel::{OutcomeStatus, PersistedText};

pub(crate) struct VendorOutcome {
    pub(crate) status: OutcomeStatus,
    pub(crate) summary: Option<PersistedText>,
}

pub(crate) struct Termination {
    pub(crate) status: OutcomeStatus,
    pub(crate) failure: Option<ExecutionFailure>,
    pub(crate) detail: Option<FailureDetail>,
    pub(crate) known_exit: Option<ProcessExit>,
    pub(crate) summary: Option<PersistedText>,
}

pub(crate) const fn status_summary(status: OutcomeStatus) -> &'static str {
    match status {
        OutcomeStatus::Succeeded => "runtime execution succeeded",
        OutcomeStatus::Failed => "runtime execution failed",
        OutcomeStatus::Cancelled => "runtime execution was cancelled",
        OutcomeStatus::PermissionDenied => "runtime permission was denied",
        OutcomeStatus::SafetyUnsupported => "runtime safety bridge is unsupported",
        OutcomeStatus::Degraded => "runtime process degraded",
        OutcomeStatus::TimedOut => "runtime execution timed out",
        OutcomeStatus::MalformedOutput => "runtime output was malformed or truncated",
        OutcomeStatus::NonZeroExit => "runtime process exited unsuccessfully",
        OutcomeStatus::LaunchFailed => "runtime process could not be launched",
    }
}
pub(crate) fn reconcile_exit(
    decoder: &dyn RuntimeEventDecoder,
    vendor_outcome: &mut Option<VendorOutcome>,
    exit: ProcessExit,
) -> Termination {
    if !exit.success {
        return Termination {
            status: OutcomeStatus::NonZeroExit,
            failure: Some(ExecutionFailure::NonZeroExit),
            detail: None,
            known_exit: Some(exit),
            summary: None,
        };
    }
    if let Err(error) = decoder.finish() {
        let failure = if matches!(error, AdapterError::TruncatedStream { .. }) {
            ExecutionFailure::TruncatedStream
        } else {
            ExecutionFailure::MalformedOutput
        };
        return Termination {
            status: OutcomeStatus::MalformedOutput,
            failure: Some(failure),
            detail: Some(FailureDetail::Adapter(error)),
            known_exit: Some(exit),
            summary: None,
        };
    }
    match vendor_outcome.take() {
        Some(outcome) => Termination {
            status: outcome.status,
            failure: (outcome.status != OutcomeStatus::Succeeded)
                .then_some(ExecutionFailure::VendorFailure),
            detail: None,
            known_exit: Some(exit),
            summary: outcome.summary,
        },
        None => Termination {
            status: OutcomeStatus::MalformedOutput,
            failure: Some(ExecutionFailure::TruncatedStream),
            detail: None,
            known_exit: Some(exit),
            summary: None,
        },
    }
}
