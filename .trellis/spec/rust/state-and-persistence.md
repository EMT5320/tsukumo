# State and Persistence

## Three Ledgers

The C1 design defines three distinct records:

1. **Chronicle**: append-only facts about what happened.
2. **Canonical State**: versioned `StateRecord` values currently accepted by
   Tsukumo.
3. **Handoff/Projection Ledger**: checkpoints and receipts describing what a
   runtime actually received.

Do not collapse these into a single free-form memory table. The intended flow
is documented in
`docs/tsukumo-vision-state-handoff-convergence-2026-07-10.md` §5.

## Evidence-Backed State

Every durable state record must have:

- stable `StateKey` and typed `StateId`;
- kind (`Preference`, `Fact`, `Constraint`, `Procedure`, or `Milestone`);
- subject plus applicability scope;
- evidence strength;
- active/superseded/revoked lifecycle;
- one or more Chronicle `EventId` references;
- timestamps and optional expiry for temporary facts.

Same key plus same scope is an update/conflict decision, not an unrelated new
fact. Conflicting values must not silently overwrite history. Revocation and
supersession remain reconstructable.

`MemoryFact` and `SoulStore` in `crates/tsukumo-soul/src/store.rs` are an A1/R
probe, not the final StateRecord schema. Extend or migrate them through an
explicit C1 design; do not bolt scope/status fields onto Markdown lines in
multiple call sites.

## Storage Authority During C1 Migration

The current probe writes SQLite first and rewrites `MEMORY.md` / `USER.md` as
human-readable snapshots. Its module comment calls the files the source of
truth, but there is no import path from edited snapshots. Until C1 explicitly
settles storage authority:

- add no second independent writer;
- treat snapshot files as generated/exported views in new code;
- keep Chronicle append-only;
- make versioned canonical records queryable and referenceable;
- document migration/rebuild behavior before changing the on-disk schema.

## Handoff and Projection

- A checkpoint is task state, not a bag of recalled facts.
- Hard constraints are carried by stable `StateRef`, not repeatedly rewritten
  prose.
- Every open loop is inherited, completed, abandoned, or explicitly replaced
  in the next checkpoint version.
- Every runtime projection produces a receipt containing checkpoint,
  selected-state refs, runtime/execution, renderer version or hash, and an
  honest budget unit.
- Store enough information for a representative removed-state comparison,
  while avoiding raw secret-bearing prompt dumps by default.

## Safety Separation

Permission requests and user decisions belong to a deterministic Safety Plane.
They may be Chronicle evidence, but repeated approval must never create an
auto-approve relationship state. Models may request permission and may not
approve it.

## Database and FTS Rules

- Keep SQL inside the owning storage module.
- Parameterize values; never interpolate user text into SQL.
- Sanitize/quote FTS query syntax before `MATCH`. The current helper accepts
  hyphen/reserved tokens that can produce FTS5 syntax errors; carry a regression
  when replacing it.
- Use transactions when a logical write changes multiple durable structures.
- Do not claim a write succeeded if its evidence/receipt append failed.

