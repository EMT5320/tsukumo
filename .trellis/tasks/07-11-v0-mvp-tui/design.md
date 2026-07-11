# V0 MVP TUI — Technical Design

## Boundaries

```text
host repositories/process/safety
  -> ProductReadModel
  -> theater AppState + renderer
  -> UiAction
  -> host controller
```

Theater receives immutable read models and returns typed actions. It never opens
SQLite, spawns a runtime, or applies a StateWriter transition directly.

## App Model

The UI state owns navigation and ephemeral presentation only:

```rust
pub struct ProductReadModel {
    pub runtime: RuntimeStatusView,
    pub handoff: HandoffStatusView,
    pub states: Vec<StateView>,
    pub projection: Option<ProjectionView>,
    pub pending_permission: Option<PermissionView>,
}

pub enum Screen {
    Workshop,
    StateInspector { selected: usize },
    ProjectionInspector,
}

pub enum UiAction {
    Refresh,
    RevokeState(StateId),
    DecidePermission(PermissionRequestId, PermissionDecision),
    Quit,
}
```

Host maps actions to deterministic services and returns a new read model.

## Layout and Interaction

- Normal mode keeps the workshop/log split and adds a compact status bar.
- State and projection inspectors are overlays or full panes selected by keys.
- A pending permission always places a modal above normal navigation.
- Small terminals render a status/message fallback with required dimensions.
- Key bindings are visible in the footer and avoid hidden destructive actions.

## Terminal Lifecycle

A narrow RAII terminal guard owns raw mode, alternate screen, cursor state, and
cleanup. The event loop separates input, tick, render, and host-action handling.
Errors propagate after cleanup; panic-hook integration restores the terminal
before delegating to the previous hook.

## Tests

- Pure reducer tests for navigation and action production.
- Buffer tests for every screen/modal at normal and compact sizes.
- CJK width/border tests.
- Fake host controller tests for revoke and permission routing.
- PTY/manual visual evidence on Windows Terminal before release packaging.

## Rollback

The TUI is additive over host actions/read models. Reverting presentation code
does not alter Chronicle, state, checkpoint, or receipt data.
