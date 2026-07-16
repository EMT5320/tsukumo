# Tsukumo V0 Scope Convergence — 2026-07-11

> 2026-07-16 superseding priority: V0 is now a portfolio-first creative-tool
> release. Market-moat validation remains optional evidence, not a release
> gate. The hard gate is a reproducible, technically defensible, memorable
> Tsukumo demo by July 23. `DESIGN.md` is authoritative.

## Decision

The owner approved a functional V0 (`v0.1.0`) focused on real cross-runtime
continuity, a minimum controllable TUI, and reproducible release packaging.
The implementation sequence is deliberately slower and independently
verifiable at each boundary.

## V0 Contract

V0 keeps these load-bearing guarantees:

1. A versioned checkpoint carries task progress, open loops, stable StateRefs,
   and source EventIds across runtime switches.
2. Every runtime projection has an immutable production receipt committed
   before process launch. The receipt stores versions, selected refs, SHA-256
   digests, lengths, budget units, omissions, and redaction metadata; it stores
   no rendered prompt text.
3. Claude and Codex fixtures share normalized semantics, and an opt-in live
   path uses the same decoders and host lifecycle.
4. A deterministic with-state/without-state comparison keeps all controlled
   inputs equal except the target StateId and records the resulting normalized
   tool-argument difference.
5. The stage-first TUI can explain remembered state and projection inputs,
   revoke state, make explicit permission decisions, and render the approved
   Midnight Ninety-Nine Workshop through the bundled `default-shiori` pack or
   one validated external presentation-pack directory.
6. A clean, documented, installable build passes pinned Linux and Windows GNU
   quality gates.

## Deferred to V0.1

V0.1 owns the reusable debug/evaluation snapshot product lifecycle:

- persistent redacted prompt snapshots;
- seven-day default expiry;
- explicit long-term retain decisions;
- cleanup audit records and snapshot-management UI;
- a general artifact repository for arbitrary evaluation bundles.

V0 may create redacted, deterministic comparison fixtures in test/evidence
contexts. They are reviewed build artifacts or temporary test outputs, not a
new durable product authority. Production mode remains receipt-only.

## Ordered Task Tree

1. `07-10-c1-contracts-chronicle` — complete.
2. `07-10-c1-handoff-projection` — complete: checkpoint, selection, renderer,
   receipt, and deterministic comparison seam.
3. `07-10-c1-host-runtime` — composition root, Claude process lifecycle, Safety
   Plane.
4. `07-10-c1-cross-runtime-evidence` — Codex adapter and controlled
   cross-runtime evidence.
5. `07-11-v0-mvp-tui` — stage-first interactive product surface, default
   Shiori workshop, and minimal presentation-pack boundary.
6. `07-11-v0-release-packaging` — in progress: installability, README, license,
   committed lockfile, pinned toolchain, Linux / Windows GNU CI, and release
   verification.
7. `07-16-v0-reentry-inspect` — complete: a read-only CLI report compares the
   reviewed Git HEAD, current artifact state, progress claims, open loops, and
   prompt-free runtime identity without writing Chronicle state.

## Portfolio Exit Gate — 2026-07-16

The July 23 exit gate requires:

1. one reproducible local build and offline non-live test pass;
2. one opt-in Claude / Codex demonstration whose claims stay inside retained
   evidence;
3. the Midnight Ninety-Nine Workshop as the memorable product surface;
4. one bounded re-entry report that distinguishes current, completed, drifted,
   blocked, and unknown state without silently projecting stale claims;
5. README, architecture explanation, demo script, license, lockfile, pinned
   toolchain, and CI configuration.

The gate does not require market uniqueness, statistical product-utility
evidence, a second frontend, long-term memory curation, or feature parity with
configuration managers such as CC Switch.

## Scope Guard

The approved default workshop, Shiori five-state identity, and minimal external
presentation-pack loader belong to V0 gate 5. Additional characters, worlds,
authoring tools, hot reload, skill evolution, idle ecology, and full evaluation
infrastructure cannot displace the six V0 gates above. New scope requires an
explicit task and owner decision.
