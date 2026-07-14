# C1 Cross-Runtime Evidence — Implementation Plan

## Preconditions

- [x] Contracts, handoff/projection, and host/runtime children are archived and
      committed.
- [x] Default evidence path is credential-free and contains no personal data.
- [x] Load runtime, persistence, architecture, error, and quality specs.
- [x] Read and freeze
      `research/trellis-baseline-and-utility-evaluation.md`; do not change
      conditions or thresholds after seeing outcome data without recording the
      deviation.

## Ordered Checklist

### 0. Freeze Utility Protocol and Start the Observation Clock

- [x] Freeze C0 Trellis-only, C1 automatic migration with evidence controls
      hidden, and C2 migration plus provenance/selective revoke.
- [x] Freeze operational definitions for episode start, first correct action,
      owner intervention, task quality, and unavailable token data.
- [x] Preassign a 12-episode minimum across C0/C1/C2, retain 20 as a stretch
      target, and predeclare wrong-scope, stale, and contradictory-state faults.
- [x] Seed at least one 48–72 hour delayed-resumption task by 2026-07-14.
- [x] Prepare a reviewed observation template for continuation, recovery,
      quality, and always-on overhead metrics; retain no raw prompt or secret
      and keep manual observation work within 20 minutes per day.

### 0.5. Codex Event-Surface Recon Spike (gate for step 1)

- [x] Before writing any decoder, capture real version-pinned
      `codex exec --json` event streams from throwaway prompts and map them
      against the KernelEvent payload checklist (tool start/end granularity,
      permission/approval mechanism, mid-turn input, outcome/error shapes).
- [x] Capture stdout JSONL and stderr diagnostics separately so local launcher
      warnings cannot contaminate the decoder fixture.
- [x] Record semantic gaps versus the Claude mapping (`control_request`
      equivalent, tool-call granularity, streaming text) and decide the
      normalization strategy before implementing step 1.
- [x] Spike output is a short findings note in this task directory; no product
      code changes.

### 1. Add Codex Runtime Support

- [x] Implement stdin-based `codex exec --json --ephemeral` profile with an
      explicit least-capable sandbox and version probe.
- [x] Implement stateful thread/turn/item/error decoder.
- [x] Preserve namespaced vendor provenance and classify skip/error/truncation.
- [x] Add redacted versioned Codex fixtures and per-event tests.
- [x] Add Claude/Codex normalized conformance tests.

### 2. Prove Deterministic Behavioral Sensitivity

- [x] Create a disposable deterministic Rust fixture repository and redacted
      runtime configuration.
- [x] Source an explicit Claude-side GNU constraint event.
- [x] Run/replay with-state Codex output and capture normalized evidence.
- [x] Run/replay without-state with only the target StateId removed.
- [x] Validate invariant inputs and compare selected refs, digests, normalized
      tool arguments, and outcomes.
- [x] Keep outputs temporary or commit only reviewed redacted fixtures.

This step proves that selected state entered the controlled decision path. It
does not by itself prove task utility or the necessity of traceability.

### 3. Prove Traceability and Revoke

- [x] Trace source EventId -> StateId -> CheckpointId -> ProjectionId ->
      ExecutionId -> tool/outcome.
- [x] Apply revoke through host service and prove the next projection excludes
      the old StateRef.
- [x] Reopen and inspect the historical receipt unchanged.
- [x] Verify permission decisions still cannot create auto-approve state.

### 3.5. Post-Review Correctness and Claim Hardening

- [x] Keep unsuccessful Codex tool facts sticky through `turn.completed`; Host,
      product read model, and Theater must not report or celebrate task success.
- [x] Add a versioned capture manifest that machine-checks retained fixture,
      repository, and replay projection digests while marking unavailable
      original-capture controls and causal eligibility explicitly.
- [x] Scope Claude/Codex live tests to connectivity because assistant text is
      intentionally ignored; update the historical evidence record to state
      that the requested token was not asserted.

### 4. Measure Trellis Baseline and Fault Recovery

- [x] Add a bounded `episode seed` / `episode resume` Host entry that
      reuses Chronicle, StateWriter, checkpoint, projection, and orchestrator
      production paths.
- [x] Prove C0 creates no Tsukumo storage/spawn and C1/C2 condition visibility
      does not change rendered migration bytes.
- [x] Pre-register E02/E03 as wait-for-C1/C2 with honest source-action and
      owner-review gates; do not mark them seeded from Markdown alone.
- [ ] After a real reviewed source action, run `episode seed` and fill the
      machine-derived timestamp/window before target resume.
- [ ] Record a 12-episode minimum and up to 20 handoff episodes where feasible,
      including at least four mid-task switches, two delayed resumptions, and
      two stale/scope/conflict cases; report shortfall honestly if the window
      closes first.
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

- [x] Run fixture secret/path scanning and prompt-sentinel tests.
- [x] Run all adapter/host/soul integration and workspace gates.
- [x] Run an opt-in live smoke only when both local CLI prerequisites exist;
      enabled missing prerequisites must fail clearly.
- [ ] Run `trellis-check`, record threshold outcomes and deviations, update
      executable specs only when contracts changed, commit, and archive.

## Validation Commands

```bash
git diff --check
cargo fmt --all -- --check
cargo test -p tsukumo-adapters codex
cargo test -p tsukumo-host comparison
cargo test -p tsukumo-host --test cli_parse_contract --test episode_runner_contract
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
- Manual observation work is capped at 20 minutes per day; product execution
  and required quality checks are accounted separately.
