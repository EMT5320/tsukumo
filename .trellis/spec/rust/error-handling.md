# Error Handling

## Public Fallible APIs

Library and host paths that perform I/O, parsing, storage, process control, or
evidence writes return `Result`. Define crate-owned error enums with
`thiserror`; preserve underlying sources using `#[from]` or `#[source]`.

Current reference patterns:

- `SessionError` in `crates/tsukumo-kernel/src/session.rs` includes JSONL line
  numbers and the original serde error.
- `AdapterError` in `crates/tsukumo-adapters/src/stream_json.rs` distinguishes
  I/O from line-scoped JSON failures.
- `SoulError` in `crates/tsukumo-soul/src/store.rs` wraps filesystem and SQLite
  errors and names domain validation failures.

## Propagation Rules

- Add boundary context once, near the owner that knows it (line number,
  runtime binding, event ID, database operation, or file path).
- Do not stringify an error early when callers still need to classify it.
- Do not turn malformed required fields into plausible durable defaults.
- Unknown optional vendor events may be skipped as documented compatibility
  behavior; malformed known events are errors.
- Evidence and receipt writes are part of the claimed operation. Never discard
  their result with `let _ = ...`; either fail the operation or return an
  explicit degraded outcome that the caller surfaces.

## Panics

- `unwrap`/`expect` are acceptable in unit tests, deterministic fixtures, and
  examples where failure should terminate the demo.
- Production library and host paths must return an error for user input,
  vendor drift, filesystem, database, process, or network failures.
- Internal `expect` is allowed only when the invariant is immediate and proven
  in the same function; include a precise message.

## Process and Permission Failures

- A runtime process exit, malformed stream line, permission denial, and user
  cancellation are distinct outcomes.
- Always terminate/reap child processes on cancellation or host shutdown.
- Permission denial is a normal controlled outcome, not a relationship-state
  error and not an automatic retry authorization.

