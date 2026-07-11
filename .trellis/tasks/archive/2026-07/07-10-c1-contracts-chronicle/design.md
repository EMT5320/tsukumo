# C1 Contracts and Chronicle — Technical Design

## Scope

This child establishes the durable vocabulary and storage substrate consumed by
all later C1 children. It intentionally stops before checkpoint projection or
live process ownership.

## Kernel Contract Migration

Move existing variants from the current payload-only `KernelEvent` enum into a
`KernelEventPayload` enum and introduce an envelope equivalent to:

```rust
pub struct KernelEvent {
    pub schema_version: u16,
    pub event_id: EventId,
    pub occurred_at: Timestamp,
    pub quest_id: QuestId,
    pub session_id: SessionId,
    pub spirit_id: SpiritId,
    pub execution_id: Option<ExecutionId>,
    pub runtime: Option<RuntimeBinding>,
    pub causation_id: Option<EventId>,
    pub correlation_id: Option<CorrelationId>,
    pub payload: KernelEventPayload,
}
```

Add opaque newtypes for event, quest, session, spirit, execution, state,
checkpoint, projection, correlation, and artifact identities. A shared
`SensitiveText` wrapper has redacted `Debug`/`Display`, no implicit
`Serialize`/`Deserialize`, and an explicit exposure method for narrow validated
storage or host/runtime boundaries.

The payload contract expands beyond the existing tool/permission/end/error
variants to cover user input, runtime lifecycle/switch, permission decisions,
state lifecycle, checkpoint/projection creation, and outcome. Vendor IDs remain
namespaced provenance inside normalized payloads; the host/Chronicle writer
assigns global IDs and sequence.

The change is coordinated across all existing JSONL fixtures, session helpers,
adapters, Director matches, examples, and tests. There is no long-lived legacy
`KernelEvent` alias.

## Storage Modules

Split the current probe-sized `store.rs` rather than growing it into one mixed
module:

```text
tsukumo-soul/src/
  storage.rs       connection, migrations, transaction/unit-of-work
  chronicle.rs     append/query/replay
  state.rs         StateRecord, repository, lifecycle
  extract.rs       StateExtractor and StateDraft contracts
  export.rs        JSONL/Markdown/FTS derived views
  store.rs         temporary compatibility facade for A1 APIs
```

Minimum SQLite roles follow the Rust persistence spec:

```text
schema_migrations
chronicle_events
state_records
state_evidence
```

Child 2 adds `handoff_checkpoints`, `checkpoint_state_refs`,
`projection_receipts`, and `receipt_state_refs` through the next ordered
migration. Their names/contracts are reserved now, but this child does not
create unused downstream tables.

The connection enables foreign keys and uses explicit transactions. Chronicle
sequence is database-assigned and unique; `event_id` is globally unique.
Duplicate identical events return an idempotent outcome, while a conflicting
duplicate rolls back.

## State Contract

```rust
pub struct StateDraft {
    pub proposed_key: StateKey,
    pub kind: StateKind,
    pub scope: Applicability,
    pub content: SensitiveText,
    pub claimed_strength: EvidenceStrength,
    pub evidence_refs: Vec<EventId>,
    pub provenance: ExtractionProvenance,
    pub expires_at: Option<Timestamp>,
}

pub enum StateTransition {
    Create(StateDraft),
    Supersede { prior: StateId, draft: StateDraft },
    Revoke { prior: StateId, evidence: EventId, reason: String },
}
```

`StateWriter` resolves referenced events inside the same transaction and
validates key/scope, secret policy, lifecycle conflicts, expiry, and allowed
kind/strength combinations. One inferred observation cannot become an explicit
hard constraint. Same key/scope conflicts are explicit outcomes, never
`INSERT OR REPLACE`.

Rules and the optional structured LLM implement `StateExtractor` without a
database handle. Extraction errors produce observable skip/failure events and
do not block the primary user task.

## Legacy Migration

Migration is additive and idempotent:

1. Create `schema_migrations` and C1 tables without deleting `facts` or FTS.
2. For every legacy row selected for import, create a deterministic
   `legacy_imported` event and a low-strength state linked to it.
3. Never infer workspace/OS scope, Explicit strength, or Constraint kind from a
   legacy text line.
4. Mark completion with an ordered migration version so reopen/retry does not
   duplicate events or states.
5. Keep old snapshot files as derived output; new writes use C1 repositories.

## Transaction Flows

```text
explicit user input
  -> append user event
  -> deterministic extractor draft
  -> StateWriter validation
  -> state row + evidence edge + state-created event
  -> commit
  -> refresh derived exports
```

Export failure after commit returns a stale-export/degraded result and can be
repaired by rebuild. A Chronicle/state failure rolls back the logical write.

## Test Strategy

- Newtype/serde round trips and sensitive debug redaction.
- Envelope schema round trip and coordinated fixture replay.
- Append order, duplicate idempotency/conflict, reopen, and query filters.
- State create/conflict/supersede/revoke/expiry and evidence validation.
- Rule extractor plus recorded structured-LLM success/malformed/timeout cases.
- Transaction rollback and swallowed-error regression for the current trace
  pattern.
- Legacy migration idempotency and no hard-constraint escalation.
- Delete/rebuild derived JSONL/Markdown/FTS and prove manual export edits do not
  mutate canonical rows.

## Rollback

All schema changes are additive. Rolling back code leaves legacy and new tables
intact; no down migration deletes user data. If the new envelope migration is
reverted during development, fixtures and consumers revert in the same commit
to avoid mixed wire contracts.
