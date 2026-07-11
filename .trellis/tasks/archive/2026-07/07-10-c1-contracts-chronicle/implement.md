# C1 Contracts and Chronicle — Implementation Plan

## Preconditions

- [x] Read parent PRD/design and Rust architecture, event, persistence, error,
      and quality specs via `trellis-before-dev`.
- [x] Confirm this child is the active `in_progress` task; preserve unrelated
      untracked tool directories.
- [x] Capture current fixture/test baseline and current environmental blockers.

## Ordered Checklist

### 1. Normalize the Baseline

- [x] Run rustfmt and isolate existing formatting drift from semantic changes.
- [x] Record existing workspace test/check results honestly; do not call a
      network/toolchain setup failure a code failure or pass.

### 2. Add Shared Kernel Types

- [x] Add ID newtypes, timestamp/content-safe shared values, `RuntimeBinding`,
      correlation and projection references.
- [x] Add redacted `SensitiveText` with explicit exposure and no implicit serde;
      validated persisted types perform deliberate conversion.
- [x] Split `KernelEventPayload` from the versioned `KernelEvent` envelope.
- [x] Add/normalize lifecycle, permission, state, checkpoint, projection and
      outcome payloads required by parent R1/R2.
- [x] Add serde round-trip and redaction tests.

### 3. Migrate All Existing Consumers Atomically

- [x] Update session JSONL helpers to read/write envelopes with line context.
- [x] Update Claude-like and synthetic adapters to return payloads; keep durable
      envelope assignment out of adapters.
- [x] Update Director, drive helpers, examples and integration tests.
- [x] Rewrite committed fixtures with deterministic envelope IDs/timestamps and
      correlation; add replay compatibility tests.

### 4. Introduce Versioned SQLite Storage

- [x] Add connection/migration module, foreign-key setup and ordered
      `schema_migrations`.
- [x] Create Chronicle/state/evidence tables; leave checkpoint/receipt tables to
      child 2's next ordered migration.
- [x] Implement append/replay/query and duplicate identical/conflicting event
      behavior.
- [x] Add transaction/unit-of-work API for event + state lifecycle writes.
- [x] Make every evidence write error observable; remove `let _ =` behavior
      from durable trace paths.

### 5. Implement State Formation

- [x] Add `StateKey`, kind, subject/applicability scope, strength, status, TTL,
      evidence and provenance types.
- [x] Add rule extractor plus provider-neutral structured-LLM/recorded extractor
      seam returning `StateDraft` only.
- [x] Add deterministic StateWriter create/conflict/supersede/revoke validation.
- [x] Exclude permission decisions from auto-state extraction.
- [x] Cover explicit GNU constraint, inferred hard-constraint rejection,
      malformed/timeout extraction and secret/scope failures.

### 6. Migrate Probe Data and Derived Views

- [x] Add idempotent legacy `facts` importer with `legacy_imported` events and
      low-strength state only.
- [x] Refactor recall/brief compatibility facade to query canonical C1 state or
      clearly isolate legacy reads during transition.
- [x] Rebuild FTS, JSONL and Markdown from SQLite; never import manual edits
      implicitly.
- [x] Test export failure after commit as a recoverable stale projection.

### 7. Cross-Layer Check and Handoff

- [x] Trace the positive GNU event/state chain after close/reopen.
- [x] Run full child and workspace tests, clippy, format and diff checks.
- [x] Run `trellis-check` and update specs for discoveries.
- [x] Commit after explicit confirmation.
- [ ] Archive during `finish-work`.
- [x] Record any schema/wire details needed by the handoff/projection child.

## Validation Commands

```bash
git diff --check
cargo fmt --all -- --check
cargo test -p tsukumo-kernel
cargo test -p tsukumo-adapters
cargo test -p tsukumo-soul
cargo test -p tsukumo-theater
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 ./.trellis/scripts/task.py validate 07-10-c1-contracts-chronicle
```

## Risky Files and Rollback

- `crates/tsukumo-kernel/src/event.rs` and every fixture are one wire-contract
  boundary; revert them together if the migration fails.
- `crates/tsukumo-soul/src/store.rs` currently uses `INSERT OR REPLACE` and
  rewrites snapshots. Preserve old APIs behind a facade until new transactional
  tests pass; do not delete legacy data.
- SQLite migrations are forward/additive and idempotent. Test on copies; never
  implement a data-dropping rollback.
- Keep source comments/doc comments in English to match the codebase.
