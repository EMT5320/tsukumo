# V0 Release Packaging — Implementation Plan

## Preconditions

- [ ] All functional children are archived and committed.
- [ ] The product binary and TUI pass their local quality and visual gates.

## Ordered Checklist

### 1. Installable Product Entry

- [ ] Finalize binary name, help, version, config, data-dir, fixture, and live
      commands.
- [ ] Verify `cargo install --path` and an isolated first run.
- [ ] Keep credentials and prompts out of diagnostics.

### 2. Repository and Package Metadata

- [ ] Add MIT `LICENSE` and Cargo repository/readme metadata.
- [ ] Track `Cargo.lock` and reconcile/pin real MSRV/toolchain/targets.
- [ ] Add release profile or packaging metadata only when measured/required.

### 3. CI and Clean-Environment Verification

- [ ] Add Linux and Windows GNU fmt/check/clippy/test jobs.
- [ ] Add fixture/evidence secret and personal-path validation.
- [ ] Exercise the documented fixture quickstart in CI or a deterministic smoke.
- [ ] Keep live smoke manual and opt-in.

### 4. README and Release Materials

- [ ] Write README along the user journey and include current screenshots.
- [ ] Document data/privacy, revoke, limitations, troubleshooting, and removal.
- [ ] Prepare release notes and a `v0.1.0` checklist.

### 5. Final Receipt

- [ ] Verify from a clean checkout on the pinned toolchain.
- [ ] Run full gates, TUI visual QA, install/run smoke, and secret scan.
- [ ] Run `trellis-check`, update specs, commit/archive, then review the tag.

## Validation Commands

```bash
git diff --check
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo install --path crates/tsukumo-host --locked --root <temp-root>
python3 ./.trellis/scripts/task.py validate 07-11-v0-release-packaging
```

## Risk and Rollback

- Do not claim Rust 1.75 support until it passes with locked dependencies.
- README and screenshots must describe the same release-candidate revision.
- Do not tag a candidate with environmental or code gate failures.
