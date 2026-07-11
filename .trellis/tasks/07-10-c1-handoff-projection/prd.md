# C1 Handoff and Projection

## Parent and Dependency

- Parent: `.trellis/tasks/07-10-c1-handoff-continuity`
- Depends on: archived `07-10-c1-contracts-chronicle`
- V0 scope decision: `docs/tsukumo-v0-scope-convergence-2026-07-11.md`

## Goal

Compile Chronicle and Canonical State into a versioned, actionable
HandoffCheckpoint and commit an explainable ProjectionReceipt for every runtime
projection before any process can launch.

## User Value

When the owner switches tools, the next runtime receives enough task state to
continue work. The owner can later explain which durable state was selected
without accumulating full prompts as a privacy liability.

## Confirmed Evidence

- `BriefCompiler` currently provides a character-capped fact list with no
  checkpoint/open-loop semantics or stable StateRefs.
- The legacy inject trace records only brief/goal lengths and cannot attribute a
  projection to checkpoint, state, runtime, or execution.
- Contracts/Chronicle now provides stable IDs, StateRecord lifecycle,
  historical selection, SQLite authority, and sensitive-value boundaries.
- V0 production persistence is receipt-only. General debug/eval prompt snapshot
  storage, seven-day expiry, retain, and cleanup audit are deferred to V0.1.

## Requirements

- Implement checkpoint goal, progress, decisions, constraint refs, artifacts,
  open loops, next actions, state refs, source refs, and version identity.
- Reject a new checkpoint when a prior open loop silently disappears; every
  prior loop must be inherited, completed, abandoned, or replaced.
- Implement deterministic state selection from checkpoint pins, scope,
  lifecycle/expiry, strength, freshness, stable tie-breakers, and a declared
  character budget.
- Implement a canonical versioned renderer with stable section order, LF
  normalization, one final newline, exact SHA-256 overall/section digests,
  byte/character lengths, explicit budget unit, omissions, and redactions.
- Persist checkpoint, receipt, and StateRef/source-event edges in the same
  SQLite authority with immutable historical rows.
- Keep raw rendered prompt bytes inside `SensitiveText`; receipt, Chronicle,
  logs, errors, and debug representations cannot contain prompt text/secrets.
- Expose a `PreparedProjection` only after receipt commit; persistence failure
  cannot produce a launchable value.
- Provide a deterministic with-state/without-state comparison seam that removes
  one target StateId and reports invariant input/digest differences without
  persisting prompt snapshots.
- Clearly deprecate compatibility briefing/prompt assembly for production
  launch paths and direct new hosts to receipt-committed projections.

## Acceptance Criteria

- [x] A new checkpoint preserves or explicitly resolves every prior open loop.
- [x] Hard constraints use stable StateRefs; display text changes do not change
      identity.
- [x] Missing/unresolved pinned refs fail closed; revoked, expired, and
      scope-inapplicable state is omitted with deterministic reasons.
- [x] Receipt identifies checkpoint, selected refs, runtime, execution,
      versions, hashes, lengths, redactions, omissions, and budget unit.
- [x] Identical renderer inputs/version produce identical bytes and hashes;
      relevant mutations change the expected section and overall hashes.
- [x] Receipt schema/rows, Chronicle, logs, errors, and Debug output exclude a
      sentinel prompt secret and contain no rendered-text field.
- [x] Receipt/selected-ref transaction failure yields no `PreparedProjection`.
- [x] Historical receipts remain unchanged after state supersede/revoke, while
      later selection excludes inactive state.
- [x] The deterministic comparison removes one target StateId and keeps every
      other controlled input equal.

## Out of Scope

- Real subprocesses, permission UI, second runtime, full MCP recall, raw prompt
  archives, encrypted transcript storage, persistent debug/eval snapshots,
  expiry/retain scheduling, cleanup audit, and general CaseBundle storage.
