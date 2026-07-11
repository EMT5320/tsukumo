# V0 MVP TUI — Implementation Plan

## Preconditions

- [ ] Earlier C1 children are archived and committed.
- [ ] Host exposes typed read models and actions with no theater dependency
      reversal.
- [ ] Load Rust architecture, theater, error, and quality specs before editing.

## Ordered Checklist

### 1. Product Read Models and Actions

- [ ] Add host-owned runtime, handoff, state, projection, and permission views.
- [ ] Add typed refresh/revoke/permission/quit actions and a fake controller.
- [ ] Prove prompt text and raw secrets cannot enter UI read models.

### 2. Interactive App State

- [ ] Add screen/navigation/modal state and exhaustive key mapping.
- [ ] Route revoke and permission decisions through typed actions.
- [ ] Add reducer/action tests before renderer changes.

### 3. Render the Minimum Surfaces

- [ ] Extend workshop with runtime/handoff status and remembered notice.
- [ ] Add state and projection inspectors.
- [ ] Add blocking permission modal and compact-size fallback.
- [ ] Add normal/compact/CJK buffer tests.

### 4. Real Terminal Lifecycle

- [ ] Add crossterm raw/alternate-screen guard and terminal restoration.
- [ ] Add input/tick/resize/render loop with bounded redraw behavior.
- [ ] Add injected-error restoration test and manual Windows Terminal smoke.

### 5. Quality and Handoff

- [ ] Run theater/host tests and full workspace gates.
- [ ] Capture fresh visual evidence for each screen and modal.
- [ ] Run `trellis-check`, update specs, commit, and archive.

## Validation Commands

```bash
git diff --check
cargo fmt --all -- --check
cargo test -p tsukumo-theater
cargo test -p tsukumo-host
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 ./.trellis/scripts/task.py validate 07-11-v0-mvp-tui
```

## Risk and Rollback

- Terminal cleanup failures block V0 usability.
- Permission keys require explicit labels and no implicit approval path.
- Theater dependency direction is a hard gate; host owns all side effects.
