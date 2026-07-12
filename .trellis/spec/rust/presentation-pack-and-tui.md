# Presentation Pack and Product TUI

## Scenario: Versioned presentation packs and the interactive product surface

### 1. Scope / Trigger

Read this contract before changing any of the following:

- presentation-pack JSON, validation, semantic palette roles, scene geometry, or sprites;
- the host pack source, `--presentation-pack`, or `TSUKUMO_DATA_DIR`;
- theater `ProductView`, `AppState`, semantic UI inputs/actions, or product rendering;
- the host product controller, Chronicle/Soul projection, permission UI, or TUI lifecycle.

The presentation pack is inert display data. Kernel, Soul, runtime prompts, and permission authority remain independent of the selected pack. The host owns file I/O, durable actions, and terminal lifecycle. Theater owns pure validation, reduction, and rendering.

### 2. Signatures

```rust
// Pure parser: the host has already read and bounded all documents.
pub fn parse_presentation_pack(
    documents: PackDocuments<'_>,
) -> Result<ValidatedPresentationPack, PresentationPackError>;

// Host boundary: selects embedded content or one explicit directory.
pub fn load_presentation_pack(
    source: &PresentationPackSource,
) -> Result<ValidatedPresentationPack, PresentationPackLoadError>;

// Durable product authority consumed by the terminal loop.
pub trait ProductController {
    fn refresh(&mut self) -> Result<ProductSnapshot, ProductControllerError>;
    fn apply(&mut self, action: UiAction)
        -> Result<ProductControl, ProductControllerError>;
}

// Soul query used by bounded product read-model assembly.
impl SoulStore {
    pub fn list_active_states_limited(
        &self,
        limit: usize,
    ) -> Result<Vec<StateRecord>, SoulError>;

    pub fn replay_permission_events(
        &self,
        maximum_events: usize,
        maximum_bytes: usize,
    ) -> Result<Vec<PersistedEvent>, SoulError>;

    pub fn latest_projection_event(
        &self,
        execution_id: Option<&ExecutionId>,
    ) -> Result<Option<PersistedEvent>, SoulError>;

    pub fn latest_checkpoint_event(&self) -> Result<Option<PersistedEvent>, SoulError>;
    pub fn latest_runtime_status_event(&self) -> Result<Option<PersistedEvent>, SoulError>;
}

pub fn run_tui(
    pack: &ValidatedPresentationPack,
    controller: &mut dyn ProductController,
    snapshot: ProductSnapshot,
    reduced_motion: bool,
) -> Result<(), TuiError>;
```

CLI and environment surface:

```text
tsukumo-host [--presentation-pack <directory>] [--reduced-motion]
TSUKUMO_DATA_DIR=<Soul and Chronicle directory>  # optional; defaults to ./data
```

`HostProductController::open(data_dir, pack)` and its first `refresh()` must finish before `run_tui` enters raw mode or the alternate screen.

### 3. Contracts

#### Presentation pack

- Schema version: `PACK_SCHEMA_VERSION == 1`.
- Directory shape: `pack.json`, one manifest-referenced scene JSON, and one manifest-referenced sprite JSON.
- The host reads at most 4 MiB across the three UTF-8 documents.
- External pack and data roots accept local drive paths only. UNC, verbatim,
  device, alternate-data-stream, terminal-unsafe, symlink/reparse, and
  hard-linked file inputs fail before pack parsing or SQLite open. On Windows,
  each path component is opened no-follow and retained with delete sharing
  disabled; pack bytes are read from the validated final handle, and product
  data/critical-file guards live longer than the SQLite connection. The main
  database uses `SQLITE_OPEN_NOFOLLOW`; `soul.db-journal`, `soul.db-wal`, and
  `soul.db-shm` are atomically pre-created or validated and then held no-follow
  with delete sharing disabled before SQLite opens. `journal_mode=PERSIST`
  fixes runtime writes to the guarded rollback journal; any WAL transition can
  only see the already-guarded regular sidecars and must fail closed if cleanup
  is incompatible with their lifetime locks. Existing data trees
  are walked under a 10,000-entry budget.
- All user-visible pack copy is non-empty, bounded to 512 characters, and rejects terminal control characters.
- Manifest asset paths are normalized relative paths and are opened beneath the guarded selected root.
- Required semantic palette roles are `ink`, `surface`, `border`, `text_primary`, `text_muted`, `accent`, and `urgent`.
- Every accepted theme keeps semantic foreground roles at or above 2.0:1
  contrast against ink and surface after resolving TrueColor, the standard
  ANSI-256 RGB table, and monochrome tones. Distinct palette indexes alone do
  not prove readable contrast. Permission decisions use a host-fixed
  high-contrast safety theme independent of pack colors.
- A transparent top or bottom logical pixel inherits the destination buffer
  background. It never resets that half-cell to the user's terminal default.
