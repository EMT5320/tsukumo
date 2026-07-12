# Tsukumo V0 Visual Contract

> Status: approved by the owner on 2026-07-11.
> Scope: default workshop screen and Shiori identity for the V0 TUI.
> Rendering target: Ratatui + HalfBlock with deterministic pack assets.

## Reference Artifacts

| Artifact | Contract |
|---|---|
| `tsukumo-v0-workshop-concept-v1.png` | Default-screen composition, spatial grammar, material, hierarchy, and palette |
| `tsukumo-v0-shiori-character-reference-v1.png` | Shiori identity, silhouette, props, and semantic poses |

Reference priority:

1. Product semantics and safety constraints in the active Trellis PRD.
2. Spatial composition and art direction in the workshop concept.
3. Shiori identity invariants in the character reference.
4. Terminal constraints and accessibility rules in this document.

Generated text is illustrative. Pack data and typed read models own canonical
copy.

## Experience Brief

The user is the guild master of a workshop that stays quietly alive during long
agent sessions. The default view must communicate runtime state at a glance,
preserve focus during ambient work, and make permission blocking unmistakable.

Design principles:

1. Stage first: the workshop reads as a place before it reads as a dashboard.
2. Factual control: narrative presentation never hides runtime, risk, or action.
3. Quiet continuity: Shiori provides a stable relationship face across runtime
   switches.
4. Meaningful motion: every animation communicates a semantic state.
5. Replaceable content: world, persona, copy, palette, scene, and sprite data
   come from a presentation pack.

## Layout Contract

### Full Layout

Target: at least 100x30 terminal cells.

| Region | Approximate budget | Purpose |
|---|---:|---|
| Header/border | 2 rows | Product/world title and attention state |
| Workshop stage | 19 rows | Shiori, facilities, runtime portal, movement space |
| Factual log | 6 rows | Bounded events and operator-relevant status |
| Footer | 3 rows | Current state and visible keyboard actions |

The stage receives roughly 70% of usable height. State and projection inspectors
open as overlays or replacement panes; they do not permanently consume stage
width.

### Compact Layout

Target: 72-99 columns and 22-29 rows.

- Preserve Shiori, runtime plaque, current attention, shortened log, and footer.
- Simplify decorative facility detail while keeping semantic destinations.
- Inspectors use full-pane views.
- Permission modal keeps its full decision set.

### Text Fallback

Below 72x22:

- Render runtime, handoff, attention, pending permission, and resize guidance.
- Preserve all keyboard decisions.
- Suppress pixel scenery and motion.
- Never clip CJK, risk reasons, or permission choices.

## Spatial Grammar

| Facility | Product meaning | Shiori action |
|---|---|---|
| Quest board | Task and handoff | Reads, pins, or updates a notice |
| Runtime portal | Runtime binding and switch | Registers arrivals and watches transitions |
| Memory cabinet | Durable state | Files or retrieves a record |
| Projection desk | Checkpoint and projection | Prepares a bounded scroll |
| Contract station | Permission request | Opens the ledger and raises the seal |

Executor identity remains on the portal plaque, status bar, and factual log.

## Color Tokens

Tokens were extracted from the approved concepts and rounded to stable values.

| Token | Value | Use |
|---|---|---|
| `ink-950` | `#01060C` | Terminal backdrop |
| `ink-900` | `#12181E` | Stage shadow and log surface |
| `wood-800` | `#2D1D15` | Furniture shadow |
| `wood-600` | `#653F22` | Furniture midtone |
| `indigo-700` | `#1F2A41` | Shiori mantle and banners |
| `brass-500` | `#9B6E49` | Frames, ledger corners, separators |
| `parchment-300` | `#D9B28F` | Paper, primary warm surface |
| `silver-400` | `#AFA0A2` | Shiori hair |
| `spirit-cyan-400` | `#44CEDE` | Portal, spirit flame, focus |
| `spirit-cyan-800` | `#07596F` | Cyan shadow and ANSI fallback |
| `vermilion-500` | `#8C3C2B` | Permission and urgent seal |
| `vermilion-800` | `#5D2019` | Urgent shadow |
| `text-primary` | `#E4CFB8` | Primary readable text |
| `text-muted` | `#918E82` | Secondary factual text |

