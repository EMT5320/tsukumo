# V0 MVP TUI

## Parent and Dependencies

- Parent: `.trellis/tasks/07-10-c1-handoff-continuity`
- Depends on: contracts/Chronicle, handoff/projection, host/runtime, and the
  cross-runtime evidence contracts.

## Goal

Turn the existing print-mode theater into the minimum interactive Tsukumo
product surface for understanding handoff state, reviewing durable state,
revoking it, and deciding permissions.

## User Value

The owner can run Tsukumo in one terminal, see what is happening, understand
what was remembered and projected, correct durable state, and approve or deny
blocked actions without inspecting SQLite or JSONL files.

## Confirmed Evidence

- `tsukumo-theater` already has a pure Director, `StageWorld`, buffer renderer,
  fixture replay, and CJK-aware string conversion.
- The current entry point is a fixed 72x22 print demo with no crossterm event
  loop, navigation, host read models, or typed actions.
- Theater must remain presentation-only; host owns storage, runtime control,
  state transitions, and permission decisions.

## Requirements

- Add a real terminal lifecycle with alternate screen/raw mode, resize handling,
  bounded tick rate, clean quit, and guaranteed restoration on errors.
- Add host-owned read models for runtime/execution/handoff status, state
  evidence, checkpoint/projection metadata, selected StateRefs, omissions, and
  pending permission requests.
- Add typed UI actions for navigation, state revoke, allow once, allow session,
  deny, refresh, and quit.
- Present four minimum surfaces: workshop, state inspector, projection
  inspector, and blocking permission modal.
- Keep remembered notices non-blocking and permission decisions explicit.
- Preserve CJK width and border alignment at supported terminal sizes; show a
  compact fallback instead of clipping at small sizes.
- Keep persona and visual copy in theater; runtime prompts remain factual.

## Acceptance Criteria

- [ ] A real TTY session opens, redraws, responds to keys, resizes, and restores
      the terminal after normal quit and injected failure.
- [ ] The workshop shows current runtime, execution, handoff status, attention,
      actor, and bounded log output.
- [ ] The state inspector shows value, scope, strength, status, and source event
      refs; revoke emits a typed host action and refreshes the view.
- [ ] The projection inspector shows checkpoint/projection IDs, selected refs,
      versions, budget use, and omissions without rendered prompt text.
- [ ] The permission modal shows redacted tool arguments, cwd, risk, and runtime
      and supports allow-once, allow-session, and deny with no implicit default.
- [ ] Buffer, action-routing, resize, terminal-restoration, and CJK tests pass.

## Out of Scope

- New characters, asset pipeline, complex animation, multiple world themes,
  mouse-first interaction, or general database/process access from theater.
