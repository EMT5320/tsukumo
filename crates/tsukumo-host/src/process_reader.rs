//! Bounded concurrent stdout and stderr readers.

use crate::config::ProcessLimits;
use crate::process::ProcessError;
use std::io::Read;
use std::mem;
use std::sync::mpsc::SyncSender;
use std::thread::{self, JoinHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StreamKind {
    Stdout,
    Stderr,
}

impl StreamKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

pub(crate) enum ProcessSignal {
    Line(StreamKind, String),
    Finished,
    Fault(ProcessError),
}

pub(crate) fn spawn_reader<R>(
    source: R,
    kind: StreamKind,
    limits: ProcessLimits,
    sender: SyncSender<ProcessSignal>,
) -> JoinHandle<()>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let outcome = read_stream(source, kind, limits, &sender);
        let signal = match outcome {
            Ok(()) => ProcessSignal::Finished,
            Err(error) => ProcessSignal::Fault(error),
        };
        // A disconnected receiver means its owning process handle was dropped.
        if sender.send(signal).is_err() {}
    })
}

fn read_stream<R: Read>(
    mut source: R,
    kind: StreamKind,
    limits: ProcessLimits,
    sender: &SyncSender<ProcessSignal>,
) -> Result<(), ProcessError> {
    let line_limit = match kind {
        StreamKind::Stdout => limits.stdout_line_bytes(),
        StreamKind::Stderr => limits.stderr_total_bytes(),
    };
    let mut total_bytes = 0usize;
    let mut line = Vec::new();
    let mut chunk = [0u8; 8_192];
    loop {
        let read = source
            .read(&mut chunk)
            .map_err(|source| ProcessError::Read {
                stream: kind.label(),
                source,
            })?;
        if read == 0 {
            if !line.is_empty() {
                send_line(kind, &mut line, sender)?;
            }
            return Ok(());
        }
        for byte in &chunk[..read] {
            total_bytes = total_bytes.saturating_add(1);
            if kind == StreamKind::Stderr && total_bytes > limits.stderr_total_bytes() {
                return Err(ProcessError::StderrLimitExceeded {
                    maximum: limits.stderr_total_bytes(),
                });
            }
            if *byte == b'\n' {
                send_line(kind, &mut line, sender)?;
                continue;
            }
            line.push(*byte);
            if line.len() > line_limit {
                return Err(ProcessError::LineLimitExceeded {
                    stream: kind.label(),
                    maximum: line_limit,
                });
            }
        }
    }
}

fn send_line(
    kind: StreamKind,
    line: &mut Vec<u8>,
    sender: &SyncSender<ProcessSignal>,
) -> Result<(), ProcessError> {
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    let bytes = mem::take(line);
    let text = String::from_utf8(bytes).map_err(|_| ProcessError::InvalidUtf8 {
        stream: kind.label(),
    })?;
    sender
        .send(ProcessSignal::Line(kind, text))
        .map_err(|_| ProcessError::OutputChannelClosed)
}
