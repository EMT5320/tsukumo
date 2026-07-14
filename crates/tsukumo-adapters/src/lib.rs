//! Runtime adapters normalize vendor streams into shared kernel payloads.
//!
//! Adapters never assign durable event IDs or own Chronicle persistence.

pub mod briefing;
pub mod claude;
pub mod codex;
pub mod codex_json;
pub mod runtime;
pub mod stream_json;
pub mod synthetic;
mod vendor_fields;

pub use briefing::{
    assemble_prompt, BriefingSource, NullBriefing, PromptAssemblyContext, StubBriefing,
};
pub use claude::{claude_c1_success_fixture, ClaudeRuntimeProfile, ClaudeSafetyMode};
pub use codex::{
    codex_0_135_0_gnu_capture_manifest, codex_0_135_0_gnu_with_state_fixture,
    codex_0_135_0_gnu_without_state_fixture, codex_0_135_0_success_fixture, CodexRuntimeProfile,
    CodexSandboxMode,
};
pub use codex_json::CodexJsonDecoder;
pub use runtime::{
    DecodeDisposition, DecodedRuntimeLine, PromptDelivery, RuntimeCommandSpec, RuntimeEventDecoder,
    RuntimeLaunchConfig, RuntimeProfile, RuntimeProfileError, RuntimeSafetyCapability,
};
pub use stream_json::{
    parse_stream_json_line, parse_stream_json_reader, parse_stream_json_str, AdapterError,
    ClaudeStreamDecoder, DecodeError,
};
pub use synthetic::{synthetic_demo_payloads, synthetic_demo_stream_jsonl};
