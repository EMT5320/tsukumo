# Theater

## Layering

The theater pipeline has three responsibilities:

```text
KernelEvent -> pure Director -> StageEvent -> StageWorld reducer -> renderer
```

- `director.rs` maps normalized facts into presentation events.
- `world.rs` owns mutable stage state and lossy realtime reduction.
- `render.rs` reads `StageWorld` and renders; it does not parse kernel/vendor
  payloads or mutate canonical state.
- `drive.rs` is thin orchestration shared by fixtures and demos.

## Pure Director

Follow `crates/tsukumo-theater/src/director.rs`:

- no I/O, clocks, process calls, or mutable globals;
- input is a normalized event plus explicit context;
- output is `Vec<StageEvent>`;
- line-book overrides are data, not hidden global configuration;
- every meaningful mapped event leaves a log line.

Presentation persona is applied here or in later presentation transforms. It
must not be injected into the external runtime prompt.

## Realtime State

`StageWorld` is a lossy realtime view, not the historical source of truth.

- Animation may merge/drop intermediate visual frames.
- The execution path never waits for animation.
- `Urgent` is reserved for blocking approval/failure; normal work is `Focus`,
  settled state is `Ambient`.
- Chronicle/log persistence must not depend on the stage log cap.
- Spirit selection uses persistent IDs, not vendor names.

## Rendering

- Keep HalfBlock-compatible rendering as the universal path.
- Render into a `ratatui::Buffer` so core output can be tested without a TTY.
- Use `unicode-width` semantics for CJK/wide characters. The
  `buffer_to_string` implementation and render tests are the current examples.
- Keep terminal capability upgrades outside the Director and state reducers.

## Tests

For theater changes, cover the smallest applicable layers:

- Director unit test for event-to-stage mapping.
- StageWorld reducer/snapshot test for state transitions.
- Buffer/string render test for visible layout or CJK behavior.
- Fixture replay when a kernel contract or multi-event sequence changes.

References:

- `crates/tsukumo-theater/src/director.rs`
- `crates/tsukumo-theater/src/world.rs`
- `crates/tsukumo-theater/src/render.rs`
- `crates/tsukumo-theater/tests/fixture_replay.rs`

