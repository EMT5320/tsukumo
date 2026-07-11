# C1 Handoff and Projection — Implementation Plan

## Preconditions

- [x] Contracts/Chronicle child is archived and its schema/event contract is
      committed.
- [x] Read parent/child artifacts and Rust persistence/error/quality specs with
      `trellis-before-dev`.
- [x] Reopen the positive GNU StateRecord fixture and confirm stable IDs before
      building downstream references.

## Ordered Checklist

### 1. Add Checkpoint Domain Types and Storage

- [x] Implement checkpoint, progress/decision/artifact/open-loop/next-action
      types and version identity.
- [x] Implement compiler validation and exhaustive open-loop transitions.
- [x] Implement immutable SQLite checkpoint rows, StateRef/source-event edges,
      version uniqueness and reopen queries.
- [x] Add low-frequency trigger enum/seam without per-tool LLM calls.

### 2. Add Deterministic State Selection

- [x] Implement active/expiry/scope filters and checkpoint-pinned constraints.
- [x] Implement stable ranking/tie-breakers and character budget admission.
- [x] Return selected refs plus deterministic omission reasons.
- [x] Add focused scope/revoke/expiry/ranking/budget regression tests.

### 3. Implement Canonical Projection

- [x] Freeze section order, LF/final-newline normalization and renderer version.
- [x] Resolve StateRefs at render time and exclude theater/persona text.
- [x] Return `SensitiveText`; ensure debug/error paths redact content.
- [x] Add SHA-256 dependency/utility and compute overall/section digests,
      bytes/chars and explicit budget unit.
- [x] Add golden and mutation tests.

### 4. Persist Production Receipts

- [x] Implement immutable receipt and selected-state edge schema/repository.
- [x] Validate checkpoint/state/runtime/execution references in one transaction.
- [x] Expose only `PreparedProjection` after receipt commit.
- [x] Prove historical receipts survive state revoke/supersede unchanged.
- [x] Add sentinel tests showing no prompt/secret in schema, rows, Chronicle,
      logs, errors or debug output.

### 5. Add the Deterministic Comparison Seam

- [x] Prepare with-state/without-state projections from one frozen input set.
- [x] Remove only one target StateId and report selected-ref/digest differences.
- [x] Add an invariant checker for every controlled non-target input.
- [x] Keep outputs temporary or in reviewed redacted fixtures; add no snapshot
      table, expiry scheduler, retain API, or cleanup audit.
- [x] Add prompt-sentinel, fixture/path, and secret validation.

### 6. Compatibility and Quality Gate

- [x] Mark `BriefCompiler`/prompt assembly as A1 compatibility-only and direct
      production hosts to `SoulStore::prepare_projection`.
- [x] Mark legacy inject traces as size telemetry with no projection claim.
- [x] Run cross-layer persistence/reopen and synthetic comparison tests.
- [x] Run `trellis-check` and update executable specs.
- [ ] Commit and archive child.

## Validation Commands

```powershell
git diff --check
cargo +stable-x86_64-pc-windows-gnu fmt --all -- --check
cargo +stable-x86_64-pc-windows-gnu check --workspace --all-targets --offline
cargo +stable-x86_64-pc-windows-gnu clippy --workspace --all-targets --offline -- -D warnings
cargo +stable-x86_64-pc-windows-gnu test --workspace --offline
$env:RUSTDOCFLAGS = '-D warnings'; cargo +stable-x86_64-pc-windows-gnu doc --workspace --no-deps --offline
python ./.trellis/scripts/task.py validate 07-10-c1-handoff-projection
```

Test filters are illustrative until module/test names land; the unfiltered
workspace command is the authoritative gate.

## Risky Files and Rollback

- Renderer output is a versioned contract. Never update golden/hash expectations
  without bumping the renderer/projection version and explaining why.
- The `SensitiveText` exposure point must stay narrow; code review every call.
- Receipt insertion and selected-ref insertion share one transaction. A partial
  receipt is invalid.
- V0 must not add persistent prompt-snapshot columns/tables or a second durable
  artifact authority.
