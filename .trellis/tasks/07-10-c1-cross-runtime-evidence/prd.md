# C1 Cross-Runtime Evidence

## Parent and Dependencies

- Parent: `.trellis/tasks/07-10-c1-handoff-continuity`
- Depends on: contracts/Chronicle, handoff/projection, and host/runtime.

## Goal

Add Codex as the second runtime and complete a controlled Claude-to-Codex
continuity proof with a deterministic removed-state comparison and a
post-revoke trace.

## User Value

The owner can switch between the two primary coding-agent runtimes while the
same spirit state and task checkpoint continue to influence work, with an
inspectable evidence chain and bounded claims.

## Confirmed Evidence

- Claude CLI and Codex CLI are the approved V0 runtime pair.
- Codex `exec --json` emits JSONL suitable for the existing normalized adapter
  boundary.
- Host and production receipts own real execution evidence; the comparison
  bundle is a deterministic test/evidence artifact, not a second durable
  product store.
- Interactive UI and release CI now have dedicated downstream tasks.

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
- [ ] Default tests require no CLI credentials and contain no secrets or
      personal paths.
- [ ] Opt-in live smoke records both CLI versions and fails clearly when an
      explicitly enabled prerequisite is missing.

## Out of Scope

- Interactive TUI, release packaging/CI, persistent debug prompt snapshots,
  seven-day expiry/retain lifecycle, general evaluation artifact storage, and
  broad causal claims.
