# Codex 0.135.0 GNU Removed-State Evidence — 2026-07-13

## Scope

This is a paired-capture replay-difference artifact for the C1 engineering
ladder. It does not establish that the target state alone caused the observed
difference, task utility, successful command execution, or natural stale-state
incidence.

## Controlled Setup

- Codex CLI: `codex-cli 0.135.0`.
- Source evidence in the deterministic replay: one explicit GNU constraint
  event bound to `claude_cli/fixture`, `SpiritId=yuka`.
- Target runtime: `codex_cli/owned_process` through the production Host port.
- Disposable crate: tracked under
  `crates/tsukumo-host/tests/fixtures/cross_runtime_rust` and materialized into a
  temporary directory by the test.
- Repository fixture SHA-256: `5ba04cd5db4a7f8cf02537fe2ce940740dda78c5ab60507758d79f80f5a6b578`.
- The crate passed `cargo +stable-x86_64-pc-windows-gnu test --offline` before
  the model capture.
- Both model stimuli were recorded as using the same base task and safety
  configuration. The handoff-constraint section contained either the GNU
  StateRef text or an empty marker. Raw prompt snapshots were not retained.
- Exact model identity, model/user configuration digests, capture prompt
  digests, and per-run timestamps were not retained. They cannot be
  reconstructed from the redacted JSONL streams.

The live stimulus was a controlled capture aid. The committed Host test replays
each reviewed capture against its corresponding production-rendered
`PreparedProjection`. The capture manifest distinguishes these verifiable
replay bindings from unavailable original-capture controls.

## Observed Commands

| Condition | Reviewed command intent | Tool status | Turn status |
|---|---|---|---|
| with state | `cargo +stable-x86_64-pc-windows-gnu test --offline`; fallback `rustup run stable-x86_64-pc-windows-gnu cargo test --offline` | both `declined`, exit `-1` | completed |
| without state | `cargo test --offline` | `declined`, exit `-1` | completed |

The fail-closed policy rejected every Cargo command. The vendor
`turn.completed` only means the model turn ended normally. Current
normalization emits `Outcome(Failed)` for both streams because their tool errors
remain sticky through terminal reconciliation. The task command did not
execute inside the Codex run. The with-state model attempted a second
GNU-equivalent command after the first rejection, so the two reviewed streams
also differ in tool count.

## Reviewed Fixtures

- `crates/tsukumo-adapters/fixtures/codex_0_135_0_gnu_with_state.jsonl`
  - 8 JSONL lines
  - SHA-256 `c54f9ec7fb395134b907ed874764e89f82a447034dfa8fe54fadc9af859ad567`
- `crates/tsukumo-adapters/fixtures/codex_0_135_0_gnu_without_state.jsonl`
  - 6 JSONL lines
  - SHA-256 `dd05664296fd5c58b7e381f8e216cb6de8edf05d91f2cdb07d2df363c93e55d2`
- `crates/tsukumo-adapters/fixtures/codex_0_135_0_gnu_capture_manifest.json`
  - binds both fixture SHA-256 values, repository fixture SHA-256, replay
    projection SHA-256 values, runtime profile, sandbox, and approval policy;
  - keeps unavailable capture controls as `null`;
  - records `causal_claim_eligible=false`.

Thread IDs were replaced with deterministic fixture labels. No user-home path,
auth material, raw prompt, or temporary repository path remains. Stderr stayed
diagnostic-only and was not promoted into fixtures.

## Replay Contract

`cross_runtime_comparison_contract.rs` proves:

1. the Claude-bound source event creates the explicit GNU StateRecord;
2. one checkpoint prepares with-state and target-StateId-excluded receipts;
3. `compare_projection_receipts` accepts only the Constraints section and
   dependent digest change;
4. the Host sends each production-rendered prompt through stdin;
5. both reviewed Codex streams cross the production decoder, envelope,
   Chronicle, and outcome path;
6. normalized tool arguments visibly differ and both policy declines remain
   explicit tool errors;
7. the capture manifest matches fixture, repository, and replay projection
   digests and keeps missing original-capture controls explicit;
8. the bounded replay manifest contains selected refs, commands, error flags,
   failed outcomes, and invariant metadata without raw prompt text.

## Claim

The two reviewed capture streams contain different command intents and replay
through the expected with-state/without-state production projections. Because
the original model/config/prompt digests are unavailable, this artifact does
not attribute that difference solely to the target state. C1 vs C0
continuation utility and C2 recovery utility remain open for the
pre-registered observation protocol.
