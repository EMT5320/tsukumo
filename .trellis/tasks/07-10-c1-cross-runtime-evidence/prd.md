# C1 Cross-Runtime Evidence

## Parent and Dependencies

- Parent: `.trellis/tasks/07-10-c1-handoff-continuity`
- Depends on: contracts/Chronicle, handoff/projection, and host/runtime.

## Goal

Add Codex as the second runtime and complete a controlled Claude-to-Codex
continuity proof. Evaluate trusted handoff against a Trellis-only strong
baseline, separating automatic migration value from provenance/revoke value.

## User Value

The owner can switch between the two primary coding-agent runtimes with less
time spent restoring context, correcting stale state, and recovering from a bad
handoff. If those gains do not appear, the owner gets an evidence-backed pivot
or stop decision rather than an unbounded infrastructure project.

## Confirmed Evidence

- Claude CLI and Codex CLI are the approved V0 runtime pair.
- Codex `exec --json` emits JSONL suitable for the existing normalized adapter
  boundary.
- Host and production receipts own real execution evidence; the comparison
  bundle is a deterministic test/evidence artifact, not a second durable
  product store.
- Trellis is the V0 strong baseline: repository specs, task artifacts, context
  injection, channel runtime, Git, and ordinary owner-directed model switching.
- The current removed-state comparison proves at most behavioral sensitivity;
  it does not prove task utility or the necessity of durable traceability.
- Loomstead's closed human-rating gate is a negative prior: evidence/integrity
  differences without behavioral or user-outcome differences cannot support a
  product claim.
- Interactive UI is already implemented in a separate task; release CI remains
  downstream. Neither is part of this evidence child.

## Requirements

- Add a Codex `exec --json` runtime profile and stateful line decoder using the
  same host ports and normalized event contract as Claude.
- Preserve vendor item/thread identifiers as namespaced provenance while host
  assigns durable event identities.
- Add redacted, versioned Codex fixtures and a Claude/Codex conformance suite.
- Run a deterministic GNU-toolchain scenario where the with-state and
  without-state paths differ only by the target StateId and dependent digests.
- Record normalized selected-ref, tool-argument, outcome, and invariant
  comparison data without persisting raw prompts or credentials.
- Prove a revoke transition removes the old state from the next projection
  while the historical receipt remains readable.
- Pre-register and run three conditions: C0 Trellis-only, C1 automatic state
  migration with provenance controls hidden, and C2 migration plus source,
  receipt, causal-chain, and selective-revoke controls.
- Record natural handoff episodes separately from controlled stale, wrong-scope,
  and contradictory-state faults.
- Measure first-correct-action time, owner interventions, stale-state errors,
  context-reading tokens, task quality, bad-state diagnosis/recovery time,
  collateral revokes, recurrence, and always-on overhead.
- Freeze evidence on 2026-07-22 and make a GO/PIVOT/NO-GO decision on
  2026-07-23. Treat thresholds as an n=1 product gate, not a statistical claim.
- Keep authenticated dual-runtime smoke explicit and opt-in; missing enabled
  prerequisites produce actionable failures.

## Acceptance Criteria

- [ ] One SpiritId and checkpoint continue across Claude and Codex
      RuntimeBindings.
- [ ] Claude and Codex fixtures normalize equivalent tool/outcome semantics.
- [ ] With-state and without-state inputs match except target state selection
      and dependent hashes, and normalized tool arguments visibly differ.
- [ ] Receipt/tool/outcome refs trace to the source user EventId.
- [ ] Post-revoke projection excludes the old StateRef and preserves the old
      receipt for explanation.
- [ ] C1 vs C0 is measured on predeclared continuation metrics, and either
      first-correct-action time improves by about 30% or owner interventions by
      about 50%, without lower task quality, before claiming migration value.
- [ ] C2 vs C1 is measured on at least one predeclared stale/scope/conflict
      recovery case, and diagnosis/recovery improves by about 50% or collateral
      deletion/recurrence visibly falls before claiming traceability value.
- [ ] Normal-operation latency, token, storage, and cognitive overhead are
      recorded; injected faults are never used as evidence of natural incidence.
- [ ] Designed trace cases preserve a complete source -> state -> checkpoint ->
      receipt -> execution -> outcome/revoke chain as an engineering gate.
- [ ] Default tests require no CLI credentials and contain no secrets or
      personal paths.
- [ ] Opt-in live smoke records both CLI versions and fails clearly when an
      explicitly enabled prerequisite is missing.

## Out of Scope

- Interactive TUI, release packaging/CI, persistent debug prompt snapshots,
  seven-day expiry/retain lifecycle, general evaluation artifact storage, and
  broad causal/population claims. Permission-approval productization and
  relationship/companion expansion are P2 for this validation window.
