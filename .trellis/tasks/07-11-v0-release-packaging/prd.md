# V0 Release Packaging

## Parent and Dependencies

- Parent: `.trellis/tasks/07-10-c1-handoff-continuity`
- Depends on all functional C1/V0 children.
- Activation gate: the 2026-07-23 trusted-handoff decision must select GO, or
  the owner must explicitly approve a packaging scope for a PIVOT result.

## Goal

Package the validated Tsukumo vertical slice as an installable, documented, and
reproducible `v0.1.0` release candidate suitable for real owner testing.

This work starts after the trusted-handoff decision. Evidence-freeze demo
capture does not activate the full packaging task.

## User Value

A new checkout can build and run Tsukumo from documented commands, understand
its data/privacy boundaries, and distinguish supported fixture/live paths and
known V0 limitations.

## Requirements

- Expose one installable `tsukumo` binary with useful `--help`, version, config,
  data-directory, fixture, and live-runtime entry points.
- Add README quickstart, product status, screenshots, architecture summary,
  runtime prerequisites, data/privacy model, troubleshooting, and limitations.
- Add the declared MIT license file and complete Cargo package metadata.
- Track `Cargo.lock`, determine the real dependency MSRV, and pin the proven
  toolchain/targets without overstating compatibility.
- Add credential-free Linux and Windows GNU CI for fmt/check/clippy/tests plus
  fixture secret/path validation.
- Keep authenticated Claude/Codex smoke explicit, local/manual, and actionable
  when prerequisites are absent.
- Produce a release checklist and tag only after a clean-checkout verification.

## Acceptance Criteria

- [ ] `cargo install --path <binary-crate>` produces `tsukumo`, and documented
      fixture mode starts without external credentials.
- [ ] README commands pass from a clean checkout on the pinned toolchain.
- [ ] `LICENSE`, tracked `Cargo.lock`, toolchain declaration, package metadata,
      and Linux/Windows GNU CI exist and agree.
- [ ] CI runs the full credential-free gate and rejects secrets/personal paths
      in fixtures or evidence.
- [ ] Live smoke remains opt-in and reports missing CLI/auth prerequisites as
      actionable failures.
- [ ] Release notes state V0 capabilities, claim boundaries, known limitations,
      data location, and rollback/removal instructions.
- [ ] The final release candidate has a clean worktree and a reviewed `v0.1.0`
      tag plan.

## Out of Scope

- Installer GUI, auto-update, package-manager publication beyond Cargo/source
  instructions, signed binaries, or broad platform claims without CI evidence.
