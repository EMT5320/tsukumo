# Architecture and Boundaries

## Workspace Ownership

The root `Cargo.toml` defines one Rust workspace. Crates own these concerns:

| Crate | Owns | Must not own |
|---|---|---|
| `tsukumo-kernel` | Normalized contracts, persistent/runtime identity types, replay/session primitives | Vendor payloads, rendering, canonical relationship storage |
| `tsukumo-adapters` | Vendor/protocol decoding and runtime-specific transport details | Theater state, canonical soul state, presentation persona |
| `tsukumo-soul` | Relationship-state storage, recall, briefing/checkpoint inputs, evidence traces | Vendor process control, stage rendering |
| `tsukumo-theater` | Pure direction, stage state reduction, rendering, attention presentation | Vendor types, prompt assembly, canonical state writes |
| future `tsukumo-host` | Composition root, process lifecycle, event envelope assignment, safety/UI coordination | A second copy of adapter parsing or soul persistence logic |

Evidence: workspace manifests under `crates/*/Cargo.toml`, module-level contracts
in each `src/lib.rs`, and the C1 flow in
`docs/tsukumo-vision-state-handoff-convergence-2026-07-10.md` §16.

## Dependency Direction

Use this direction:

```text
runtime process
    -> adapter
    -> kernel event contract
    -> host composition
       -> soul / Chronicle
       -> director -> stage world -> renderer
```

- `kernel` does not import the other product crates.
- `adapters` depend on `kernel`; theater is allowed only as a dev-dependency for
  end-to-end adapter tests.
- `theater` depends on `kernel`, never on adapters or soul.
- `soul` may share kernel identity/evidence types, but does not depend on
  adapters or theater.
- The host wires adapter output, state projection, Chronicle, safety, and UI.
  Do not add adapter-to-soul coupling to avoid creating a circular composition
  path.

Current examples:

- `crates/tsukumo-adapters/tests/a1_drive_stage.rs` performs cross-crate wiring
  as a test without putting theater in adapter production dependencies.
- `crates/tsukumo-theater/src/drive.rs` accepts only `KernelEvent`.
- `crates/tsukumo-adapters/src/briefing.rs` exposes an assembly seam while the
  comment explicitly assigns briefing content ownership to soul/host.

## Composition Rules

- Side effects belong at boundaries: host, adapter transport, storage, and
  renderer entry points.
- Transformations used by replay or tests should be pure when possible.
  `tsukumo_theater::direct` is the reference pattern.
- Put vendor schema parsing in one adapter module. Consumers use normalized
  types rather than reading raw `serde_json::Value` fields again.
- A new runtime adds a binding/adapter; it does not create a vendor-specific
  spirit, memory table, or theater branch.

## Avoid

- Vendor names or ACP/stream-json types in `tsukumo-theater`.
- `builtin`, Claude, and Codex growth ledgers with separate schemas.
- Prompt-persona text in relationship projections.
- A host that buffers an entire live stream before emitting the first event.
- Convenience dependencies that reverse the ownership graph.

