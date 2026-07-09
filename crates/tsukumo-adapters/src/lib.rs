//! Tsukumo drive adapters (A1): structured streams → [`KernelEvent`].
//!
//! Default channel: Claude-like `stream-json` NDJSON (own-process / recorded).
//! Full ACP client is deferred — see task `notes-a1-channel.md`.
//!
//! Theater never sees vendor payloads; only normalized kernel events leave here.
//! Stage wiring lives in integration tests / examples (adapters stay vendor-side).

pub mod briefing;
pub mod stream_json;
pub mod synthetic;

pub use briefing::{
    assemble_prompt, BriefingSource, NullBriefing, PromptAssemblyContext, StubBriefing,
};
pub use stream_json::{
    parse_stream_json_line, parse_stream_json_reader, parse_stream_json_str, AdapterError,
    StreamJsonOptions,
};
pub use synthetic::{synthetic_demo_events, synthetic_demo_stream_jsonl};
