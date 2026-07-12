# V0 MVP TUI — Implementation Plan

## Preconditions

- [x] Earlier C1 contracts required by the UI are committed and their public
      APIs are stable enough for read-model assembly.
- [x] Run `trellis-before-dev` and load the Rust architecture, event, theater,
      error, quality, cross-layer, and reuse guidance listed in `design.md`.
- [x] Keep the approved visual references and visual contract open during all
      renderer work.
- [x] Use the workspace GNU override and cached dependencies; record any genuine
      environment blocker separately from code failures.

## Ordered Checklist

### 1. Lock Pack and UI Contracts with RED Tests

- [x] Add failing tests for presentation-pack schema, source selection, limits,
      path confinement, cross-file references, and typed errors.
- [x] Add failing tests proving KernelEvent serialization and runtime prompts
      contain no Shiori/world/pack content.
- [x] Add failing tests for presentation actor versus source Spirit attribution.
- [x] Add failing reducer tests for navigation, actions, modal priority, compact
      behavior, and reduced motion.

**Done when:** the new contract tests fail for the intended missing behavior and
existing workspace tests still compile as far as the RED state allows.

### 2. Add the Pure Presentation-Pack Model

Files:

- `crates/tsukumo-theater/src/pack/mod.rs`
- `crates/tsukumo-theater/src/pack/model.rs`
- `crates/tsukumo-theater/src/pack/validation.rs`
- `crates/tsukumo-theater/src/lib.rs`
- `crates/tsukumo-theater/Cargo.toml`

Steps:

- [x] Define schema-v1 pack, companion, terminology, palette, scene, sprite,
      animation, and validated-pack types.
- [x] Implement pure validation with bounded dimensions, counts, copy, palette
      indices, IDs, and cross-file references.
- [x] Keep validation exhaustive and free of filesystem/process access.
- [x] Add serialization, base-case, and semantic-negative tests.

**Done when:** a fully assembled in-memory default and external fixture validate
through one pure API, and every documented invalid case has a typed test.

### 3. Add Host-Owned Sources and CLI Selection

Files:

- `crates/tsukumo-host/src/presentation_pack.rs`
- `crates/tsukumo-host/src/config.rs`
- `crates/tsukumo-host/src/host_error.rs`
- `crates/tsukumo-host/src/lib.rs`
- `crates/tsukumo-host/src/main.rs`
- bundled files under `crates/tsukumo-host/content/default-shiori/`
- external fixture under `crates/tsukumo-host/tests/fixtures/`

Steps:

- [x] Embed `default-shiori` while parsing it through the same schema and
      validator as external content.
- [x] Add `--presentation-pack <directory>` to the typed CLI parser.
- [x] Read files with total and per-file bounds; reject absolute, parent, and
      canonical path escapes.
- [x] Finish pack loading before raw/alternate-screen entry.
- [x] Return typed path/schema/validation errors with actionable context.
- [x] Add CLI/default/external/invalid/no-side-effect contract tests.

**Done when:** the default needs no filesystem installation, an external fixture
changes presentation data, and an invalid explicit path fails without fallback
or terminal mutation.

### 4. Separate Visible Actor from Source Executor

Files:

- `crates/tsukumo-theater/src/stage.rs`
- `crates/tsukumo-theater/src/director.rs`
- `crates/tsukumo-theater/src/world.rs`
- `crates/tsukumo-theater/src/drive.rs`
- affected theater, adapter, and cross-layer tests

Steps:

- [x] Add a bounded `PresentationActorId` and `StageAttribution`.
- [x] Extend `DirectorContext` with configured actor identity and line book.
- [x] Keep the Director pure while retaining `source_spirit_id`.
- [x] Key visible StageWorld actors by presentation actor ID.
- [x] Keep runtime/source attribution visible in snapshots and product views.
- [x] Update fixture replay and adapter-to-theater tests without changing
      KernelEvent wire semantics.

**Done when:** Shiori reacts to events from multiple Spirits, every frame/log can
still identify the factual executor, and durable event round trips are unchanged.

### 5. Add Product Views, App State, and Typed Actions

Files:

- `crates/tsukumo-theater/src/app/mod.rs`
- `crates/tsukumo-theater/src/app/model.rs`
- `crates/tsukumo-theater/src/app/reducer.rs`
- host view assembler/controller modules
- focused reducer and host-routing tests

Steps:

- [x] Define bounded redacted runtime, execution, handoff, state, projection,
      permission, and notice views.
- [x] Define `Screen`, ephemeral navigation state, `UiAction`, and exhaustive
      key mapping.
- [x] Give permission modal priority over normal navigation.
- [x] Route revoke and permission decisions through host-owned side effects.
- [x] Refresh views after completed host actions.
- [x] Prove raw secrets, rendered prompts, process handles, DB handles, and
      repositories cannot enter UI views.