- Scene and sprite palette indexes must resolve inside `palette.colors`.
- Every scene declares `quest_board`, `runtime_portal`, `memory_cabinet`, `projection_desk`, and `permission_station` facility IDs because the renderer resolves those semantic anchors.
- Sprite animations provide `Idle`, `Work`, `Wait`, `Urgent`, and `Celebrate` semantic poses.
- `PresentationActorId` identifies the rendered companion. `SpiritId` identifies the event source. `StageAttribution` carries both values independently.
- Pack strings may influence theater labels, line-book copy, palette, scene pixels, and sprite pixels. They never enter `KernelEvent`, Soul state, runtime prompts, commands, permission scope, or runtime environment values.

#### Product snapshot

`ProductSnapshot` contains:

- `view`: bounded state, projection, runtime, execution, permission, and notice read models;
- `world`: a lossy `StageWorld` rebuilt from normalized Chronicle events and the selected `DirectorContext`;
- `revision`: newest replayed Chronicle sequence, or zero for an empty store.

The host replays at most the newest 1,000 Chronicle events and presents that tail in ascending sequence order. That tail is a lossy theater/log window and never defines current durable authority. Soul remains authoritative for active state and projection receipts. The product read model queries the latest checkpoint, projection, and coherent runtime-status event directly. Permission reconstruction scans only durable request/decision events under explicit 4,096-event and 32 MiB budgets; overflow returns `SoulError::ChronicleReadBudgetExceeded` instead of silently forgetting a pending request.

A permission request is identified internally by `(execution_id, session_id, runtime, VendorEventRef)`. The raw vendor reference remains unchanged in durable payloads, while the scoped key and derived `UiPermissionId` prevent two executions that reuse one vendor ID from resolving each other.

The state inspector queries at most 257 active states, exposes the first 256, and adds a visible truncation receipt when a further row exists. State evidence uses three-entry pages and projection receipts use six-entry pages so metadata plus a truncation receipt fit the minimum compact body. Visible item ranges are one-based when data exists and `0-0/0` when empty. State rows compact durable IDs by retaining both prefix and a twelve-cell suffix before value text, so a selected revoke target stays distinguishable. An empty Chronicle keeps `source_spirit_id` as `None`; presentation actor IDs never synthesize factual executor identity. Header and runtime plaque consume the same authoritative runtime source while event logs retain per-event attribution.

#### UI actions

| Action | Host effect |
|---|---|
| `Refresh` | Rebuild the product snapshot and append a bounded informational notice. |
| `RevokeState(StateId)` | Commit a typed Soul state transition with source and lifecycle events, then refresh. |
| `DecidePermission(UiPermissionId, decision)` | Resolve the matching reconstructed request and append a durable Chronicle decision; the unwired vendor bridge stays closed. |
| `Quit` | Return `ProductControl::Quit`; terminal RAII restores raw mode, alternate screen, and cursor visibility. |

The permission overlay has input priority. Navigation cannot dismiss or bypass a pending decision. Up/Down traverses every risk reason and every stable 80-character page inside one long reason; decisions remain anchored at the bottom. Reduced-motion mode freezes sprite animation and world walking at semantic key frames.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Newer/unsupported pack schema | `PresentationPackError::UnsupportedSchema` |
| Malformed manifest, scene, or sprite JSON | Typed document parse error with the document name |
| Absolute, parent, empty, or escaping asset path | Pack validation error or `PresentationPackLoadError::AssetEscape` |
| Aggregate pack input exceeds 4 MiB | `PresentationPackLoadError::FileTooLarge` before parsing |
| Pack/data path is remote, device-like, reparse/symlinked, hard-linked, or terminal-unsafe | Typed local-path rejection before parsing or storage open |
| Existing local data tree exceeds 10,000 entries | Typed local-path budget rejection; open nothing |
| Invalid UTF-8 asset | `PresentationPackLoadError::InvalidUtf8` |
| Palette role or pixel index out of range | Typed pack validation error before terminal entry |
| Semantic foreground resolves below 2.0:1 contrast with ink or surface in any color capability | Typed pack validation error before terminal entry |
| Permission authority exceeds its event or byte budget | `SoulError::ChronicleReadBudgetExceeded`; do not assemble a lossy pending set |
| Required pose/frame missing or duplicated | Typed sprite validation error before terminal entry |
| Pack copy contains ESC, newline, or another control character | `PresentationPackError::InvalidModel` before rendering |
| A required semantic facility ID is absent | `PresentationPackError::InvalidModel` before rendering |
| Soul/Chronicle open or refresh failure | `ProductControllerError::Soul`; terminal mode remains untouched |
| UI permission ID no longer resolves | Warning notice and safe refresh path; no vendor permission is granted |
| Permission lacks a durable projection receipt | `ProductControllerError::MissingPermissionReceipt` |
| A durable TUI action has no Chronicle source Spirit | `ProductControllerError::MissingSourceSpirit`; write nothing |
| Alternate-screen entry fails after raw mode starts | Raw mode is restored before `TuiError` crosses the boundary |
| Panic during an active TUI | Process-wide restoration hook performs best-effort terminal cleanup once |
| Terminal below compact threshold | Render the bounded factual fallback, including permission decisions when pending |

