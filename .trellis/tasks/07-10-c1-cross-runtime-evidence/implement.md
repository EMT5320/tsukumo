# C1 Cross-Runtime Evidence — Implementation Plan

## Preconditions

- [ ] Contracts, handoff/projection, and host/runtime children are archived and
      committed.
- [ ] Default evidence path is credential-free and contains no personal data.
- [ ] Load runtime, persistence, architecture, error, and quality specs.
- [ ] Read and freeze
      `research/trellis-baseline-and-utility-evaluation.md`; do not change
      conditions or thresholds after seeing outcome data without recording the
      deviation.

## Ordered Checklist

### 0. Codex Event-Surface Recon Spike (gate for step 1, added 2026-07-13)

- [ ] Before writing any decoder, capture real version-pinned
      `codex exec --json` event streams from throwaway prompts and map them
      against the KernelEvent payload checklist (tool start/end granularity,
      permission/approval mechanism, mid-turn input, outcome/error shapes).
- [ ] Record semantic gaps versus the Claude mapping (`control_request`
      equivalent, tool-call granularity, streaming text) and decide the
      normalization strategy before implementing step 1.
- [ ] Spike output is a short findings note in this task directory; no product
      code changes.

### 0.5. Freeze Utility Protocol (gate for product claims)

- [ ] Freeze C0 Trellis-only, C1 automatic migration with evidence controls
      hidden, and C2 migration plus provenance/selective revoke.
- [ ] Select natural handoff episodes and predeclare wrong-scope, stale, and
      contradictory-state fault cases.
- [ ] Seed at least one 48–72 hour delayed-resumption task by 2026-07-14.
- [ ] Prepare a reviewed observation template for continuation, recovery,
      quality, and always-on overhead metrics; retain no raw prompt or secret.

### 1. Add Codex Runtime Support

- [ ] Implement stdin-based `codex exec --json --ephemeral` profile with an
      explicit least-capable sandbox and version probe.
- [ ] Implement stateful thread/turn/item/error decoder.
- [ ] Preserve namespaced vendor provenance and classify skip/error/truncation.
- [ ] Add redacted versioned Codex fixtures and per-event tests.
- [ ] Add Claude/Codex normalized conformance tests.

### 2. Prove Deterministic Behavioral Sensitivity

- [ ] Create a disposable deterministic Rust fixture repository and redacted
      runtime configuration.
- [ ] Source an explicit Claude-side GNU constraint event.
- [ ] Run/replay with-state Codex output and capture normalized evidence.
- [ ] Run/replay without-state with only the target StateId removed.
- [ ] Validate invariant inputs and compare selected refs, digests, normalized
      tool arguments, and outcomes.
- [ ] Keep outputs temporary or commit only reviewed redacted fixtures.

This step proves that selected state entered the controlled decision path. It
does not by itself prove task utility or the necessity of traceability.

### 3. Prove Traceability and Revoke

- [ ] Trace source EventId -> StateId -> CheckpointId -> ProjectionId ->
      ExecutionId -> tool/outcome.
- [ ] Apply revoke through host service and prove the next projection excludes
      the old StateRef.
- [ ] Reopen and inspect the historical receipt unchanged.
- [ ] Verify permission decisions still cannot create auto-approve state.

### 4. Measure Trellis Baseline and Fault Recovery

- [ ] Record 12–20 handoff episodes where feasible, including at least four
      mid-task switches, two delayed resumptions, and two stale/scope/conflict
      cases; report shortfall honestly if the window closes first.
- [ ] Compare C1 vs C0 on time to first correct action, owner interventions,
      stale-state errors, context-reading tokens, and task quality.
- [ ] Compare C2 vs C1 on bad-state diagnosis/recovery time, collateral
      revokes, and next-handoff recurrence.
- [ ] Record normal latency, token, storage, cognitive, and adapter-maintenance
      overhead separately from injected-fault recovery.
- [ ] Freeze evidence and demo on 2026-07-22; write the 2026-07-23
      GO/PIVOT/NO-GO decision without upgrading injected faults into an
      incidence claim.

### 5. Verification and Handoff

- [ ] Run fixture secret/path scanning and prompt-sentinel tests.
- [ ] Run all adapter/host/soul integration and workspace gates.
- [ ] Run an opt-in live smoke only when both local CLI prerequisites exist;
      enabled missing prerequisites must fail clearly.
- [ ] Run `trellis-check`, record threshold outcomes and deviations, update
      executable specs only when contracts changed, commit, and archive.

## Validation Commands

```bash
git diff --check
cargo fmt --all -- --check
cargo test -p tsukumo-adapters codex
cargo test -p tsukumo-host comparison
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 ./.trellis/scripts/task.py validate 07-10-c1-cross-runtime-evidence
```

Optional live evidence gate:

```bash
TSUKUMO_RUN_LIVE_SMOKE=1 cargo test -p tsukumo-host --test cross_runtime_live -- --ignored --nocapture
```

## Risk and Rollback

- Vendor event schemas remain versioned inputs; unknown and malformed classes
  keep distinct behavior.
- Any uncontrolled with/without input difference invalidates the comparison.
- No V0 code may persist raw prompt snapshots or introduce a second authority.
- Trace completeness without continuation/recovery benefit cannot be reported
  as product utility. Permission approval and companion expansion remain P2
  until the trusted-handoff gate is decided.
