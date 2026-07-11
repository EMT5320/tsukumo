//! C1 regression for observable durable trace failures.

use tempfile::tempdir;
use tsukumo_kernel::QuestId;
use tsukumo_soul::{assemble_with_trace, SoulError, TraceLog};

#[test]
fn inject_trace_failure_is_returned_to_the_caller() {
    // Given: a trace root that is a file and cannot contain inject_trace.jsonl.
    let directory = tempdir().expect("create trace error directory");
    let blocked_root = directory.path().join("blocked");
    std::fs::write(&blocked_root, "file").expect("create blocked trace root");
    let mut trace = TraceLog::open(&blocked_root);

    // When: prompt assembly attempts to append durable trace evidence.
    let error = assemble_with_trace(
        "brief",
        "goal",
        Some(&QuestId::new("quest-trace")),
        Some(&mut trace),
    )
    .expect_err("trace append must be observable");

    // Then: the exact storage boundary failure reaches the caller.
    assert!(matches!(error, SoulError::Io(_)));
}
