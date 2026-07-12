# V0 MVP TUI — Technical Design

## Source Contracts

- Product requirements: `prd.md`
- Project north star: `DESIGN.md`
- Visual contract:
  `docs/visual-references/tsukumo-v0-visual-contract.md`
- Approved screen reference:
  `docs/visual-references/tsukumo-v0-workshop-concept-v1.png`
- Approved Shiori reference:
  `docs/visual-references/tsukumo-v0-shiori-character-reference-v1.png`
- Required Trellis guidance:
  - `.trellis/spec/rust/architecture-and-boundaries.md`
  - `.trellis/spec/rust/types-and-event-contracts.md`
  - `.trellis/spec/rust/theater.md`
  - `.trellis/spec/rust/error-handling.md`
  - `.trellis/spec/rust/quality-and-testing.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`

## Design Principles

1. **Minimal semantic core:** Kernel, Soul, runtime control, and safety contain
   no world, persona, copy, palette, or sprite dependency.
2. **Stage-first product:** the workshop is the default information surface;
   inspectors and permissions retain direct factual control.
3. **Stable relationship face:** Shiori remains visible across runtime switches;
   executor attribution remains explicit.
4. **Inert customization:** external packs supply validated presentation data
   only.
5. **Lossy theater, lossless authority:** animation may merge frames; host
   actions, Chronicle, receipts, state, and permissions never wait for theater.
6. **Accessibility before decoration:** keyboard, CJK, reduced motion, compact
   layout, and non-color signaling are first-class contracts.

## Ownership and Data Flow

```text
external pack directory / embedded default
    -> tsukumo-host reads bounded files
    -> tsukumo-theater parses typed data and validates pure invariants
    -> immutable ValidatedPresentationPack

runtime -> adapter -> KernelEvent -> host
    -> Chronicle / Soul / Safety
    -> pure Director + DirectorContext(pack presentation)
    -> StageEvent(attribution)
    -> StageWorld reducer
    -> TuiApp(ProductView + StageWorld + pack)
    -> ratatui::Buffer
    -> terminal

keyboard
    -> pure TuiApp reducer
    -> UiAction
    -> host controller
    -> refreshed ProductView
```

Crate ownership:

| Concern | Owner |
|---|---|
| Presentation pack model and pure validation | `tsukumo-theater` |
| Embedded/default source and external directory I/O | `tsukumo-host` |
| Stage attribution, reducer, layout, HalfBlock rendering | `tsukumo-theater` |
| Runtime/state/projection/permission read-model assembly | `tsukumo-host` |
| UI-facing immutable view types and `UiAction` | `tsukumo-theater` |
| Side effects for revoke and permission decisions | `tsukumo-host` |
| Raw mode, alternate screen, input, resize, restoration | `tsukumo-host::tui` |

Theater imports no host, adapter, or soul type. Host constructs theater-owned
view types and handles theater-owned typed actions.

## Presentation Pack

### V0 Directory Shape

```text
<pack-root>/
├── pack.json
├── scene.json
└── sprites/
    └── shiori.json
```

V0 uses JSON because `serde_json` is already a workspace dependency. The
format stays strict, versioned, and dependency-light. Richer authoring formats
may be added after V0 behind the same typed model.

`pack.json` owns:

- `schema_version`, `id`, and display metadata;
- companion identity, title, and owner address;
- terminology and line book;
- truecolor tokens plus ANSI-256/monochrome mappings;
- relative references to scene and sprite assets.

`scene.json` owns logical-pixel facilities, layers, anchors, and walk bounds.
`sprites/shiori.json` owns palette-indexed frames and animation sequences.

### Model

```rust
/// Immutable presentation data consumed by the theater.
pub struct PresentationPack {
    pub manifest: PackManifest,
    pub world: WorldPresentation,
    pub companion: CompanionPresentation,
    pub terminology: Terminology,
    pub line_book: LineBook,
    pub palette: Palette,
    pub scene: SceneDefinition,
    pub sprites: SpriteAtlas,
}

/// Selects the embedded default or one explicit external directory.
pub enum PresentationPackSource {
    EmbeddedDefault,
    Directory(PathBuf),
}
```

The exact source enum belongs to host. Theater owns `PresentationPack` and
`ValidatedPresentationPack`.

### Loading and Validation