**Done when:** pure reducer tests cover every screen and action, fake host tests
prove side-effect routing, and privacy sentinels stay absent.

### 6. Build Deterministic Scene and Sprite Rendering

Files:

- `crates/tsukumo-theater/src/render/mod.rs`
- `crates/tsukumo-theater/src/render/halfblock.rs`
- `crates/tsukumo-theater/src/render/layout.rs`
- `crates/tsukumo-theater/src/render/workshop.rs`
- `crates/tsukumo-theater/src/render/inspectors.rs`
- `crates/tsukumo-theater/src/render/permission.rs`
- default scene and Shiori sprite data
- focused buffer/snapshot tests

Steps:

- [x] Replace the placeholder point cluster with direct HalfBlock logical-pixel
      packing.
- [x] Hand-normalize the approved Shiori references into Idle, Work, Wait,
      Urgent, and Celebrate frames.
- [x] Hand-normalize quest board, portal, memory cabinet, projection desk,
      contract station, and walkable foreground.
- [x] Implement Full, Compact, and Fallback layout selection.
- [x] Implement truecolor, ANSI-256, and monochrome palette resolution.
- [x] Render workshop, bounded log, footer, inspectors, and permission modal.
- [x] Keep CJK width, truncation, and border calculations centralized.
- [x] Add full/compact/fallback/CJK/color/reduced-motion buffer tests.

**Done when:** the five poses and facilities remain identifiable at target sizes,
the factual hierarchy matches the approved concept, and no state relies on color
alone.

### 7. Add Real Terminal Lifecycle and Event Loop

Files:

- `Cargo.toml`
- `crates/tsukumo-host/Cargo.toml`
- `crates/tsukumo-host/src/tui/lifecycle.rs`
- host composition entrypoint and terminal lifecycle tests

Steps:

- [x] Add the crossterm dependency at the existing workspace boundary.
- [x] Implement the RAII guard for raw mode, alternate screen, cursor, and
      restoration.
- [x] Add input, resize, 10 Hz logic tick, capped 20 Hz redraw, and invalidation.
- [x] Drop intermediate animation frames when events outrun presentation.
- [x] Freeze reduced-motion mode on semantic key frames.
- [x] Restore the terminal on normal quit, injected error, and panic-hook path.
- [x] Keep pack/preflight failures outside terminal mode.

**Done when:** a real TTY can enter, resize, interact, quit, and recover from
failure with no terminal-state leak.

### 8. Cross-Layer Integration and Visual QA

- [x] Drive the default workshop from normalized fixture events through
      Director, StageWorld, ProductView, and renderer.
- [x] Exercise an external fixture pack through the same path.
- [x] Verify revoke and allow-once/allow-session/deny with fake host services.
- [x] Capture deterministic 17-mode visual evidence plus Windows ConPTY normal/resize receipts for Full, Compact, Fallback, inspectors,
      permission, monochrome, and reduced-motion cases.
- [x] Run `omo:visual-qa` in terminal/reference-fidelity mode and repair all
      Critical/Major findings.
- [x] Record any accepted minor design debt in the visual contract.

**Done when:** objective terminal artifacts support the visual and interaction
claims, and the approved reference hierarchy is preserved.

### 9. Full Quality Gate and Handoff

- [x] Run targeted pack, theater, host, adapter integration, and replay tests.
- [x] Run formatting, check, clippy, and full workspace tests.
- [x] Run `trellis-check`.
- [x] Update Rust specs with executable pack/attribution/rendering contracts.
- [x] Run task validation and inspect the final diff.
- [ ] Commit and archive through the Trellis finish flow after explicit user
      authorization.

## Validation Commands

Use the directory GNU override. Add `--offline` where the dependency cache
allows it.

```powershell
git -c safe.directory=D:/WorkSpace/tsukumo diff --check
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p tsukumo-theater
cargo test -p tsukumo-host
cargo test -p tsukumo-adapters --tests
cargo test --workspace
python ./.trellis/scripts/task.py validate 07-11-v0-mvp-tui
```

## Risk and Rollback

| Risk | Control |
|---|---|
| Pack path traversal or memory abuse | Root confinement, byte/count/dimension limits, typed failures |
| Shiori leaks into semantic core | Negative serialization/prompt tests and dependency review |
| False executor attribution | Actor/source split with fixture coverage |
| Generated art cannot survive terminal scale | Manual normalization and buffer/visual QA |
| Permission ambiguity | Blocking modal, explicit labels, no implicit default |
| Terminal corruption | Preflight before entry, RAII restoration, panic-hook test |
| Scope growth into game engine | No pathfinding, collision, combat, hot reload, editor, or marketplace |
| Dependency reversal | Theater-owned views/actions; host-owned I/O and side effects |

A renderer or pack rollback leaves Chronicle, State, Checkpoint, Receipt,
runtime, and safety data unchanged.