Color never carries state alone. Pose, copy, border shape, and labels repeat the
same signal. ANSI-256 and monochrome mappings must preserve contrast order.

## Pixel and Material Rules

- Crisp nearest-neighbor clusters.
- Coarse dark outlines.
- Three-value shading per material.
- Hard stepped light bands.
- No gradients, dithering, antialiasing, glass, neon bloom, or soft painting.
- Props use modular tile-like geometry.
- Decorative detail yields to facility silhouette at compact sizes.
- Generated PNGs guide normalization; runtime assets are deterministic
  `SpriteFrame` and scene data.

## Shiori Contract

Canonical identity:

- `actor_id: shiori`
- display name: `栞`
- title: `九十九工房书记官`
- owner address: `会长`

The actor ID controls presentation only. An empty Chronicle displays no factual executor, and every visible runtime/source label comes from the host read model or attributed event.

Required identifiers:

1. Silver-gray bob with one bookmark side braid.
2. Indigo book-cover mantle and parchment registrar uniform.
3. Brass-cornered oversized ledger.
4. Cyan bookmark spirit flame.
5. One vermilion sealing-wax accent.

Target sprite envelope: approximately 16x20 logical pixels before HalfBlock
packing.

| Pose | Large-shape signal |
|---|---|
| Idle | Closed ledger held upright; compact silhouette |
| Work | Ledger or scroll opens horizontally; writing arm extends |
| Wait | Pen pauses; head and gaze lift |
| Urgent | Ledger opens; vermilion seal rises above the hand |
| Celebrate | Gold/brass completion stamp presses downward |

Facial micro-detail is optional. Braid, ledger, flame, and seal remain stable
across frames.

## Presentation-Pack Boundary

V0 ships `default-shiori` and accepts one explicit external directory through
`--presentation-pack <directory>`.

A validated pack may provide:

- manifest and schema version;
- companion metadata and titles;
- terminology and line book;
- palette and terminal fallback mappings;
- scene/facility definitions;
- deterministic sprite frames.

Pack loading and file I/O belong to host composition. Theater receives an
immutable validated pack. Packs have no scripts, network access, prompt
injection, process access, repository access, or host-action authority.

## Inclusive and Adaptive Requirements

| Context | Required behavior |
|---|---|
| Long-running focused session | Ambient motion stays quiet and bounded |
| Keyboard-only operation | Every surface and permission decision is reachable and labeled |
| CJK terminal | Width calculations preserve borders, labels, and bubbles |
| 80x24 or split pane | Compact layout preserves current state and decisions |
| ANSI-256 or monochrome | Labels, pose, and borders repeat color signals |
| Reduced motion | Freeze on semantic key poses; disable pulse and travel animation |
| Sensitive runtime data | Read models expose redacted summaries only |

## Accepted Design Debt

- The concept images contain more environmental detail than HalfBlock can retain.
  Manual normalization may simplify props while preserving silhouettes.
- Terminal font metrics vary. Tests cover supported sizes and CJK widths; exact
  glyph appearance remains terminal-dependent.
- V0 ships one visible companion. Multi-actor blocking, pathfinding, and scene
  crowding remain deferred.
- PNG-to-sprite automation is deferred. The approved images remain visual
  contracts; production frames are hand-normalized deterministic data.

## Generation Record

- Mode: built-in Codex image generation.
- Workshop prompt intent: stage-first 100x30 terminal UI, Midnight Ninety-Nine
  Workshop, mapped facilities, one Shiori actor, quiet log/footer.
- Character prompt intent: preserve the approved identity and show one neutral
  view plus Idle, Work, Wait, Urgent, and Celebrate pose studies.