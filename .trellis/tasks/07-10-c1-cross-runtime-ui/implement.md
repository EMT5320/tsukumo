# C1 Cross-Runtime and UI — Implementation Plan

## Preconditions

- [ ] All earlier C1 children are archived and committed.
- [ ] Read parent/child artifacts, runtime research and all affected Rust specs
      via `trellis-before-dev`.
- [ ] Confirm default CI path is credential-free and the controlled fixture
      repository contains no personal paths/secrets.

## Ordered Checklist

### 1. Add Codex Runtime Support

- [ ] Implement Codex command profile using stdin, `exec --json`, `--ephemeral`
      and least-capable explicit sandbox.
- [ ] Add version probe and controlled `--ignore-user-config` option without
      silently dropping repository instructions.
- [ ] Implement stateful JSONL decoder for thread/turn/item/error events.
- [ ] Add redacted versioned Codex fixture and per-event tests.
- [ ] Add Claude/Codex normalized conformance suite.

### 2. Build the Representative CaseBundle

- [ ] Create disposable deterministic Rust fixture repository and redacted
      runtime configuration manifest.
- [ ] Record/source the explicit Claude-side GNU constraint event.
- [ ] Execute/replay with-state Codex run and capture normalized tool/outcome
      evidence.
- [ ] Execute/replay without-state run with only the target StateId removed.
- [ ] Validate all invariant inputs, compare normalized tool arguments/outcomes,
      and persist the bounded claim summary.
- [ ] Add post-revoke projection/run proving the old state is absent while the
      historical receipt remains readable.

### 3. Add Minimal Product Read Models and Actions

- [ ] Add host read models for state evidence, checkpoint/projection status,
      selected refs and pending permission requests.
- [ ] Extend theater reducers/renderers for remembered notice, runtime/handoff
      status, state inspector, projection inspector and permission modal.
- [ ] Return typed revoke and permission actions to host; keep SQLite/process
      access out of theater.
- [ ] Add Director, reducer, buffer/CJK and action-routing tests.

### 4. Establish Reproducible Build and CI

- [ ] Resolve dependency MSRV and reconcile it with workspace `rust-version`.
- [ ] Track generated `Cargo.lock` and pin the proven Rust toolchain.
- [ ] Add Linux and Windows GNU CI jobs for fmt/check/clippy/test.
- [ ] Run fixture secret/path scanner and CaseBundle validation in default CI.
- [ ] Keep live dual-runtime smoke manual/opt-in and record both CLI versions.
- [ ] Separate setup/network/toolchain failures from code/test failures in
      reports.

### 5. Final Vertical Acceptance

- [ ] Run full fixture-driven source event -> state -> checkpoint -> receipt ->
      Codex tool -> outcome trace.
- [ ] Run with/without comparison and inspect invariant manifest.
- [ ] Run revoke flow through UI action and subsequent projection.
- [ ] Verify permission approval never creates an auto-approve state.
- [ ] Run full workspace quality gate on both CI targets.
- [ ] If both CLIs/auth are locally available, run opt-in dual-runtime smoke;
      enabled missing prerequisites must fail clearly.
- [ ] Run `trellis-check`, update specs, commit/archive child, then return to
      parent acceptance review.

## Validation Commands

```bash
git diff --check
cargo fmt --all -- --check
cargo test -p tsukumo-adapters codex
cargo test -p tsukumo-host case_bundle
cargo test -p tsukumo-theater
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 ./.trellis/scripts/task.py validate 07-10-c1-cross-runtime-ui
```

Optional real evidence gate:

```bash
TSUKUMO_RUN_LIVE_SMOKE=1 cargo test -p tsukumo-host --test cross_runtime_live -- --ignored --nocapture
```

## Risky Files and Rollback

- Codex item schemas are vendor-versioned inputs. Unknown events are observable
  skips; malformed known events fail. Never relax a fixture to hide drift.
- CaseBundle normalization must keep with/without inputs equal except for the
  target state; any other difference invalidates the comparison.
- Theater additions remain presentation-only. Revert UI independently without
  deleting state, Chronicle, checkpoint or receipt rows.
- Toolchain/lockfile changes are reviewed as reproducibility changes, not mixed
  with regenerated runtime fixtures without explanation.
