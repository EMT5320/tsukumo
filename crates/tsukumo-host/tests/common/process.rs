//! Fake process and clock ports for Host integration tests.

use std::collections::VecDeque;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tsukumo_host::{
    HostClock, ProcessError, ProcessExit, ProcessLaunch, ProcessRunner, RuntimeHandle,
    RuntimeOutput,
};
use tsukumo_kernel::Timestamp;

pub struct FixedClock {
    next: AtomicI64,
}

impl FixedClock {
    pub const fn new(first: i64) -> Self {
        Self {
            next: AtomicI64::new(first),
        }
    }
}

impl HostClock for FixedClock {
    fn now(&self) -> Result<Timestamp, tsukumo_host::ClockError> {
        Ok(Timestamp::from_unix_millis(
            self.next.fetch_add(1, Ordering::SeqCst),
        ))
    }
}

pub(super) struct FakeMetrics {
    pub(super) cancel_count: AtomicUsize,
    pub(super) exit_emitted: AtomicBool,
    spawn_count: AtomicUsize,
    captured_prompt: Mutex<Option<String>>,
}

pub struct FakeRunner {
    pub(super) metrics: Arc<FakeMetrics>,
    outputs: Mutex<Option<VecDeque<RuntimeOutput>>>,
    fail_spawn: bool,
    fail_cleanup: bool,
}

impl FakeRunner {
    pub fn new(outputs: impl IntoIterator<Item = RuntimeOutput>) -> Self {
        Self {
            metrics: Arc::new(FakeMetrics {
                spawn_count: AtomicUsize::new(0),
                cancel_count: AtomicUsize::new(0),
                exit_emitted: AtomicBool::new(false),
                captured_prompt: Mutex::new(None),
            }),
            outputs: Mutex::new(Some(outputs.into_iter().collect())),
            fail_spawn: false,
            fail_cleanup: false,
        }
    }

    pub fn failing() -> Self {
        Self {
            fail_spawn: true,
            ..Self::new([])
        }
    }

    pub fn with_cleanup_failure(mut self) -> Self {
        self.fail_cleanup = true;
        self
    }
    pub fn spawn_count(&self) -> usize {
        self.metrics.spawn_count.load(Ordering::SeqCst)
    }

    pub fn cancel_count(&self) -> usize {
        self.metrics.cancel_count.load(Ordering::SeqCst)
    }

    pub fn captured_prompt(&self) -> Option<String> {
        self.metrics
            .captured_prompt
            .lock()
            .expect("lock captured prompt")
            .clone()
    }

    pub(super) fn exit_signal(&self) -> Arc<FakeMetrics> {
        self.metrics.clone()
    }
}

impl ProcessRunner for FakeRunner {
    fn spawn(&self, launch: ProcessLaunch) -> Result<Box<dyn RuntimeHandle>, ProcessError> {
        self.metrics.spawn_count.fetch_add(1, Ordering::SeqCst);
        if self.fail_spawn {
            return Err(ProcessError::Spawn(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "injected launch failure",
            )));
        }
        let prompt = launch.prompt.map(|text| text.expose().to_owned());
        *self
            .metrics
            .captured_prompt
            .lock()
            .expect("lock captured prompt") = prompt;
        let outputs = self
            .outputs
            .lock()
            .expect("lock fake outputs")
            .take()
            .unwrap_or_default();
        Ok(Box::new(FakeHandle {
            metrics: self.metrics.clone(),
            outputs,
            reaped: false,
            exit: ProcessExit {
                code: Some(137),
                success: false,
            },
            fail_cleanup: self.fail_cleanup,
        }))
    }
}

struct FakeHandle {
    metrics: Arc<FakeMetrics>,
    outputs: VecDeque<RuntimeOutput>,
    reaped: bool,
    exit: ProcessExit,
    fail_cleanup: bool,
}

impl fmt::Debug for FakeHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FakeHandle")
            .field("queued_outputs", &self.outputs.len())
            .field("reaped", &self.reaped)
            .finish()
    }
}

impl RuntimeHandle for FakeHandle {
    fn next(&mut self, _wait: Duration) -> Result<RuntimeOutput, ProcessError> {
        let output = self.outputs.pop_front().unwrap_or(RuntimeOutput::Idle);
        if let RuntimeOutput::Exited(exit) = output {
            self.reaped = true;
            self.exit = exit;
            self.metrics.exit_emitted.store(true, Ordering::SeqCst);
            Ok(RuntimeOutput::Exited(exit))
        } else {
            Ok(output)
        }
    }

    fn cancel_and_reap(&mut self) -> Result<ProcessExit, ProcessError> {
        if !self.reaped {
            self.reaped = true;
            self.metrics.cancel_count.fetch_add(1, Ordering::SeqCst);
        }
        if self.fail_cleanup {
            Err(ProcessError::Kill(std::io::Error::other(
                "injected cleanup failure",
            )))
        } else {
            Ok(self.exit)
        }
    }
}

pub fn successful_outputs() -> Vec<RuntimeOutput> {
    let mut outputs = tsukumo_adapters::claude_c1_success_fixture()
        .lines()
        .map(|line| RuntimeOutput::StdoutLine(line.to_owned()))
        .collect::<Vec<_>>();
    outputs.push(RuntimeOutput::Exited(ProcessExit {
        code: Some(0),
        success: true,
    }));
    outputs
}
