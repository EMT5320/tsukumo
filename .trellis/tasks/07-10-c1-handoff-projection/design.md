# C1 Handoff and Projection — Technical Design

## Scope

This child consumes the first child's Chronicle and StateRepository. It creates
versioned handoff checkpoints, deterministic projection selection/rendering,
immutable receipts, and a runtime-free comparison seam. It does not spawn a
vendor process or persist debug prompt snapshots.

## HandoffCheckpoint

```rust
pub struct HandoffCheckpoint {
    pub id: CheckpointId,
    pub quest_id: QuestId,
    pub version: u64,
    pub previous_id: Option<CheckpointId>,
    pub goal: PersistedText,
    pub progress: Vec<ProgressItem>,
    pub decisions: Vec<Decision>,
    pub constraint_refs: Vec<StateRef>,
    pub artifacts: Vec<ArtifactReference>,
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

## Deterministic Comparison Seam

V0 production mode stores no rendered text and adds no debug snapshot table or
artifact lifecycle. A pure comparison helper prepares two projections from the
same repository snapshot and request. The without-state request excludes one
target StateId; all other checkpoint, runtime, execution-independent config,
renderer version, budget, and non-target state inputs remain equal.

The helper reports selected-ref and digest differences plus an invariant
manifest. Tests use temporary values or reviewed redacted fixtures. Persistent
redacted prompt snapshots, seven-day expiry, explicit retain, cleanup audit,
and general evaluation artifact storage belong to V0.1.

## Compatibility

The existing `BriefCompiler`, `assemble_delegation_prompt`, and
`assemble_with_trace` remain A1 fixture/legacy compatibility surfaces. Their
public documentation deprecates them for production launches and states that
legacy size telemetry carries no projection-evidence claim. New hosts launch
only `SoulStore::prepare_projection` output; receipt/Chronicle evidence owns the
production claim.

## Test Strategy

- Checkpoint version and complete open-loop transition tests.
- Scope/revoke/expiry/ranking/budget selection table tests with stable ties.
- Canonical renderer golden and SHA-256 mutation tests.
- Receipt SQLite round-trip, parent/edge-table immutability triggers,
  selected-ref foreign keys, and storage-before-launch type/API test.
- Receipt prompt-sentinel tests across serialization, SQL, Chronicle, errors,
  logs, and debug formatting.
- Synthetic removed-state comparison and invariant-manifest tests without
  prompt snapshot persistence.

## Rollback

Checkpoint and receipt tables are append-only. A renderer regression is rolled
back by code while keeping historical versioned receipts readable. Never
rewrite old digests to match a new renderer.
