# Codex 0.135.0 GNU Removed-State Evidence ? 2026-07-13

## Scope

This is a controlled behavioral-sensitivity artifact for the C1 engineering
ladder. It does not establish task utility, successful command execution, or
natural stale-state incidence.

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
- Both model stimuli used the same base task and safety configuration. The
  handoff-constraint section contained either the GNU StateRef text or an empty
  marker. Raw prompt snapshots were not retained.

The live stimulus was a controlled capture aid. The committed Host test uses
production-rendered `PreparedProjection` values and binds each condition to its
reviewed capture.

## Observed Commands

| Condition | Reviewed command intent | Tool status | Turn status |
|---|---|---|---|
| with state | `cargo +stable-x86_64-pc-windows-gnu test --offline`; fallback `rustup run stable-x86_64-pc-windows-gnu cargo test --offline` | both `declined`, exit `-1` | completed |
| without state | `cargo test --offline` | `declined`, exit `-1` | completed |

The fail-closed policy rejected every Cargo command. The completed turn only
means the model turn ended normally. The task command did not execute inside
the Codex run. The with-state model attempted a second GNU-equivalent command
after the first rejection, so the two reviewed streams also differ in tool
count.

## Reviewed Fixtures

- `crates/tsukumo-adapters/fixtures/codex_0_135_0_gnu_with_state.jsonl`
  - 8 JSONL lines
  - SHA-256 `c54f9ec7fb395134b907ed874764e89f82a447034dfa8fe54fadc9af859ad567`
- `crates/tsukumo-adapters/fixtures/codex_0_135_0_gnu_without_state.jsonl`
  - 6 JSONL lines
  - SHA-256 `dd05664296fd5c58b7e381f8e216cb6de8edf05d91f2cdb07d2df363c93e55d2`

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
7. the bounded manifest contains selected refs, commands, error flags,
   outcomes, and invariant metadata without raw prompt text.

## Claim

The target state changed Codex command intent under a controlled replay pair.
This supports behavioral sensitivity for the recorded setup. It leaves C1 vs
C0 continuation utility and C2 recovery utility open for the pre-registered
observation protocol.
