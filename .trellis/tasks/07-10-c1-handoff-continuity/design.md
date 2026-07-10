# C1 Handoff Continuity — Technical Design

## Status and Inputs

This is the parent integration design for C1. Product intent comes from
`DESIGN.md`; the frozen domain design comes from
`docs/tsukumo-vision-state-handoff-convergence-2026-07-10.md`; executable Rust
contracts come from `.trellis/spec/rust/`.

The parent task owns no feature implementation. Its children deliver the
vertical slice in dependency order and the parent performs contract and
evidence-chain review at each boundary.

## Architecture

```text
user/UI event
  -> tsukumo-host assigns envelope identity
  -> SQLite Chronicle transaction
     -> StateExtractor -> StateDraft -> deterministic StateWriter
     -> versioned Canonical State

runtime switch / handoff trigger
  -> HandoffCompiler -> immutable Checkpoint
  -> StateSelector -> ProjectionRenderer
  -> production ProjectionReceipt commit
  -> prompt bytes through child stdin
  -> Claude/Codex process stdout JSONL
  -> vendor adapter -> KernelEventPayload
  -> host envelope writer -> Chronicle
  -> pure Director -> StageWorld -> ratatui
```

The same persisted envelope is used by live events, committed fixture replay,
and Chronicle replay. Vendor JSON never crosses the adapter boundary. Theater
is a lossy view and never controls execution or writes canonical state.

## Crate Ownership

| Area | Owner | Notes |
|---|---|---|
| IDs, event envelope/payload, runtime-neutral shared value types | `tsukumo-kernel` | No vendor schema, SQL, or UI |
| Claude/Codex command profiles and JSONL decoders | `tsukumo-adapters` | No soul storage or theater dependency |
| Chronicle, state, checkpoint, projection, receipt, CaseBundle persistence | `tsukumo-soul` | One SQLite authority; exports derived |
| Director, view model reduction, TUI rendering | `tsukumo-theater` | No process control or canonical writes |
| Process lifecycle, envelope assignment, transactions across ports, Safety Plane, UI actions | new `tsukumo-host` | Composition root only |

No child may create runtime-specific state tables or a second Spirit identity.

## Core Contracts

### Identity and Events

`KernelEvent` becomes a versioned envelope around `KernelEventPayload`. Stable
IDs and correlation connect user input, state lifecycle, checkpoint,
projection, tool calls, permission decisions, and outcome. Events outside a
runtime execution may omit `execution_id`/runtime binding; code must never fill
those fields with fabricated placeholders.

### Three Ledgers, One Authority

One `soul.db` is authoritative for:

1. append-only Chronicle events;
2. immutable/versioned StateRecords and evidence edges;
3. immutable checkpoints, receipts, and selected-state edges.

JSONL, FTS, `MEMORY.md`, and `USER.md` are projections rebuilt from committed
SQLite rows. A logical event/state write shares one transaction. A receipt is
committed before its runtime process starts.

### State Formation

Rules handle explicit structured signals and a provider-neutral structured LLM
extractor handles free-form text. Both only produce `StateDraft`. The
deterministic `StateWriter` owns evidence, scope, secret, lifecycle, and
strength checks. Repeated permission approval never becomes an auto-approve
relationship state.

### Handoff and Projection

Checkpoints are versioned task state, not recalled fact dumps. Hard constraints
are carried by `StateRef`; open loops must be inherited or explicitly resolved.
State selection is deterministic for a fixed database snapshot and budget.

Production receipts contain IDs, selected refs, versions, SHA-256/section
digests, lengths, explicit budget units, omission reasons, and redaction
metadata. They contain no raw prompt. An explicit debug/eval CaseBundle may
store a separately hashed, redacted snapshot with seven-day default expiry or
an explicit retain decision.

### Runtime and Safety

The host owns each process and writes the prompt through stdin so secrets do not
appear in argv/env. Stdout is decoded line by line and persisted before theater
consumes it. Cancellation, timeout, malformed stream, non-zero exit, permission
denial, and success remain distinct outcomes.

Safety is deterministic and separate from relationship state. C1 implements
the host-facing permission request/decision seam and never uses vendor
"skip permissions" flags. A vendor permission bridge that is not truly wired
is reported as degraded/unsupported rather than presented as live fidelity.

## C1 Evidence Chain

The representative case is fixed:

1. A user event in Claude context explicitly states that Tsukumo uses the GNU
   Rust toolchain on Windows.
2. The event produces an explicit, workspace/OS-scoped constraint.
3. A versioned checkpoint carries its `StateRef` across a runtime switch.
4. A Codex projection receipt proves that the ref was selected and projected.
5. Codex emits a tool call using the GNU-qualified command; the tool event and
   outcome link back through execution/projection/state/event IDs.
6. A controlled without-state CaseBundle removes only that state and compares
   resulting tool arguments.

Runtime evidence supports “selected, projected, then observed.” Only the
controlled pair supports a narrower causal comparison claim.

## Compatibility and Migration

- The current `KernelEvent` wire shape is an internal probe contract. C1 makes a
  coordinated breaking migration across fixtures, adapters, session replay,
  Director tests, and examples rather than maintaining two live event models.
- SQLite migrations are additive and idempotent. Existing `facts`/FTS data is
  retained during migration; an importer may emit explicit `legacy_imported`
  Chronicle evidence and low-strength state, but never fabricates Explicit or
  Constraint status.
- Existing Markdown files remain rebuildable views. Manual edits are not
  imported implicitly.
- Receipt schema is append-only/versioned; future digest or renderer versions
  coexist with historical rows.

## Operational Boundaries

- Default CI is credential-free and uses reviewed, redacted Claude/Codex
  fixtures.
- `TSUKUMO_RUN_LIVE_SMOKE=1` opts into both authenticated CLIs in a disposable
  repository, records CLI versions, and fails if prerequisites are missing.
- `Cargo.lock` is tracked because C1 adds an executable host. The final child
  pins a toolchain proven against dependencies and runs Linux plus Windows GNU
  gates.
- Prompt text, auth files, credentials, and raw vendor transcripts never enter
  receipts or committed CaseBundles.

## Delivery Gates

1. Contracts/Chronicle: envelope, persistence, state writer, migration and
   replay tests pass.
2. Handoff/Projection: deterministic checkpoint/selection/receipt and synthetic
   CaseBundle tests pass.
3. Host/Runtime: first event arrives before process exit, receipt precedes
   spawn, cancellation reaps, and Claude fixture/live seam passes.
4. Cross-runtime/UI: Codex conformance, removed-state case, minimal TUI and full
   reproducible quality matrix pass.

If a child changes an upstream serialized contract, work returns to the owning
child and all downstream fixtures are regenerated intentionally. Rollback uses
normal commits; no migration deletes legacy rows or historical receipts.