1. Host selects the embedded default when the flag is absent.
2. `--presentation-pack <directory>` selects one explicit external root.
3. Host reads bounded files before entering raw/alternate-screen mode.
4. Theater deserializes and validates all cross-file references.
5. Host passes one immutable validated pack into the app.
6. An explicit invalid pack returns an actionable error; it never silently
   falls back to the bundled pack.

V0 limits:

- total decoded pack input: 4 MiB;
- palette: at most 32 entries;
- scene: at most 120x60 logical pixels;
- sprite frame: at most 32x40 logical pixels;
- animation: at most 16 frames per semantic pose;
- copy: at most 512 characters per entry;
- paths: relative, no `..`, no absolute paths, and canonical targets remain
  inside the chosen root;
- palette indices and every referenced asset must resolve.

Errors remain typed:

```rust
pub enum PresentationPackError {
    Io { path: PathBuf, source: io::Error },
    InvalidJson { path: PathBuf, source: serde_json::Error },
    UnsupportedSchema { found: u16 },
    InvalidPath { path: PathBuf },
    MissingAsset { path: PathBuf },
    DuplicateId { id: String },
    LimitExceeded { field: &'static str, maximum: usize },
    InvalidPaletteIndex { index: u8 },
    InvalidModel { field: &'static str, reason: String },
}
```

Bundled-default fixtures pass the same parser and validator as external packs.

## Identity and Attribution

Shiori is a presentation actor. A `KernelEvent.spirit_id` remains the source
executor identity. Stage events carry both facts:

```rust
/// Keeps the visible actor separate from the factual executor.
pub struct StageAttribution {
    pub actor_id: PresentationActorId,
    pub source_spirit_id: SpiritId,
}
```

`DirectorContext` receives the pack's actor ID and line book. The pure Director
uses that context to emit Shiori reactions while retaining the source Spirit.
`StageWorld` keys visible actors by `PresentationActorId`; the runtime plaque and status consume the host-authoritative runtime `source_spirit_id`, while logs preserve each event's attributed source and `RuntimeBinding`.

This changes theater presentation serialization only. `KernelEvent`, Chronicle,
Soul state, projection receipts, and runtime fixtures keep their current wire
contracts.

## Product View and Actions

```rust
pub struct ProductView {
    pub runtime: RuntimeStatusView,
    pub execution: ExecutionStatusView,
    pub handoff: HandoffStatusView,
    pub states: Vec<StateView>,
    pub projection: Option<ProjectionView>,
    pub pending_permission: Option<PermissionView>,
    pub notices: Vec<NoticeView>,
}

pub enum Screen {
    Workshop,
    StateInspector { selected: usize },
    ProjectionInspector,
}

pub enum UiAction {
    Refresh,
    RevokeState(StateId),
    DecidePermission(UiPermissionId, PermissionDecision),
    Quit,
}
```

View types contain bounded, redacted display data. They contain no rendered
prompt, raw credentials, unbounded tool arguments, database connection, process
handle, or repository handle.

The pure app reducer maps input to state plus an optional `UiAction`. Host
executes actions and supplies a refreshed `ProductView`.

## Layout

### Full: >=100x30

- header/border: 2 rows;
- workshop stage: 19 rows;
- factual log: 6 rows;
- footer: 3 rows.

The stage keeps the five facilities and open walking space. The inspector opens
as an overlay or replacement pane. The permission modal overlays every screen.

### Compact: 72-99 x 22-29

- preserve Shiori, runtime, attention, shortened log, and footer;
- reduce decorative facility layers;
- use full-pane inspectors;
- preserve every permission field and decision.

### Text Fallback: below 72x22

- show runtime, handoff, attention, pending permission, and resize guidance;
- preserve all keyboard decisions;
- suppress scenery and animation;
- preserve CJK and risk text without clipping.

## Rendering

### Deterministic HalfBlock Packing

The existing `Canvas<Points>` placeholder is replaced for production assets by
a direct logical-pixel renderer:

- top logical pixel -> cell foreground;
- bottom logical pixel -> cell background;
- cell symbol -> `▀`;
- transparent pairs preserve the underlying scene;
- palette resolution happens once per capability mode.

This avoids Canvas resampling and keeps pack frames deterministic in tests.

Implemented module split (representative):

