# C1 Host and Runtime — Implementation Plan

## Preconditions

- [ ] Contracts and handoff/projection children are archived and committed.
- [ ] Read parent/child artifacts, runtime research and Rust architecture,
      runtime, persistence, error and quality specs via `trellis-before-dev`.
- [ ] Confirm a prepared projection cannot exist before receipt commit.

## Ordered Checklist

### 1. Scaffold the Host

- [ ] Add `tsukumo-host` library/binary to the workspace with one-way
      dependencies on kernel/adapters/soul/theater.
- [ ] Add typed config, injected clock and testable orchestrator ports.
- [ ] Keep `main.rs` thin and all orchestration library-testable.

### 2. Build Generic Process Lifecycle

- [ ] Implement command spec, process runner/handle and incremental
      stdout/stderr channel.
- [ ] Deliver prompt through stdin only; close it after the one-shot payload.
- [ ] Bound/redact diagnostics and keep prompt out of args/env/debug/errors.
- [ ] Implement timeout, cancellation, termination and exactly-once reap.
- [ ] Build deterministic fake-child/fake-runner tests before live CLI wiring.

### 3. Refactor the Claude Adapter

- [ ] Add Claude runtime profile and safe command flags/version probe.
- [ ] Refactor current `Read -> Vec` parser into stateful line decoder.
- [ ] Preserve vendor IDs as provenance and classify unknown vs malformed known
      events.
- [ ] Add reviewed redacted Claude JSONL fixture and decoder tests.

### 4. Wire the End-to-End Host Loop

- [ ] Accept `PreparedProjection`, append lifecycle event and spawn only after
      receipt commit.
- [ ] For each decoded payload, assign envelope IDs/correlation, commit
      Chronicle, then drive Director/StageWorld immediately.
- [ ] Reconcile terminal vendor event with process exit and append one outcome.
- [ ] Stop/cancel on Chronicle failure so theater cannot outrun durable truth.
- [ ] Trace tool start/end/outcome back to execution and projection.

### 5. Implement Safety Plane Seam

- [ ] Add typed permission request/risk/decision/session-grant state machine.
- [ ] Route decisions through host and Chronicle; exclude them from auto-state
      extraction.
- [ ] Add once/session/deny, stale request and repeated request tests.
- [ ] Add explicit degraded/unsupported result for an unwired vendor bridge.
- [ ] Verify command specs never use dangerous permission-bypass flags.

### 6. Runtime Verification and Handoff

- [ ] Prove first event arrives before fake process exits.
- [ ] Prove launch failure, malformed/truncated stream, timeout, cancel,
      non-zero exit and stderr overflow all reap and persist distinct outcomes.
- [ ] Run default recorded-fixture integration.
- [ ] If local Claude prerequisites exist, run opt-in safe smoke; otherwise
      verify the enabled gate reports an actionable failure.
- [ ] Run `trellis-check`, update specs, commit and archive child.

## Validation Commands

```bash
git diff --check
cargo fmt --all -- --check
cargo test -p tsukumo-adapters
cargo test -p tsukumo-host
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 ./.trellis/scripts/task.py validate 07-10-c1-host-runtime
```

Optional live gate:

```bash
TSUKUMO_RUN_LIVE_SMOKE=1 cargo test -p tsukumo-host --test claude_live -- --ignored --nocapture
```

## Risky Files and Rollback

- `process.rs` owns child resources. Tests must demonstrate cleanup before a
  live run is accepted on each target.
- Adapter refactor must keep fixture and live decoders identical; do not retain
  a second batch-only parser.
- Chronicle-before-theater ordering is an invariant. If fan-out becomes
  concurrent, preserve commit acknowledgment before presentation.
- A failed live smoke never mutates reviewed fixtures automatically.
