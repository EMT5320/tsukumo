# Tsukumo V0 Scope Convergence — 2026-07-11

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
5. The TUI can explain remembered state and projection inputs, revoke state,
   and make explicit permission decisions.
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
5. `07-11-v0-mvp-tui` — minimum interactive product surface.
6. `07-11-v0-release-packaging` — installability, README, license, lockfile,
   toolchain, CI, and release verification.

## Scope Guard

Presentation polish, character expansion, skill evolution, idle ecology, and
full evaluation infrastructure cannot displace the six V0 gates above. New
scope requires an explicit task and owner decision.