```text
tsukumo-theater/src/
├── app/
│   ├── mod.rs
│   ├── model.rs
│   └── reducer.rs
├── pack/
│   ├── mod.rs
│   ├── model.rs
│   └── validation.rs
├── render/
│   ├── mod.rs
│   ├── halfblock.rs
│   ├── layout.rs
│   ├── workshop.rs
│   ├── inspectors.rs
│   └── permission.rs
├── director.rs
├── stage.rs
└── world.rs
```

Host terminal ownership lives in `tsukumo-host/src/tui.rs` with `tui/lifecycle.rs`, `tui/input.rs`, and `tui/local.rs`.

Keep pure modules small and documented. Avoid a second palette decoder, layout
calculator, or action transition table.

### Motion

- logic tick: 10 Hz;
- render cap: 20 Hz, with event/resize-driven invalidation;
- animation never delays execution;
- intermediate frames may be dropped;
- reduced motion freezes each semantic state on its key frame;
- urgent pulse also has a static border/label/pose signal.

## Shiori State Contract

| State | Required large-shape cue |
|---|---|
| Idle | Closed ledger held upright |
| Work | Ledger/scroll opens horizontally; writing arm extends |
| Wait | Pen pauses; gaze lifts |
| Urgent | Open ledger plus raised vermilion seal |
| Celebrate | Downward completion stamp and restrained smile |
| Upset/degraded | Reuse Urgent/Wait assets in V0 with distinct copy and border |

V0 does not add a sixth production animation solely for Upset. The app composes
available frames with distinct attention treatment.

## Terminal Lifecycle

A narrow RAII guard owns:

- raw mode;
- alternate screen;
- cursor visibility;
- mouse capture state when applicable;
- terminal restoration.

The event loop separates input, tick, render, resize, and host-action handling.
Pack loading and host preflight happen before the guard. Errors propagate after
restoration. A panic hook attempts restoration, then delegates to the previous
hook.

## Inclusive Personas and Adaptive Behavior

| Persona/context | Primary task | Pass condition |
|---|---|---|
| Long-session guild master | Monitor agents without losing focus | Ambient screen remains quiet; urgent events are unmistakable |
| Keyboard-only operator | Inspect state and decide permissions | Every action is visible, focusable, and deterministic |
| CJK user | Read labels, logs, and bubbles | No border corruption or wide-char clipping |
| Split-pane user at 80x24 | Retain control in limited space | Compact mode preserves state and permission decisions |
| Reduced-motion user | Operate without pulse/travel motion | Semantic key frames and labels communicate every state |
| Low-color/remote terminal | Distinguish state without truecolor | Pose, text, and border repeat the signal |

## Tests and Evidence

### Pure/Fixture Tests

- pack model round trips and unsupported schema;
- path traversal, oversized input, unresolved assets, palette index, duplicate
  IDs, and malformed JSON;
- embedded default and external fixture produce equivalent semantic surfaces;
- Director preserves source Spirit while targeting the configured actor;
- reducer navigation and exhaustive key mapping;
- action routing for revoke and permission decisions;
- HalfBlock packing and transparent overlay;
- Full/Compact/Fallback buffer snapshots;
- CJK borders and truncation;
- truecolor/ANSI-256/monochrome mapping;
- reduced-motion key-frame selection;
- prompt/secret sentinels absent from views and buffers;
- terminal restoration after normal exit, injected error, and panic-hook path.

### Manual/Visual Evidence

Capture fresh deterministic terminal evidence for:

- 100x30 full workshop and 80x24 compact workshop;
- text fallback, state inspector, projection inspector, and permission modal;
- truecolor, ANSI-256, monochrome, urgent monochrome, and reduced motion;
- every semantic pose plus motion start/settled endpoints.

Record Windows ConPTY receipts for normal `S -> Q` interaction and live
`80x24 -> 100x30` resize. Windows Terminal application-specific capture remains
a separate future environment check.

Run `omo:visual-qa` in terminal mode after implementation. Compare layout
hierarchy, facility silhouettes, Shiori identifiers, palette story, and
attention states against the approved references.

## Rollback

- Presentation-pack and UI code are additive over durable contracts.
- Reverting the renderer or app leaves Chronicle, State, Checkpoint, Receipt,
  and runtime data unchanged.
- An external pack failure occurs before terminal entry and before host action.
- The embedded default remains the recovery surface.