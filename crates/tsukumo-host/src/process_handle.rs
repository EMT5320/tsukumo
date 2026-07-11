//! Standard process handle, output polling, and exactly-once reaping.

use crate::process::{ProcessError, ProcessExit, RuntimeHandle, RuntimeOutput};
use crate::process_reader::{ProcessSignal, StreamKind};
use std::fmt;
use std::process::Child;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const READER_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

/// Concrete standard process handle with exactly-once OS cleanup.
pub struct StandardRuntimeHandle {
    child: Child,
    receiver: Receiver<ProcessSignal>,
    readers: Vec<JoinHandle<()>>,
    readers_finished: usize,
    exit: Option<ProcessExit>,
}

impl StandardRuntimeHandle {
    pub(crate) fn new(
        child: Child,
        receiver: Receiver<ProcessSignal>,
        readers: Vec<JoinHandle<()>>,
    ) -> Self {
        Self {
            child,
            receiver,
            readers,
            readers_finished: 0,
            exit: None,
        }
    }

    fn record_signal(
        &mut self,
        signal: ProcessSignal,
    ) -> Result<Option<RuntimeOutput>, ProcessError> {
        match signal {
            ProcessSignal::Line(StreamKind::Stdout, line) => {
                Ok(Some(RuntimeOutput::StdoutLine(line)))
            }
            ProcessSignal::Line(StreamKind::Stderr, line) => {
                Ok(Some(RuntimeOutput::StderrLine(line)))
            }
            ProcessSignal::Finished => {
                self.readers_finished = self.readers_finished.saturating_add(1);
                Ok(None)
            }
            ProcessSignal::Fault(error) => Err(error),
        }
    }

    fn finish_if_ready(&mut self) -> Result<Option<RuntimeOutput>, ProcessError> {
        if self.readers_finished < self.readers.len() {
            return Ok(None);
        }
        let Some(status) = self.child.try_wait().map_err(ProcessError::Wait)? else {
            return Ok(None);
        };
        let exit = ProcessExit::from(status);
        self.exit = Some(exit);
        self.join_readers()?;
        Ok(Some(RuntimeOutput::Exited(exit)))
    }

    fn join_readers(&mut self) -> Result<(), ProcessError> {
        for reader in self.readers.drain(..) {
            reader
                .join()
                .map_err(|_| ProcessError::ReaderThreadPanicked)?;
        }
        Ok(())
    }

    fn drain_readers(&mut self) -> Result<(), ProcessError> {
        let deadline = Instant::now() + READER_SHUTDOWN_TIMEOUT;
        while self.readers_finished < self.readers.len() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(ProcessError::ReaderShutdownTimedOut);
            }
            match self.receiver.recv_timeout(remaining) {
                Ok(ProcessSignal::Finished) => {
                    self.readers_finished = self.readers_finished.saturating_add(1);
                }
                Ok(ProcessSignal::Line(_, _) | ProcessSignal::Fault(_)) => {}
                Err(RecvTimeoutError::Timeout) => {
                    return Err(ProcessError::ReaderShutdownTimedOut);
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        self.join_readers()
    }

    fn reap(&mut self) -> Result<ProcessExit, ProcessError> {
        if let Some(exit) = self.exit {
            return Ok(exit);
        }
        let status = match self.child.try_wait().map_err(ProcessError::Wait)? {
            Some(status) => status,
            None => {
                if let Err(source) = self.child.kill() {
                    if let Some(status) = self.child.try_wait().map_err(ProcessError::Wait)? {
                        status
                    } else {
                        return Err(ProcessError::Kill(source));
                    }
                } else {
                    self.child.wait().map_err(ProcessError::Wait)?
                }
            }
        };
        let exit = ProcessExit::from(status);
        self.exit = Some(exit);
        self.drain_readers()?;
        Ok(exit)
    }
}

impl fmt::Debug for StandardRuntimeHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StandardRuntimeHandle")
            .field("readers_finished", &self.readers_finished)
            .field("reaped", &self.exit.is_some())
            .finish()
    }
}

impl RuntimeHandle for StandardRuntimeHandle {
    fn next(&mut self, wait: Duration) -> Result<RuntimeOutput, ProcessError> {
        if let Some(exit) = self.exit {
            return Ok(RuntimeOutput::Exited(exit));
        }
        let Some(deadline) = Instant::now().checked_add(wait) else {
            return Err(ProcessError::WaitDurationTooLarge);
        };
        loop {
            if let Some(output) = self.finish_if_ready()? {
                return Ok(output);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Ok(RuntimeOutput::Idle);
            }
            match self.receiver.recv_timeout(remaining) {
                Ok(signal) => {
                    if let Some(output) = self.record_signal(signal)? {
                        return Ok(output);
                    }
                }
                Err(RecvTimeoutError::Timeout) => return Ok(RuntimeOutput::Idle),
                Err(RecvTimeoutError::Disconnected) => {
                    if self.readers_finished < self.readers.len() {
                        return Err(ProcessError::OutputChannelClosed);
                    }
                    // Pipe closure can lead OS exit publication by one scheduler turn.
                    std::thread::sleep(remaining.min(Duration::from_millis(1)));
                }
            }
        }
    }

    fn cancel_and_reap(&mut self) -> Result<ProcessExit, ProcessError> {
        self.reap()
    }
}

impl Drop for StandardRuntimeHandle {
    fn drop(&mut self) {
        // Drop cannot surface cleanup errors; explicit host paths retain them.
        if self.reap().is_err() {}
    }
}
