//! Tsukumo host composition root and owned runtime lifecycle.

mod cleanup;
mod cli;
mod clock;
mod config;
mod envelope;
mod host_api;
mod host_error;
mod ledger;
mod local_path;
mod orchestrator;
mod presentation_pack;
mod process;
mod process_handle;
mod process_reader;
mod product;
mod report;
mod safety;
mod session;
mod terminal;
mod tui;

pub use cli::{parse_host_args, HostCliError, HostCommand, HostRunOptions};
pub use clock::{ClockError, HostClock, SystemClock};
pub use config::{ExecutionPolicy, ProcessConfigError, ProcessLimits};
pub use envelope::ExecutionContext;
pub use host_api::{
    CancellationToken, ExecutionRequest, HostServices, Presentation, RuntimeSelection,
};
pub use host_error::HostError;
pub use ledger::HostLedger;
pub use orchestrator::RuntimeOrchestrator;
pub use presentation_pack::{
    load_presentation_pack, PresentationPackLoadError, PresentationPackSource,
};
pub use process::{
    ProcessError, ProcessExit, ProcessLaunch, ProcessRunner, ProcessTreeCapability, RuntimeHandle,
    RuntimeOutput, StandardProcessRunner,
};
pub use process_handle::StandardRuntimeHandle;
pub use product::{
    HostProductController, ProductControl, ProductController, ProductControllerError,
    ProductSnapshot,
};
pub use report::{CleanupStatus, ExecutionFailure, ExecutionReport, FailureDetail};
pub use safety::{
    BridgeError, PermissionBridge, PermissionController, PermissionRegistration, PermissionRequest,
    PermissionResolution, PermissionResolutionSource, PermissionScope, SafetyError,
    UnwiredPermissionBridge,
};
pub use tui::{color_capability_from_env, map_terminal_key, run_tui, TuiError};
