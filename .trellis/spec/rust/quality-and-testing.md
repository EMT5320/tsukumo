# Quality and Testing

## Code Style

- Rust identifiers, doc comments, and source comments are English, matching the
  existing crates. User-visible theater copy may be localized.
- Run rustfmt; do not hand-align code against formatter output.
- Keep public modules documented with ownership/boundary intent.
- Prefer small typed helpers over repeated raw `serde_json::Value` extraction.
- Use exhaustive `match` statements for event and reducer transitions.

## Required Checks

Run from the workspace root for implementation changes:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If the project pins a toolchain/target, use that declaration consistently in
all four commands. Do not report checks as passing when dependency download,
linker, or target setup prevented execution; record the environmental blocker
separately from code failures.

The repository currently lacks CI, a pinned `rust-toolchain.toml`, and a
tracked `Cargo.lock`. This Windows workspace has both MSVC and GNU stable
installed; C1 verification uses the cached GNU toolchain explicitly and stays
offline:

```text
cargo +stable-x86_64-pc-windows-gnu <command> --offline
```

Do not infer remote reproducibility from this local gate. Tracking `Cargo.lock`,
pinning the tested toolchain/target, and Linux + Windows GNU CI remain assigned
to the cross-runtime/UI child.

## Test Layers

Use the narrowest test that proves the contract, plus cross-layer coverage when
data crosses crates:

| Layer | Local reference |
|---|---|
| Serialization/newtype round trip | `tsukumo-kernel/src/identity.rs` tests |
| Pure mapping | `tsukumo-theater/src/director.rs` tests |
| Reducer/snapshot | `tsukumo-theater/src/world.rs` tests |
| Persist/reopen/recall | `tsukumo-soul/tests/cross_session_recall.rs` |
| Adapter-to-stage integration | `tsukumo-adapters/tests/a1_drive_stage.rs` |
| Historical replay | `tsukumo-theater/tests/fixture_replay.rs` |

New behavior requires a test. Bug fixes require a regression that fails for
the old behavior. Contract changes require both serialization and replay or
integration coverage.

## Fixture Discipline

- Fixtures represent a documented external or persisted contract, not arbitrary
  test data.
- Synthetic permission events must be labeled synthetic; do not claim they
  prove live runtime permission fidelity.
- Keep fixture IDs deterministic and include correlation/evidence references
  once the C1 envelope lands.
- Test vendor cleanliness at the normalized boundary.

## Review Checklist

- Dependency direction still follows `architecture-and-boundaries.md`.
- Vendor fields are decoded once in adapters.
- New event/state fields survive persistence and replay.
- State changes reference Chronicle evidence.
- Permission decisions remain outside relationship-state inference.
- No evidence write error is swallowed.
- No presentation copy enters runtime prompts.
- Formatting, check, clippy, and tests have honest recorded outcomes.


## Review-work regression lanes

A final C1 check includes semantic-negative tests, durable attribution/value
validation, bounded-reader consumption, SQLite replay validation, historical
state selection, derived FTS freshness, legacy conflict visibility, metadata
sentinels, and the adapter-to-Chronicle-to-Theater chain. Passing only the
positive GNU demo is insufficient.