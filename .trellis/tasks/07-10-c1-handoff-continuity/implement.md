# C1 Handoff Continuity — Execution Plan

## Start Rule

Do not start this parent task for implementation. After owner review, start the
first child that owns code:

```bash
python3 ./.trellis/scripts/task.py start 07-10-c1-contracts-chronicle
```

Every child must be implemented, checked, committed, finished, and archived
before the dependent child starts. At each boundary, review parent acceptance
criteria and evidence-link compatibility.

## Ordered Delivery

### 1. Contracts and Chronicle

- [ ] Implement typed identity, event envelope/payload and shared sensitive
      value contract.
- [ ] Migrate all existing fixtures, adapters, theater consumers, examples and
      session replay to the versioned envelope.
- [ ] Add SQLite migrations, append-only Chronicle, StateRecord/StateWriter,
      hybrid extractor seam and derived exports.
- [ ] Prove legacy migration does not fabricate explicit/hard constraints.
- [ ] Pass child quality gate and archive the child.

Integration gate: a persisted explicit GNU constraint reopens with identical
IDs/scope/evidence and can be replayed through theater without vendor JSON.

### 2. Handoff and Projection

- [ ] Implement checkpoint version/open-loop invariants and deterministic state
      selection.
- [ ] Implement canonical renderer, production receipt, SHA-256/section
      metadata and receipt-before-launch API.
- [ ] Implement redacted expiring CaseBundle snapshots and synthetic
      removed-state comparison.
- [ ] Pass privacy, persistence and deterministic-golden tests; archive child.

Integration gate: a prepared Codex projection contains the GNU StateRef and a
durable receipt, while all persisted/logged representations exclude a sentinel
prompt secret.

### 3. Host and Claude Runtime

- [ ] Add `tsukumo-host` composition root and process ports.
- [ ] Refactor Claude decoder to incremental shared fixture/live operation.
- [ ] Wire receipt -> stdin -> process -> payload -> envelope -> Chronicle ->
      theater.
- [ ] Implement lifecycle/reap and deterministic Safety Plane seams.
- [ ] Pass fake-process, recorded fixture and opt-in Claude smoke gates; archive
      child.

Integration gate: the first Claude event is committed/rendered before process
exit, and every terminal/cancel/failure path reaps exactly once.

### 4. Cross-Runtime, UI and Final Evidence

- [ ] Add Codex `exec --json` profile/decoder and shared conformance tests.
- [ ] Run fixture-driven and opt-in real Claude -> Codex handoff.
- [ ] Produce with-state/without-state CaseBundle and post-revoke projection.
- [ ] Add minimal state/projection/permission TUI over host read models/actions.
- [ ] Track lockfile, pin proven toolchain, add Linux + Windows GNU CI and pass
      full workspace gate.
- [ ] Archive child, then perform parent acceptance review.

## Parent Acceptance Review

- [ ] Trace one positive run from source `EventId` through `StateId`,
      `CheckpointId`, `ProjectionId`, `ExecutionId`, tool correlation and
      outcome.
- [ ] Trace the removed-state pair and verify its invariant manifest.
- [ ] Trace revoke/supersede and prove old receipts remain explainable while new
      projections exclude inactive state.
- [ ] Inspect Safety Plane evidence and prove approvals never enter automatic
      relationship extraction.
- [ ] Inspect committed fixtures/CaseBundles and run secret/path validation.
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
