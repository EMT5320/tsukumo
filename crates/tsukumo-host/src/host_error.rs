//! Errors that prevent or interrupt trustworthy Host accounting.

use crate::clock::ClockError;
use crate::process::ProcessError;
use crate::report::CleanupStatus;
use thiserror::Error;
use tsukumo_adapters::RuntimeProfileError;
use tsukumo_kernel::{ExecutionId, ProjectionId, RuntimeBinding};
use tsukumo_soul::SoulError;

/// Failures that prevent or interrupt trustworthy Host accounting.
#[derive(Debug, Error)]
pub enum HostError {
    #[error("projection receipt {projection_id} is missing from the selected ledger")]
    MissingReceipt { projection_id: ProjectionId },
    #[error("projection receipt {projection_id} differs from the prepared value")]
    ReceiptMismatch { projection_id: ProjectionId },
    #[error("runtime selection does not match projection receipt")]
    RuntimeMismatch {
        receipt: RuntimeBinding,
        selected: RuntimeBinding,
    },
    #[error("permission resolution scope does not match its projection receipt")]
    PermissionScopeMismatch,
    #[error("permission resolution has no matching durable request")]
    MissingPermissionRequest,
    #[error("execution {execution_id} already has its deterministic start event")]
    AlreadyExecuted { execution_id: ExecutionId },
    #[error(transparent)]
    Profile(#[from] RuntimeProfileError),
    #[error(transparent)]
    Clock(#[from] ClockError),
    #[error("clock failed while a runtime process was active; cleanup: {cleanup:?}")]
    ClockDuringExecution {
        #[source]
        source: ClockError,
        cleanup: CleanupStatus,
        cleanup_error: Option<ProcessError>,
    },
    #[error("Chronicle failed before a runtime process was active")]
    ChronicleBeforeSpawn(#[source] SoulError),
    #[error("Chronicle failed while a runtime process was active; cleanup: {cleanup:?}")]
    ChronicleDuringExecution {
        #[source]
        source: SoulError,
        cleanup: CleanupStatus,
        cleanup_error: Option<ProcessError>,
    },
}