### 5. Good / Base / Bad Cases

- **Good:** an external pack changes actor art, world terminology, palette, and line-book text; the same Chronicle events retain their `SpiritId`, Soul records, runtime prompt, and permission scope.
- **Base:** the embedded Shiori pack opens an empty `TSUKUMO_DATA_DIR`, shows an offline workshop, accepts refresh and quit, and leaves no terminal modes active.
- **Bad:** an explicit pack directory is missing or malformed. The host returns a typed error and does not silently fall back to embedded content.
- **Bad:** a screen is assembled from a static fixture in the production entrypoint. The product must come from `HostProductController::refresh()`.
- **Bad:** render code counts CJK glyphs by Unicode scalar count. Width and truncation must use `unicode-width` cell semantics.

### 6. Tests Required

- `tsukumo-theater/tests/presentation_pack_contract.rs`
  - assert schema, path, palette-role, pixel-index, duplicate-frame, control-copy, required-facility, and readable-text validation;
  - assert every accepted index and semantic role resolves safely.
- `tsukumo-theater/tests/presentation_pack_security_contract.rs`
  - assert DOS superscript device aliases, ANSI `0/16` and `15/231` aliases, and near-equal TrueColor values fail closed.
- `tsukumo-host/tests/presentation_pack_contract.rs`
  - assert embedded default selection, explicit-directory override, no fallback on explicit errors, and five distinct Shiori poses.
- `tsukumo-host/tests/presentation_boundary_contract.rs`
  - assert default persona strings stay absent from `KernelEvent` and runtime prompts.
- `tsukumo-host/tests/product_controller_contract.rs`
  - assert empty-store refresh with no fabricated source, human-readable state scope, durable revoke, durable permission denial, and runtime sidecar replacement failure under a successful persistent-journal transaction.
- `tsukumo-host/tests/product_render_contract.rs`
  - assert full/compact/fallback geometry, attention-preserving headers, authoritative runtime-plaque attribution, and CJK-safe scene bands.
- `tsukumo-host/tests/product_inspector_render_contract.rs` and `product_inspector_edge_contract.rs`
  - assert scrolling state selection, distinguishable revoke IDs, one-based paged evidence/projection ranges, inspector refs, styled projection spans, and minimum-compact truncation receipts.
- `tsukumo-host/tests/product_permission_render_contract.rs`
  - assert paged permission evidence, all three decisions at every layout tier, and host-fixed high-contrast safety colors.
- `tsukumo-theater/tests/app_reducer_contract.rs`
  - assert semantic navigation/actions, modal priority, redaction, and reduced-motion behavior.
- `tsukumo-host/tests/local_path_contract.rs`
  - assert remote/device roots fail before access, reserved/ADS/superscript-device paths create no store, hard-linked trees and junction components fail closed, guarded data roots resist concurrent replacement, and an ordinary local directory opens.
- `tsukumo-soul/tests/c1_chronicle_read.rs`
  - assert permission budget overflow is typed and latest checkpoint,
    projection, and runtime queries use Chronicle sequence and execution scope.
- `tsukumo-host/src/tui/lifecycle.rs`
  - assert entry rollback and the ordered restoration operation set through fake mode operations.
- Final QA must run a real PTY session and verify successful quit plus cursor restoration, alongside fresh visual captures for every enumerated mode, ANSI-256, monochrome urgent, and motion endpoints.

### 7. Wrong vs Correct

#### Wrong

```rust
// This leaks a replaceable presentation persona into runtime authority.
let prompt = format!("You are {}. {base_prompt}", pack.companion().display_name);
runtime.spawn(&prompt)?;
```

```rust
// This freezes production behavior at a demo snapshot.
run_tui(&pack, &mut FakeController, initial_product_view(), false)?;
```

#### Correct

```rust
// The pack configures only the theater transform and renderer.
let director = DirectorContext::from_pack(&pack);
let snapshot = controller.refresh()?;
run_tui(&pack, &mut controller, snapshot, reduced_motion)?;
```

```rust
// Event-source identity and rendered actor identity remain independently typed.
let attribution = StageAttribution {
    actor_id: pack.companion().actor_id.clone(),
    source_spirit_id: event.spirit_id.clone(),
};
```
```rust
// Wrong: the 1,000-event theater tail silently defines permission authority.
let permissions = rebuild_permissions(&store.replay_recent_events(1_000)?)?;
```

```rust
// Correct: the lossy tail drives theater while scoped authority has its own budget.
let stage_events = store.replay_recent_events(1_000)?;
let permission_events = store.replay_permission_events(4_096, 32 * 1024 * 1024)?;
let permissions = rebuild_permissions(&permission_events)?;
```