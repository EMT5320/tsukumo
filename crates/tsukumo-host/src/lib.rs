//! Tsukumo host composition root and owned runtime lifecycle.

mod cleanup;
mod clock;
mod config;
mod envelope;
mod host_api;
mod host_error;
mod ledger;
mod orchestrator;
mod process;
mod process_handle;
mod process_reader;
mod report;
mod safety;
mod session;
mod terminal;

pub use clock::{ClockError, HostClock, SystemClock};
pub use config::{ExecutionPolicy, ProcessConfigError, ProcessLimits};
pub use envelope::ExecutionContext;
pub use host_api::{
    CancellationToken, ExecutionRequest, HostServices, Presentation, RuntimeSelection,
};
pub use host_error::HostError;
pub use ledger::HostLedger;
pub use orchestrator::RuntimeOrchestrator;
pub use process::{
    ProcessError, ProcessExit, ProcessLaunch, ProcessRunner, ProcessTreeCapability, RuntimeHandle,
    RuntimeOutput, StandardProcessRunner,
};
pub use process_handle::StandardRuntimeHandle;
pub use report::{CleanupStatus, ExecutionFailure, ExecutionReport, FailureDetail};
pub use safety::{
    BridgeError, PermissionBridge, PermissionController, PermissionRegistration, PermissionRequest,
    PermissionResolution, PermissionResolutionSource, PermissionScope, SafetyError,
    UnwiredPermissionBridge,
};
