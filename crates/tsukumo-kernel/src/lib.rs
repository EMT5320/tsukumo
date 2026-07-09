//! Tsukumo L1 kernel: normalized event contract and soft identity stubs.
//!
//! Adapters (ACP / stream-json / builtin) produce [`KernelEvent`].
//! Theater and growth layers must not see vendor-specific payloads.

pub mod event;
pub mod identity;
pub mod session;

pub use event::{KernelEvent, ToolResult};
pub use identity::{BackendKind, ExecutorId};
pub use session::{parse_jsonl_line, read_jsonl_events, SessionError};
