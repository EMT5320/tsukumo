//! Public request and service groups for Host orchestration.

use crate::clock::HostClock;
use crate::envelope::ExecutionContext;
use crate::ledger::HostLedger;
use crate::process::{ProcessRunner, ProcessTreeCapability};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tsukumo_adapters::{
    RuntimeEventDecoder, RuntimeLaunchConfig, RuntimeProfile, RuntimeSafetyCapability,
};
use tsukumo_soul::PreparedProjection;
use tsukumo_theater::{DirectorContext, StageWorld};

/// Cooperative cancellation flag checked between bounded process polls.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    /// Requests cancellation without touching the process from another thread.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    /// Reports whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Mutable durable services and immutable process/clock ports.
pub struct HostServices<'a> {
    pub(crate) ledger: &'a mut dyn HostLedger,
    pub(crate) runner: &'a dyn ProcessRunner,
    pub(crate) clock: &'a dyn HostClock,
}

impl<'a> HostServices<'a> {
    /// Groups the authority, process allocator, and timestamp source.
    pub fn new(
        ledger: &'a mut dyn HostLedger,
        runner: &'a dyn ProcessRunner,
        clock: &'a dyn HostClock,
    ) -> Self {
        Self {
            ledger,
            runner,
            clock,
        }
    }
}

/// Lossy presentation sink updated only after Chronicle acknowledges an event.
pub struct Presentation<'a> {
    pub(crate) world: &'a mut StageWorld,
    pub(crate) director: &'a DirectorContext,
}

impl<'a> Presentation<'a> {
    /// Groups Theater state with its pure Director context.
    pub fn new(world: &'a mut StageWorld, director: &'a DirectorContext) -> Self {
        Self { world, director }
    }
}

/// Adapter profile and host launch inputs selected for one execution.
pub struct RuntimeSelection<'a> {
    pub(crate) profile: &'a dyn RuntimeProfile,
    pub(crate) launch: &'a RuntimeLaunchConfig,
}

impl<'a> RuntimeSelection<'a> {
    /// Creates a selected runtime without copying command or environment values.
    pub fn new(profile: &'a dyn RuntimeProfile, launch: &'a RuntimeLaunchConfig) -> Self {
        Self { profile, launch }
    }
}

pub(crate) struct RunningResources {
    pub(crate) decoder: Box<dyn RuntimeEventDecoder>,
    pub(crate) cancellation: CancellationToken,
    pub(crate) safety_capability: RuntimeSafetyCapability,
    pub(crate) process_tree: ProcessTreeCapability,
}
/// Receipt-committed input and its runtime/envelope coordinates.
pub struct ExecutionRequest<'a> {
    pub(crate) prepared: &'a PreparedProjection,
    pub(crate) runtime: RuntimeSelection<'a>,
    pub(crate) context: ExecutionContext,
    pub(crate) cancellation: CancellationToken,
}

impl<'a> ExecutionRequest<'a> {
    /// Creates an execution request with a fresh cooperative cancellation token.
    pub fn new(
        prepared: &'a PreparedProjection,
        runtime: RuntimeSelection<'a>,
        context: ExecutionContext,
    ) -> Self {
        Self {
            prepared,
            runtime,
            context,
            cancellation: CancellationToken::default(),
        }
    }

    /// Supplies a cancellation token shared with the caller.
    pub fn with_cancellation(mut self, cancellation: CancellationToken) -> Self {
        self.cancellation = cancellation;
        self
    }
}
