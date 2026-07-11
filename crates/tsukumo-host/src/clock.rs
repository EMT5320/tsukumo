//! Wall-clock port used when the Host assigns durable event timestamps.

use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tsukumo_kernel::Timestamp;

/// Timestamp source injected into deterministic tests and the production host.
pub trait HostClock {
    fn now(&self) -> Result<Timestamp, ClockError>;
}

/// Production UTC clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl HostClock for SystemClock {
    fn now(&self) -> Result<Timestamp, ClockError> {
        let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let milliseconds =
            i64::try_from(duration.as_millis()).map_err(|_| ClockError::TimestampOutOfRange)?;
        Ok(Timestamp::from_unix_millis(milliseconds))
    }
}

/// Clock failures kept separate from runtime outcomes.
#[derive(Debug, Error)]
pub enum ClockError {
    #[error("system clock is before the Unix epoch: {0}")]
    BeforeUnixEpoch(#[from] std::time::SystemTimeError),
    #[error("system timestamp does not fit signed milliseconds")]
    TimestampOutOfRange,
}
