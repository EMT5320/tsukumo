# C1 Handoff and Projection — Progress

## Completed

- Added versioned immutable checkpoints with explicit open-loop transitions,
  StateRef/source-event edges, and low-frequency trigger identities.
- Added deterministic state selection, canonical rendering, SHA-256 overall and
  section digests, character-budget admission, and metadata-only receipts.
- Added receipt-first `PreparedProjection`, sensitive delegation-goal redaction
  metadata, deterministic removed-state comparison, and A1 compatibility
  boundary documentation.
- Split migration SQL and handoff domain/validation modules to keep new Rust
  modules focused and below the project size ceiling.

## Review Fixes

- Added update/delete guards for checkpoint/receipt parent and ordered edge
  tables; schema tests also forbid prompt/snapshot authority.
- Bound idempotent checkpoint and receipt retries to the original Chronicle
  EventId and revalidated same-ID envelope content before returning success.
- Added direct expiry, specificity ranking, sensitive Debug/error, rollback,
  reopen, and Soul-to-Chronicle-to-Theater regressions.

## Validation Evidence

Verified on 2026-07-11 with `stable-x86_64-pc-windows-gnu`, offline:

- `cargo fmt --all -- --check` — passed.
- `cargo check --workspace --all-targets --offline` — passed.
- `cargo clippy --workspace --all-targets --offline -- -D warnings` — passed.
- `cargo test --workspace --offline` — 105 passed, 0 failed.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --offline` — passed.
- `task.py validate 07-10-c1-handoff-projection` — passed.

## Remaining

- Commit the verified implementation and documentation.
- Finish/archive this child, then start `07-10-c1-host-runtime` in a later
  development step.
