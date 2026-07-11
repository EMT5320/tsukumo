//! Runtime adapters normalize vendor streams into shared kernel payloads.
//!
//! Adapters never assign durable event IDs or own Chronicle persistence.

pub mod briefing;
pub mod stream_json;
pub mod synthetic;

pub use briefing::{
    assemble_prompt, BriefingSource, NullBriefing, PromptAssemblyContext, StubBriefing,
};
pub use stream_json::{
    parse_stream_json_line, parse_stream_json_reader, parse_stream_json_str, AdapterError,
    DecodeError,
};
pub use synthetic::{synthetic_demo_payloads, synthetic_demo_stream_jsonl};
