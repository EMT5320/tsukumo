# C1 closeout checkpoint (2026-07-11)

## Current state

- Active task: `.trellis/tasks/07-10-c1-contracts-chronicle`
- Status: `in_progress`
- Five-lane review: goal PASS, code PASS, security PASS, QA PASS, context PASS.
- No commit, push, or archive has been performed.

## Delivered contracts

- Frozen kernel event envelopes, attribution, redaction, bounded JSONL readers,
  and distinct permission/safety/degraded outcomes.
- SQLite Chronicle replay with contract revalidation and append-only behavior.
- Deterministic StateWriter with evidence causation, immutable evidence refs,
  historical intervals, conflict handling, TTL, and backdated-write rejection.
- Schema-1 migration reconstructs inactive intervals, rejects unprovable rows,
  and invalidates old legacy completion markers.
- GNU explicit-state formation uses normalized sentence allowlists; negative,
  opposed, hedged, question, one-off, and substring-lookalike text is rejected.
- Legacy import is bounded and idempotent. Sensitive pre-C1 text stays available
  for explicit review while recall, snapshots, and briefing projections omit it.
- Adapter fixture envelopes persist, reopen, replay, and continue through Theater.
- Canonical exports and FTS rebuild from SQLite.

## Verification evidence

- `cargo +stable-x86_64-pc-windows-gnu fmt --all -- --check`: PASS
- `cargo +stable-x86_64-pc-windows-gnu check --workspace --all-targets --offline`: PASS
- `cargo +stable-x86_64-pc-windows-gnu clippy --workspace --all-targets --offline -- -D warnings`: PASS
- `cargo +stable-x86_64-pc-windows-gnu test --workspace --offline`: 94/94 PASS
- `git -c safe.directory=D:/WorkSpace/tsukumo diff --check`: PASS
  (Git emitted line-ending notices only).
- `python ./.trellis/scripts/task.py validate 07-10-c1-contracts-chronicle`: PASS
- Pure Rust LOC ceiling: PASS; maximum is 247, with zero files above 250.
- Manual C1 demo: first run `created`, second run `unchanged`;
  Chronicle remained two ordered events and lifecycle causation pointed to the
  source user event.
- Temporary demo and QA directories were removed.

## Non-blocking follow-ups

- Apply the shared secret detector to legacy `id` and `session_id` metadata.
- Add total-stream/event budgets beyond the existing per-line limits.
- Harden export/snapshot destinations against symlink or reparse-point targets.
- Add direct coverage for the migration missing-evidence rollback branch.

## Next step

Phase 3.4 was explicitly approved on 2026-07-11. Land the two planned work
commits, then run Trellis finish-work for archive and journal bookkeeping. Do not
push.
