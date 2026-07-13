# C1 Handoff Continuity — Execution Plan

## Start Rule

Do not start this parent task for implementation. Contracts/Chronicle,
Handoff/Projection, Host/Runtime, and MVP TUI are archived; the next code-owning
child is:

```powershell
python ./.trellis/scripts/task.py start 07-10-c1-cross-runtime-evidence
```

Every child must be implemented, checked, committed, finished, and archived
before the dependent child starts. At each boundary, review parent acceptance
criteria and evidence-link compatibility.

Full V0 release packaging waits for the 2026-07-23 GO/PIVOT/NO-GO gate. Minimal
demo capture may happen during evidence freeze without activating the release
child.

## Ordered Delivery

### 1. Contracts and Chronicle

- [x] Implement typed identity, event envelope/payload and shared sensitive
      value contract.
- [x] Migrate all existing fixtures, adapters, theater consumers, examples and
      session replay to the versioned envelope.
- [x] Add SQLite migrations, append-only Chronicle, StateRecord/StateWriter,
      hybrid extractor seam and derived exports.
- [x] Prove legacy migration does not fabricate explicit/hard constraints.
- [x] Pass child quality gate and archive the child.

Integration gate: a persisted explicit GNU constraint reopens with identical
IDs/scope/evidence and can be replayed through theater without vendor JSON.

### 2. Handoff and Projection

- [x] Implement checkpoint version/open-loop invariants and deterministic state
      selection.
- [x] Implement canonical renderer, production receipt, SHA-256/section
      metadata and receipt-before-launch API.
- [x] Implement a deterministic with-state/without-state comparison seam with
      no general prompt-snapshot persistence.
- [x] Pass privacy, persistence and deterministic-golden tests; archive child.

Integration gate: a prepared Codex projection contains the GNU StateRef and a
durable receipt, while all persisted/logged representations exclude a sentinel
prompt secret.

### 3. Host and Claude Runtime

- [x] Add `tsukumo-host` composition root and process ports.
- [x] Refactor Claude decoder to incremental shared fixture/live operation.
- [x] Wire receipt -> stdin -> process -> payload -> envelope -> Chronicle ->
      theater.
- [x] Implement lifecycle/reap and deterministic Safety Plane seams.
- [x] Pass fake-process, recorded fixture and opt-in Claude smoke gates; archive
      child.

Integration gate: the first Claude event is committed/rendered before process
exit, and every terminal/cancel/failure path reaps exactly once.

### 4. Cross-Runtime Evidence

- [x] Add Codex `exec --json` profile/decoder and shared conformance tests.
- [ ] Run fixture-driven and opt-in real Claude -> Codex handoff.
- [x] Produce the controlled with-state/without-state comparison and post-revoke
      projection without a persistent snapshot subsystem.
- [ ] Archive the child after traceability, privacy, and claim-boundary checks.

### 5. MVP TUI

- [x] Add host read models and typed UI actions.
- [x] Add workshop status, state/projection inspectors, and permission modal.
- [x] Prove terminal restoration, resize, compact layout, and CJK alignment.
- [x] Archive the child after functional and visual checks.

### 6. V0 Release Packaging

- [ ] Finalize an installable `tsukumo` binary and package metadata.
- [ ] Add README, MIT LICENSE, tracked lockfile, and proven toolchain declaration.
- [ ] Add Linux + Windows GNU credential-free CI and clean-checkout smoke.
- [ ] Archive the child, then perform parent acceptance review.

## Parent Acceptance Review

- [x] Trace one positive run from source `EventId` through `StateId`,
      `CheckpointId`, `ProjectionId`, `ExecutionId`, tool correlation and
      outcome.
- [x] Trace the removed-state pair and verify its invariant manifest.
- [x] Trace revoke/supersede and prove old receipts remain explainable while new
      projections exclude inactive state.
- [x] Inspect Safety Plane evidence and prove approvals never enter automatic
      relationship extraction.
- [x] Inspect committed comparison fixtures and run secret/path validation.
- [ ] Run full quality gate from a clean checkout on the recorded toolchain.
- [ ] Update specs for implementation learnings, commit, finish/archive parent,
      and record the developer journal.

## Full Validation

```bash
git diff --check
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 ./.trellis/scripts/task.py validate 07-10-c1-handoff-continuity
```

The opt-in live gate is additional and never substitutes for deterministic CI:

```bash
TSUKUMO_RUN_LIVE_SMOKE=1 cargo test -p tsukumo-host --test cross_runtime_live -- --ignored --nocapture
```

## Risk and Rollback Points

- Event wire migration touches all crates: keep it in one child/commit series
  with coordinated fixtures; never leave mixed envelope/payload consumers.
- SQLite changes are additive; never use a destructive down migration. Back up
  a real `soul.db` before a live migration smoke.
- Receipt/renderer versions are immutable. Fix a renderer by adding a version,
  not rewriting historical hashes.
- Runtime cleanup is target-sensitive. A failed reap/process-tree test blocks
  live readiness for that target.
- Live outputs are evidence artifacts, not golden replacements. A flaky live
  run cannot overwrite reviewed deterministic fixtures automatically.
