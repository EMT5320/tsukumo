# Bootstrap Tsukumo Rust Guidelines

## Goal

Replace the generic full-stack Trellis templates with project-specific coding
guidance derived from Tsukumo's Rust workspace, tests, and frozen design
contracts.

## Background

`trellis init` classified the empty repository as full stack and created
`backend/` and `frontend/` placeholders. The repository now contains four Rust
crates (`kernel`, `adapters`, `soul`, and `theater`) plus a planned host. The
template split no longer represents actual ownership or language conventions.

## Scope

- Reshape `.trellis/spec/` around the Rust workspace.
- Document crate boundaries and dependency direction.
- Document normalized event, identity, replay, state/evidence, theater, error,
  and quality contracts.
- Cite real source files, tests, and project design documents.
- Preserve and reuse the shared thinking guides.

## Out of Scope

- Product source changes.
- Implementing C1 contracts or fixing probe technical debt.
- Inventing frontend/backend layers that do not exist.
- Claiming a Rust quality gate passed when the local toolchain/network cannot
  execute it.

## Files

- `.trellis/spec/rust/index.md`
- `.trellis/spec/rust/architecture-and-boundaries.md`
- `.trellis/spec/rust/types-and-event-contracts.md`
- `.trellis/spec/rust/state-and-persistence.md`
- `.trellis/spec/rust/theater.md`
- `.trellis/spec/rust/error-handling.md`
- `.trellis/spec/rust/quality-and-testing.md`
- `.trellis/spec/guides/*` (preserved shared guidance)

## Acceptance Criteria

- [x] Generic backend/frontend placeholder specs are removed.
- [x] Rust specs contain concrete repository paths and local examples.
- [x] Crate boundaries and cross-layer data flow are documented.
- [x] Frozen C1 identity, evidence, safety, and projection invariants are
      discoverable before implementation.
- [x] Index files match the final spec tree.
- [x] No template placeholder text remains in `.trellis/spec/`.
- [x] Spec links and Trellis package discovery are verified.
- [ ] Changes are committed and this bootstrap task is archived.
