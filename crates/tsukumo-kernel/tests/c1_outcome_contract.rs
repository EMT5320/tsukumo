//! Stable terminal outcome wire values reserved for the C1 host.

use tsukumo_kernel::OutcomeStatus;

#[test]
fn permission_safety_and_degraded_outcomes_have_distinct_wire_values() {
    // Given: terminal statuses that must remain distinct from cancellation and failure.
    let statuses = [
        OutcomeStatus::PermissionDenied,
        OutcomeStatus::SafetyUnsupported,
        OutcomeStatus::Degraded,
    ];

    // When: the statuses cross the shared event wire boundary.
    let values = statuses
        .into_iter()
        .map(|status| serde_json::to_string(&status).expect("serialize outcome status"))
        .collect::<Vec<_>>();

    // Then: later hosts can persist each controlled outcome without free-text inference.
    assert_eq!(
        values,
        vec![
            "\"permission_denied\"",
            "\"safety_unsupported\"",
            "\"degraded\"",
        ]
    );
}
