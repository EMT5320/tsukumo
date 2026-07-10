# Rust Workspace Guidelines

These specifications describe Tsukumo's current Rust workspace and the design
contracts already frozen for the C1 handoff slice. They replace the generic
backend/frontend templates created by `trellis init`.

## Read by Change Type

| Change | Required guidance |
|---|---|
| Any cross-crate or dependency change | [Architecture and Boundaries](./architecture-and-boundaries.md) |
| Kernel event, adapter payload, fixture, or identity change | [Types and Event Contracts](./types-and-event-contracts.md) |
| Soul, Chronicle, state, checkpoint, receipt, or database change | [State and Persistence](./state-and-persistence.md) |
| Director, StageWorld, rendering, animation, or attention change | [Theater](./theater.md) |
| Fallible I/O, parsing, process, or storage change | [Error Handling](./error-handling.md) |
| Every implementation and review | [Quality and Testing](./quality-and-testing.md) |

## Non-Negotiable Invariants

1. Runtime/vendor payloads are normalized before they leave adapters.
2. Theater consumes normalized events and never owns runtime or soul state.
3. A persistent spirit identity is distinct from its current runtime binding.
4. Relationship state comes from referenced evidence; permissions never become
   durable state through inference.
5. Live execution, persisted Chronicle events, replay, and tests use the same
   typed event contract.
6. Presentation persona stays in theater; runtime prompts contain facts,
   constraints, procedures, and handoff state only.

Primary design sources:

- `DESIGN.md`
- `docs/tsukumo-vision-state-handoff-convergence-2026-07-10.md`
- `crates/tsukumo-kernel/src/event.rs`
- `crates/tsukumo-theater/src/director.rs`

