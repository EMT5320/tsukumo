//! Cleanup evidence retained when a live Host operation loses durable authority.

use crate::process::{ProcessError, RuntimeHandle};
use crate::report::CleanupStatus;

pub(crate) struct CleanupAttempt {
    pub(crate) status: CleanupStatus,
    pub(crate) error: Option<ProcessError>,
}

pub(crate) fn cancel_after_host_failure(handle: &mut dyn RuntimeHandle) -> CleanupAttempt {
    match handle.cancel_and_reap() {
        Ok(exit) => CleanupAttempt {
            status: CleanupStatus::Cancelled(exit),
            error: None,
        },
        Err(error) => CleanupAttempt {
            status: CleanupStatus::Failed,
            error: Some(error),
        },
    }
}
