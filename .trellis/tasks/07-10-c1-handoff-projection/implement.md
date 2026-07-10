# C1 Handoff and Projection — Implementation Plan

## Preconditions

- [ ] Contracts/Chronicle child is archived and its schema/event contract is
      committed.
- [ ] Read parent/child artifacts and Rust persistence/error/quality specs with
      `trellis-before-dev`.
- [ ] Reopen the positive GNU StateRecord fixture and confirm stable IDs before
      building downstream references.

## Ordered Checklist

### 1. Add Checkpoint Domain Types and Storage

- [ ] Implement checkpoint, progress/decision/artifact/open-loop/next-action
      types and version identity.
- [ ] Implement compiler validation and exhaustive open-loop transitions.
- [ ] Implement immutable SQLite checkpoint rows, StateRef/source-event edges,
      version uniqueness and reopen queries.
- [ ] Add low-frequency trigger enum/seam without per-tool LLM calls.

### 2. Add Deterministic State Selection

- [ ] Implement active/expiry/scope filters and checkpoint-pinned constraints.
- [ ] Implement stable ranking/tie-breakers and character budget admission.
- [ ] Return selected refs plus deterministic omission reasons.
- [ ] Add table-driven scope/revoke/expiry/ranking/budget tests.

### 3. Implement Canonical Projection

- [ ] Freeze section order, LF/final-newline normalization and renderer version.
- [ ] Resolve StateRefs at render time and exclude theater/persona text.
- [ ] Return `SensitiveText`; ensure debug/error paths redact content.
- [ ] Add SHA-256 dependency/utility and compute overall/section digests,
      bytes/chars and explicit budget unit.
- [ ] Add golden and mutation tests.

### 4. Persist Production Receipts

- [ ] Implement immutable receipt and selected-state edge schema/repository.
- [ ] Validate checkpoint/state/runtime/execution references in one transaction.
- [ ] Expose only `PreparedProjection` after receipt commit.
- [ ] Prove historical receipts survive state revoke/supersede unchanged.
- [ ] Add sentinel tests showing no prompt/secret in schema, rows, Chronicle,
      logs, errors or debug output.

### 5. Add Debug/Eval CaseBundle Artifacts

- [ ] Implement named deterministic redaction profile and redaction manifest.
- [ ] Write only redacted canonical snapshots, with independent digest.
- [ ] Implement seven-day default expiry, controlled-clock cleanup and explicit
      retain choice.
- [ ] Build synthetic with-state/without-state bundle and invariant checker.
- [ ] Add fixture/path/secret validation.

### 6. Compatibility and Quality Gate

- [ ] Route `BriefCompiler`/prompt assembly compatibility through the new
      selector/renderer or mark the facade deprecated without duplicate logic.
- [ ] Replace `inject_trace.jsonl` claims with receipt/Chronicle evidence.
- [ ] Run cross-layer persistence/reopen and synthetic CaseBundle tests.
- [ ] Run `trellis-check`, update specs, commit and archive child.

## Validation Commands

```bash
git diff --check
cargo fmt --all -- --check
cargo test -p tsukumo-soul checkpoint
cargo test -p tsukumo-soul projection
cargo test -p tsukumo-soul receipt
cargo test -p tsukumo-soul case_bundle
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 ./.trellis/scripts/task.py validate 07-10-c1-handoff-projection
```

Test filters are illustrative until module/test names land; the unfiltered
workspace command is the authoritative gate.

## Risky Files and Rollback

- Renderer output is a versioned contract. Never update golden/hash expectations
  without bumping the renderer/projection version and explaining why.
- The `SensitiveText` exposure point must stay narrow; code review every call.
- Receipt insertion and selected-ref insertion share one transaction. A partial
  receipt is invalid.
- Cleanup deletes only snapshot artifact bytes, never receipts/checkpoints or
  audit metadata.
