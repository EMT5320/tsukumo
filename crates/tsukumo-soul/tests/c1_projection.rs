//! Contract tests for deterministic projection selection, rendering, and receipts.

mod common;

use common::{
    checkpoint_event, event, persist_gnu_constraint, projection_event, projection_request,
};
use tempfile::tempdir;
use tsukumo_kernel::{CheckpointId, KernelEventPayload, PersistedText, QuestId, Timestamp};
use tsukumo_soul::{
    CheckpointTrigger, CheckpointWriteRequest, HandoffCheckpoint, OperatingSystem,
    ProjectionOmissionReason, ProjectionSection, ProjectionWriteRequest, SoulStore, StateRef,
    StateScope,
};

#[test]
fn projection_is_canonical_scoped_and_reopenable() {
    // Given: one pinned GNU constraint, one out-of-scope state, and a checkpoint.
    let directory = tempdir().expect("create projection test directory");
    let mut store = SoulStore::open(directory.path()).expect("open projection store");
    let scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    let selected = persist_gnu_constraint(&mut store, "state-gnu", "gnu", scope, 100);
    let other_scope = StateScope::workspace_os("other", OperatingSystem::Windows);
    let omitted = persist_gnu_constraint(&mut store, "state-other", "other", other_scope, 110);
    let source = event(
        "event-checkpoint-source",
        190,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Continue the Tsukumo MVP"),
        },
    );
    store
        .append_event(&source)
        .expect("append checkpoint source");
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-projection"),
        QuestId::new("quest-projection"),
        1,
        None,
        PersistedText::from_reviewed("Continue Tsukumo MVP"),
        Timestamp::from_unix_millis(200),
        CheckpointTrigger::RuntimeSwitch,
    )
    .with_constraint_refs(vec![StateRef::new(
        selected.state_id.clone(),
        selected.version,
    )])
    .with_source_event_refs(vec![source.event_id]);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            checkpoint.clone(),
            checkpoint_event(&checkpoint),
        ))
        .expect("save projection checkpoint");

    // When: the same logical projection is prepared twice with distinct IDs.
    let first_request = projection_request(
        "projection-1",
        "execution-1",
        &checkpoint.id,
        "Run the full test suite",
    );
    let first_event = projection_event(
        "event-projection-1",
        500,
        &first_request.projection_id,
        &checkpoint.id,
        &first_request.execution_id,
        &first_request.runtime,
    );
    let first = store
        .prepare_projection(ProjectionWriteRequest::new(first_request, first_event))
        .expect("prepare first projection");
    let second_request = projection_request(
        "projection-2",
        "execution-2",
        &checkpoint.id,
        "Run the full test suite",
    );
    let second_event = projection_event(
        "event-projection-2",
        500,
        &second_request.projection_id,
        &checkpoint.id,
        &second_request.execution_id,
        &second_request.runtime,
    );
    let second = store
        .prepare_projection(ProjectionWriteRequest::new(second_request, second_event))
        .expect("prepare second projection");

    // Then: canonical bytes/hash are stable and only the applicable StateRef is selected.
    let expected = concat!(
        "# Tsukumo handoff v1\n",
        "Precedence: current user instructions and repository rules override this handoff.\n\n",
        "## Goal\nContinue Tsukumo MVP\n\n",
        "## Current progress\n- (none)\n\n",
        "## Decisions\n- (none)\n\n",
        "## Constraints\n",
        "- [state:state-gnu@v1] Use the GNU Rust toolchain on Windows\n\n",
        "## Artifacts\n- (none)\n\n",
        "## Open loops\n- (none)\n\n",
        "## Next actions\n- (none)\n\n",
        "## Delegation goal\nRun the full test suite\n",
    );
    assert_eq!(first.rendered_prompt().expose(), expected);
    assert_eq!(first.receipt.rendered_char_count, 380);
    assert_eq!(first.receipt.rendered_byte_count, 380);
    assert_eq!(
        first.receipt.rendered_digest.value,
        "e9d3389becd7f7aa529c3b71e2be4b5627a8ecf431035d19445fa7fb38843463"
    );
    assert_eq!(
        first.receipt.rendered_digest,
        second.receipt.rendered_digest
    );
    assert_eq!(first.receipt.sections, second.receipt.sections);
    assert_eq!(
        first.receipt.selected_state_refs,
        vec![StateRef::new(selected.state_id, selected.version)]
    );
    assert!(first.receipt.omissions.iter().any(|entry| {
        entry.state_id == omitted.state_id
            && entry.reason == ProjectionOmissionReason::ScopeMismatch
    }));
    assert!(first
        .receipt
        .sections
        .iter()
        .any(|section| section.section == ProjectionSection::Constraints));
    drop(store);

    let reopened = SoulStore::open(directory.path()).expect("reopen projection store");
    assert_eq!(
        reopened
            .projection_receipt(&first.receipt.id)
            .expect("load receipt"),
        Some(first.receipt)
    );
}
