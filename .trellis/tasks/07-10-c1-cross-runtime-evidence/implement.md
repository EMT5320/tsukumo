# C1 Cross-Runtime Evidence — Implementation Plan

## Preconditions

- [ ] Contracts, handoff/projection, and host/runtime children are archived and
      committed.
- [ ] Default evidence path is credential-free and contains no personal data.
- [ ] Load runtime, persistence, architecture, error, and quality specs.

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

### 1. Add Codex Runtime Support

- [ ] Implement stdin-based `codex exec --json --ephemeral` profile with an
      explicit least-capable sandbox and version probe.
- [ ] Implement stateful thread/turn/item/error decoder.
- [ ] Preserve namespaced vendor provenance and classify skip/error/truncation.
- [ ] Add redacted versioned Codex fixtures and per-event tests.
- [ ] Add Claude/Codex normalized conformance tests.

### 2. Build Deterministic Cross-Runtime Evidence

- [ ] Create a disposable deterministic Rust fixture repository and redacted
      runtime configuration.
- [ ] Source an explicit Claude-side GNU constraint event.
- [ ] Run/replay with-state Codex output and capture normalized evidence.
- [ ] Run/replay without-state with only the target StateId removed.
- [ ] Validate invariant inputs and compare selected refs, digests, normalized
      tool arguments, and outcomes.
- [ ] Keep outputs temporary or commit only reviewed redacted fixtures.

### 3. Prove Traceability and Revoke

- [ ] Trace source EventId -> StateId -> CheckpointId -> ProjectionId ->
      ExecutionId -> tool/outcome.
- [ ] Apply revoke through host service and prove the next projection excludes
      the old StateRef.
- [ ] Reopen and inspect the historical receipt unchanged.
- [ ] Verify permission decisions still cannot create auto-approve state.

### 4. Verification and Handoff

- [ ] Run fixture secret/path scanning and prompt-sentinel tests.
- [ ] Run all adapter/host/soul integration and workspace gates.
- [ ] Run an opt-in live smoke only when both local CLI prerequisites exist;
      enabled missing prerequisites must fail clearly.
- [ ] Run `trellis-check`, update specs, commit, and archive.

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
