# C1 Cross-Runtime and UI — Technical Design

## Scope

This child adds the Codex runtime profile/decoder, runs the representative
Claude-to-Codex handoff and removed-state comparison, exposes the minimum TUI
needed to understand/control the flow, and closes the reproducible quality
gate.

## Codex Runtime Profile

Reference command shape:

```text
stdin -> codex exec --json --ephemeral --sandbox <least-capable-profile> -
```

For controlled comparison, `--ignore-user-config` may remove personal
`config.toml` drift. Repository instructions remain in force unless the
CaseBundle explicitly defines a synthetic repository without them. The profile
records `codex --version`; it does not read or persist `auth.json`.

The line decoder maps documented JSONL types to shared payload semantics:

- `thread.started` -> runtime provenance/session metadata;
- `turn.started` / `turn.completed` / `turn.failed` -> lifecycle/outcome;
- `item.started` / `item.updated` / `item.completed` command executions ->
  correlated tool start/progress/end;
- file changes, MCP calls, web searches, plans, reasoning, and messages -> the
  smallest honest normalized event or an observable documented skip;
- `error` or malformed known items -> classified adapter/runtime failure.

Vendor item IDs are namespaced provenance and correlation hints, not global
event IDs. A shared conformance suite proves that Claude and Codex fixtures
produce equivalent normalized tool/outcome semantics for the C1 case.

## Representative CaseBundle

```rust
pub struct CaseBundleManifest {
    pub schema_version: u16,
    pub case_id: CaseId,
    pub source_runtime: RuntimeVersion,
    pub target_runtime: RuntimeVersion,
    pub repository_fixture: ArtifactDigest,
    pub model_and_config: RedactedRuntimeConfig,
    pub target_state: StateId,
    pub with_state: CaseRunRef,
    pub without_state: CaseRunRef,
    pub invariants: Vec<InvariantCheck>,
    pub comparison: ComparisonSummary,
}
```

The deterministic fixture path is:

1. Host records the user's explicit Windows GNU constraint in a Claude-bound
   session.
2. StateWriter creates the scoped explicit constraint.
3. Handoff compiler produces a checkpoint for a Codex execution.
4. With-state projection selects the constraint; normalized Codex fixture/live
   output contains the GNU-qualified cargo command.
5. Without-state projection removes only that StateId. Repository fixture,
   goal, runtime/model/config, sandbox, renderer version, and all other selected
   state remain equal.
6. The comparison records prompt-digest/selected-ref differences, normalized
   tool argument differences, outcome, latency/token metadata when available,
   and invariant results.

The bundle contains normalized/redacted evidence and optional redacted prompt
snapshots under the receipt retention policy. Raw prompts, credentials, user
home paths, and uncontrolled repository contents are rejected by fixture
validation.

## Product Claim Boundary

- A normal run may say the GNU state was selected and projected, followed by a
  GNU tool call.
- The paired case may say removing that state changed the observed tool
  arguments under the recorded controlled conditions.
- C1 does not generalize the result to all models/tasks or claim universal
  causality.

## Minimal TUI

Reuse the ratatui theater surface. Add presentation state, renderers, and host
actions without granting theater write authority:

```text
main workshop
  - current runtime/execution and handoff status
  - non-blocking "remembered" notice
  - attention state / actor / log

state inspector
  - state value, scope, strength, status, source event refs
  - revoke action (sent to host)

projection inspector
  - checkpoint/projection IDs, selected StateRefs, versions, budget/omissions

permission modal
  - tool, redacted args, cwd, risk, runtime
  - allow once / allow session / deny
```

`StageWorld` may remain lossy for animation. Inspector data is a read model
loaded from repositories by host and passed to theater. UI actions are typed
messages returned to host; theater never calls SQLite or a runtime directly.
State creation notices are non-blocking. Permission decisions are blocking and
cannot be dismissed as implicit approval.

## Revoke Flow

```text
UI revoke action
  -> host writes user decision event
  -> StateWriter revoke transition
  -> new state lifecycle event
  -> UI refresh
  -> next checkpoint/projection excludes revoked state
  -> historical receipt remains inspectable
```

The final acceptance run includes a post-revoke Codex projection proving the
old ref is absent.

## Reproducible Toolchain and CI

- Track `Cargo.lock` for the executable workspace.
- Resolve the real dependency MSRV before pinning an exact toolchain; keep
  `workspace.package.rust-version` honest.
- Add credential-free Linux and Windows GNU jobs for format, workspace check,
  clippy with warnings denied, and workspace tests.
- Vendor fixture tests and CaseBundle validation run in default CI.
- Live smoke is a separate local/manual gate using
  `TSUKUMO_RUN_LIVE_SMOKE=1`; it records both CLI versions and fails explicitly
  when gated prerequisites are absent.
- Environmental setup failures and Rust/code test failures are reported
  separately.

## Test Strategy

- Codex per-line decoder tests for all supported event/item classes, unknown
  kinds, malformed known kinds, and truncated turn.
- Claude/Codex shared conformance fixture suite.
- Full fixture-driven source event -> state -> checkpoint -> receipt -> tool ->
  outcome evidence-chain integration.
- With-state/without-state invariant and normalized argument diff tests.
- Revoke then re-project integration while old receipt remains readable.
- Director/reducer/render tests for notices, inspectors, handoff, permission
  modal, CJK width, and typed UI actions.
- Fixture secret/path scanner and receipt no-prompt sentinel test.
- Full Linux/Windows GNU quality matrix plus opt-in real dual-runtime smoke.

## Rollback

Codex profile, fixtures, and UI panels are additive. Reverting the TUI does not
delete Chronicle/state data. A failed live comparison remains a failed artifact
with diagnostics and must not replace the deterministic fixture baseline.
