# C1 Handoff and Projection — Technical Design

## Scope

This child consumes the first child's Chronicle and StateRepository. It creates
versioned handoff checkpoints, deterministic projection selection/rendering,
immutable receipts, and a runtime-free CaseBundle seam. It does not spawn a
vendor process.

## HandoffCheckpoint

```rust
pub struct HandoffCheckpoint {
    pub id: CheckpointId,
    pub quest_id: QuestId,
    pub version: u64,
    pub previous_id: Option<CheckpointId>,
    pub goal: String,
    pub progress: Vec<ProgressItem>,
    pub decisions: Vec<Decision>,
    pub constraint_refs: Vec<StateRef>,
    pub artifacts: Vec<ArtifactRef>,
    pub open_loops: Vec<OpenLoop>,
    pub next_actions: Vec<NextAction>,
    pub source_event_refs: Vec<EventId>,
    pub created_at: Timestamp,
}
```

Each new version supplies a transition for every previous open loop:
`Inherited`, `Completed`, `Abandoned`, or `ReplacedBy`. Compilation rejects a
silent disappearance. Hard constraints remain stable `StateRef`s and resolve
to current display text only while rendering.

Checkpoint generation is triggered at runtime switch, imminent context
compression, pause/interruption, milestone, completion, or explicit user
request. It does not call an LLM after every tool event. C1 starts with a
deterministic compiler input contract; a future summarizer can propose a draft
behind the same validation gate.

## State Selection

Selection is pure for a fixed repository snapshot and request:

1. include checkpoint-pinned hard constraints first;
2. reject revoked, expired, unresolved, or scope-inapplicable state;
3. rank remaining candidates by task applicability, strength, freshness, then
   stable key/ID tie-breakers;
4. admit candidates while the declared budget allows;
5. record every considered omission with a deterministic reason.

C1's default budget is Unicode scalar/character based because no model-neutral
tokenizer exists yet. Receipts record `BudgetUnit::Characters`, plus exact byte
and character counts. A future token budget must name its tokenizer and create
a new projection/renderer version.

## Projection Renderer

The canonical section order is versioned and frozen by golden tests:

```text
Tsukumo handoff header / precedence
Goal
Current progress
Decisions
Constraints (StateRef + resolved text)
Artifacts
Open loops
Next actions
User delegation goal
```

The renderer normalizes UTF-8 line endings to LF and emits one final newline.
It returns a `SensitiveText` prompt plus a receipt draft; `Debug`/`Display`
never reveal the prompt. Theater persona, bubbles, and relationship flavor text
are excluded.

SHA-256 covers the exact bytes later sent to runtime stdin. Each named section
has its own digest/byte/character count. The receipt also records selected refs,
checkpoint/runtime/execution, projection and renderer versions, budget,
omission reasons, and redaction metadata.

## Persistence Ordering

```text
checkpoint + repository snapshot
  -> selection + render in memory
  -> validate all refs and digests
  -> save immutable receipt + state-ref edges in SQLite
  -> return PreparedProjection
```

Only `PreparedProjection` may be passed to the future host. A receipt failure
returns an error and no launchable value. Historical receipts never change when
a state is revoked or superseded.

## Debug/Eval Snapshot and CaseBundle

Production mode stores no rendered text. Explicit debug/eval mode:

1. applies a named deterministic redaction profile in memory;
2. verifies sentinel secrets are absent;
3. stores the redacted canonical snapshot as a separate CaseBundle artifact;
4. hashes the redacted bytes independently;
5. records expiring retention (seven days by default) or explicit retain.

Cleanup deletes expired snapshot bytes while retaining artifact identity,
digest, redaction/expiry metadata, and an audit result. Reviewed committed
fixtures are synthetic/recorded assets and use a separate provenance marker.

`CaseBundle` contains fixed inputs, state snapshot, checkpoint, receipt,
expected runtime events/outcome, and comparison metadata. Its synthetic
with-state/without-state pair differs only in the target state inclusion and
dependent projection digests.

## Compatibility

The existing `BriefCompiler`/`assemble_delegation_prompt` remain temporary A1
facades or are migrated to use the new selector/renderer. They must not keep an
independent selection algorithm. The old `inject_trace.jsonl` is replaced by
receipt/Chronicle evidence rather than expanded into another authority.

## Test Strategy

- Checkpoint version and complete open-loop transition tests.
- Scope/revoke/expiry/ranking/budget selection table tests with stable ties.
- Canonical renderer golden and SHA-256 mutation tests.
- Receipt SQLite round-trip, immutability, selected-ref foreign keys, and
  storage-before-launch type/API test.
- Sentinel-secret tests across receipt serialization, SQL, Chronicle, errors,
  and debug formatting.
- Redaction-before-write, seven-day cleanup, explicit retain, and controlled
  clock tests.
- Synthetic removed-state CaseBundle invariant comparison.

## Rollback

Checkpoint and receipt tables are append-only. A renderer regression is rolled
back by code while keeping historical versioned receipts readable. Never
rewrite old digests to match a new renderer.
