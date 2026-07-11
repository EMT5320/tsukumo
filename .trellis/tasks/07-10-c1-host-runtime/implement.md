# C1 Host and Runtime — Implementation Plan

## Preconditions

- [x] Contracts and handoff/projection children are archived and committed.
- [x] Read parent/child artifacts, runtime research and Rust architecture,
      runtime, persistence, error and quality specs via `trellis-before-dev`.
- [x] Confirm a prepared projection cannot exist before receipt commit.

## Ordered Checklist

### 1. Scaffold the Host

- [x] Add `tsukumo-host` library/binary to the workspace with one-way
      dependencies on kernel/adapters/soul/theater.
- [x] Add typed config, injected clock and testable orchestrator ports.
- [x] Keep `main.rs` thin and all orchestration library-testable.

### 2. Build Generic Process Lifecycle

- [x] Implement command spec, process runner/handle and incremental
      stdout/stderr channel.
- [x] Deliver prompt through stdin only; close it after the one-shot payload.
- [x] Bound/redact diagnostics and keep prompt out of args/env/debug/errors.
- [x] Implement timeout, cancellation, termination and exactly-once reap.
- [x] Build deterministic fake-child/fake-runner tests before live CLI wiring.

### 3. Refactor the Claude Adapter

- [x] Add Claude runtime profile and safe command flags/version probe.
- [x] Refactor current `Read -> Vec` parser into stateful line decoder.
- [x] Preserve vendor IDs as provenance and classify unknown vs malformed known
      events.
- [x] Add reviewed redacted Claude JSONL fixture and decoder tests.

### 4. Wire the End-to-End Host Loop

- [x] Accept `PreparedProjection`, append lifecycle event and spawn only after
      receipt commit.
- [x] For each decoded payload, assign envelope IDs/correlation, commit
      Chronicle, then drive Director/StageWorld immediately.
- [x] Reconcile terminal vendor event with process exit and append one outcome.
- [x] Stop/cancel on Chronicle failure so theater cannot outrun durable truth.
- [x] Trace tool start/end/outcome back to execution and projection.

### 5. Implement Safety Plane Seam

- [x] Add typed permission request/risk/decision/session-grant state machine.
- [x] Route decisions through host and Chronicle; exclude them from auto-state
      extraction.
- [x] Add once/session/deny, stale request and repeated request tests.
- [x] Add explicit degraded/unsupported result for an unwired vendor bridge.
- [x] Verify command specs never use dangerous permission-bypass flags.

### 6. Runtime Verification and Handoff

- [x] Prove first event arrives before fake process exits.
- [x] Prove launch failure, malformed/truncated stream, timeout, cancel,
      non-zero exit and stderr overflow all reap and persist distinct outcomes.
- [x] Run default recorded-fixture integration.
- [x] Run the opt-in safe smoke against local Claude 2.1.205 after renewed
      risk approval. The allowlisted synthetic projection, empty temporary
      directory, safe mode, zero tools, and USD 0.05 cap completed successfully
      in 7.95 seconds on 2026-07-11.
- [x] Run `trellis-check` and update the applicable Rust specs.
- [ ] Commit and archive the child after the authorized live-smoke decision.

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
