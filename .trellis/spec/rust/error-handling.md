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
- `SoulError` in `crates/tsukumo-soul/src/storage.rs` wraps filesystem and
  SQLite errors and names domain validation, migration, budget, and legacy
  sensitive-content failures.

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


## C1 typed failure distinctions

- `DecodeError` separates invalid JSON, missing known fields, invalid known
  fields, unsupported known values, and oversized lines; `AdapterError` adds
  the exact source line.
- `EventContractError` separates unsupported schema, missing attribution,
  invalid durable fields, and sensitive content. JSONL load, Chronicle append,
  and Chronicle replay all apply the same event gate.
- `ExtractError` is provider-neutral. `extract_non_blocking` converts malformed,
  timeout, and unavailable extraction into a recoverable observable error
  payload while returning no partial state.
- `StateValidationError` identifies missing/permission/future evidence,
  repeated-strength failure, evidence-chain mismatch, metadata/scope failure,
  conflict, backdated transition, inactive state, expiry, and lifecycle
  mismatch. Migration fails with `InvalidStoredValue` when an inactive
  schema-one interval cannot be reconstructed.
- `HandoffError` separates shape/secret failures, missing source/state refs,
  version/quest mismatches, incomplete open-loop transitions, and immutable
  checkpoint conflicts.
- `ProjectionError` separates checkpoint/state lookup, event mismatch, budget
  refusal, immutable receipt conflict, stored-edge corruption, and comparison
  invariant failure. Receipt-first APIs never return `PreparedProjection` on
  any of these errors.
- Idempotent checkpoint/receipt retries keep the original creation EventId and
  re-run Chronicle duplicate validation; same-ID/different-envelope attempts
  surface `SoulError::ConflictingEvent`.
- `OutcomeStatus` reserves distinct wire values for `PermissionDenied`,
  `SafetyUnsupported`, and `Degraded`. Do not encode these as cancellation,
  generic failure, or summary text.
- `HostError` separates receipt/runtime preflight, duplicate execution,
  permission-evidence/scope failure, clock failure, and Chronicle failure.
  Chronicle/clock failures while a process is live retain both cleanup status
  and the typed cleanup error. Controlled runtime failures return one
  `ExecutionReport` with distinct status/failure/detail fields.
