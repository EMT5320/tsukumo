# V0 MVP TUI

## Parent and Dependencies

- Parent: `.trellis/tasks/07-10-c1-handoff-continuity`
- Depends on the contracts/Chronicle, handoff/projection, host/runtime, and
  cross-runtime evidence boundaries defined by the parent task tree.

## Goal

Turn the existing print-mode theater into the minimum interactive Tsukumo
product surface: a stage-first terminal pixel game that explains current
runtime activity, durable state, projection inputs, handoff continuity, and
permission decisions.

## User Value

The guild master can stay in one terminal, understand what is happening, see
what Tsukumo remembered and projected, correct durable state, and approve or
deny blocked actions without opening SQLite or JSONL artifacts.

## Confirmed Evidence

- `tsukumo-theater` already has a pure Director, `StageWorld`, buffer
  renderer, fixture replay, and CJK-aware string conversion.
- At task intake, the entry point was a fixed 72x22 print demo with no crossterm event
  loop, navigation, host read models, typed actions, or asset-pack loading.
- `KernelEvent` is persona- and vendor-agnostic.
- `DirectorContext::line_book` already provides a presentation-copy seam.
- At task intake, `render.rs` hardcoded the workshop title, scene colors, facilities,
  and placeholder sprite geometry.
- Theater remains presentation-only and owns pure pack validation. Host owns storage, runtime control, state
  transitions, permission decisions, and bounded pack I/O.

## Product Decisions

### Stage-First Interface

- The default screen is a terminal-native pixel-game stage inspired by the
  ambient legibility of Pixel Agents.
- The workshop dominates the full-size composition. State and projection
  inspectors open on demand through visible keyboard actions.
- Pending permissions always place an explicit blocking modal above the stage.
- The stage spatializes product functions as a quest board, runtime portal,
  memory cabinet, projection-scroll desk, and permission-contract station.
- The factual log and keyboard footer remain visible without permanently
  shrinking the stage.

### Default World and Companion

- The sole V0 world is the Midnight Ninety-Nine Workshop, a cross-runtime
  adventurers guild with restrained tsukumogami-inspired Japanese details.
- The visual grammar uses dark wood, aged brass, parchment, indigo textile,
  cyan spirit light, vermilion urgency, hard color blocks, and coarse outlines.
- The sole visible V0 actor is the main companion and workshop registrar: the
  tsukumogami of the guild contract ledger and Chronicle.
- Her canonical identity is:
  - `actor_id: shiori`
  - display name `栞`
  - romanized name `Shiori`
  - title `九十九工房书记官`
  - owner address `会长`
- The presentation actor never supplies factual executor identity; an empty Chronicle keeps `source_spirit_id` absent.
- Shiori is calm and exacting, with restrained warmth, dry literal humor, and
  visible concern for ambiguous or unfiled work.
- She stays quiet during ambient work and speaks proactively for blocking
  permissions, continuity risks, significant memories, and task settlement.
- Her terminal silhouette uses a 2.25-head chibi proportion, silver-gray bob
  hair with one bookmark-like side braid, an indigo book-cover mantle, and a
  parchment-toned registrar uniform.
- Her primary identifiers are a brass-cornered oversized contract ledger, a
  cyan bookmark spirit flame, and one vermilion sealing-wax accent.
- Executor attribution remains visible through the portal plaque, status bar,
  and log. Shiori has no second independent growth ledger in V0.

### Modular Presentation Packs

- Shiori and the Midnight Ninety-Nine Workshop ship as the bundled
  `default-shiori` presentation pack.
- The minimal core, host contracts, persistence, and safety plane contain no
  Shiori identity, guild terminology, dialogue, palette, or sprite dependency.
- V0 accepts one external versioned presentation-pack directory through
  `--presentation-pack <directory>`.
- A pack may supply companion metadata, terminology, line books, palettes,
  scene assets, and deterministic sprite frames through validated data.
- User-authored packs are inert presentation data. V0 excludes executable
  scripts, runtime prompt injection, network downloads, hot reload, marketplace
  flows, and multi-pack composition.

### Approved Visual References

- `docs/visual-references/tsukumo-v0-workshop-concept-v1.png`
  is the approved default-screen composition and art-direction contract.
- `docs/visual-references/tsukumo-v0-shiori-character-reference-v1.png`
  is the approved identity and five-pose contract for Shiori.
- Generated labels are illustrative. Canonical product copy lives in the
  versioned default pack and typed UI model.

## Requirements

- **R1 Terminal lifecycle:** alternate screen, raw mode, resize handling,
  bounded tick rate, clean quit, panic/error restoration, and no leaked cursor
  or terminal state.
- **R2 Product read model:** host-owned runtime, execution, handoff, state
  evidence, checkpoint/projection metadata, selected StateRefs, omissions, and
  pending permission views.
- **R3 Typed actions:** navigation, refresh, state revoke, allow once, allow
  session, deny, and quit route through typed host actions.
- **R4 Product surfaces:** workshop, state inspector, projection inspector, and
  blocking permission modal.
- **R5 Visual fidelity:** stage dominance, mapped facilities, Shiori identity,
  five semantic poses, CJK-safe text, and visible keyboard affordances follow
  the approved references.
- **R6 Adaptive terminal behavior:** full, compact, and text-fallback layouts;
  truecolor, ANSI-256, and monochrome-safe communication; reduced-motion mode.
- **R7 Attention discipline:** ambient presentation stays quiet; permissions,
  failures, and completion receive distinct pose, copy, and border treatment.
- **R8 Pack loading:** bundled default pack works without flags; one explicit
  external directory is parsed and validated before theater receives it.
- **R9 Pack safety:** pack data cannot execute code, alter runtime prompts,
  access repositories/processes directly, or authorize host actions.
- **R10 Presentation boundary:** persona and world copy stay outside runtime
  prompts, durable semantic events, storage records, and safety decisions.

## Acceptance Criteria

- [x] A real TTY session opens, redraws, responds to keys, resizes, and restores
      the terminal after normal quit, injected error, and panic-hook exercise.
- [x] At the supported full size, the workshop visibly dominates the screen and
      every product surface remains reachable by keyboard.
- [x] The workshop shows runtime, execution, handoff, attention, Shiori, mapped
      facilities, and bounded factual logs.
- [x] Idle, work, wait, urgent, and celebrate poses remain distinguishable after
      normalization to the terminal sprite budget.
- [x] The state inspector shows value, scope, strength, status, and source event
      refs; revoke emits a typed host action and refreshes the view.
- [x] The projection inspector shows checkpoint/projection IDs, selected refs,
      versions, budget use, and omissions without rendered prompt text.
- [x] The permission modal shows redacted tool arguments, cwd, risk, and runtime
      and supports allow-once, allow-session, and deny with no implicit default.
- [x] The bundled `default-shiori` pack reproduces the approved identity,
      palette, terminology, facilities, and line-book contract.
- [x] A fixture external pack changes presentation content without recompiling
      or changing serialized KernelEvent semantics.
- [x] Invalid or unsafe packs fail with actionable errors before rendering and
      produce no host side effects.
- [x] Full, compact, monochrome, reduced-motion, CJK, action-routing,
      terminal-restoration, and buffer tests pass.

## Out of Scope

- Multiple production characters.
- Multiple active packs, inheritance, merging, hot reload, marketplace, or
  remote installation.
- General-purpose asset authoring or conversion tools.
- Arbitrary scripts, plugins, hooks, or runtime prompt customization in packs.
- Complex animation, pathfinding, collision, mouse-first interaction, or
  multiple world themes.
- General database or process access from theater.