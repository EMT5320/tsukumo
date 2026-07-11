//! Credential-free Claude fixture used by examples and integration tests.

use crate::stream_json::{parse_stream_json_str, AdapterError};
use tsukumo_kernel::KernelEventPayload;

/// Returns the reviewed Claude-like JSONL fixture committed with this crate.
pub fn synthetic_demo_stream_jsonl() -> &'static str {
    include_str!("../fixtures/synthetic_a1.jsonl")
}

/// Normalizes the committed fixture through the production decoder.
pub fn synthetic_demo_payloads() -> Result<Vec<KernelEventPayload>, AdapterError> {
    parse_stream_json_str(synthetic_demo_stream_jsonl())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_contains_permission_tool_and_outcome_payloads() {
        // Given: the committed credential-free Claude fixture.
        let payloads = synthetic_demo_payloads().expect("decode synthetic fixture");

        // When: callers inspect the normalized payload sequence.
        let has_permission = payloads
            .iter()
            .any(|payload| matches!(payload, KernelEventPayload::PermissionRequested { .. }));
        let has_tool = payloads
            .iter()
            .any(|payload| matches!(payload, KernelEventPayload::ToolStart { tool, .. } if tool == "Bash"));
        let has_outcome = payloads
            .iter()
            .any(|payload| matches!(payload, KernelEventPayload::Outcome { .. }));

        // Then: the fixture covers the minimum drive path.
        assert!(has_permission);
        assert!(has_tool);
        assert!(has_outcome);
    }
}
