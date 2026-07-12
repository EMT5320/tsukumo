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

## Scenario: SQLite Is the Durable Authority

### 1. Scope / Trigger

This contract applies to Chronicle events, canonical StateRecords,
HandoffCheckpoints, ProjectionReceipts, and their evidence links. C1 uses one
SQLite database as the only durable authority. JSONL and `MEMORY.md` / `USER.md`
are rebuildable exports, not independent writable sources.

This decision prevents a crash from committing a state row while losing the
event or receipt that justifies it. The current A1/R probe writes SQLite first
and rewrites Markdown snapshots but incorrectly describes those snapshots as
the source of truth; C1 must remove that ambiguity.

### 2. Signatures

The storage boundary should expose transaction-aware operations equivalent to:

```rust
trait ChronicleStore {
    fn append(&mut self, event: &KernelEvent) -> Result<ChronicleSeq, StoreError>;
    fn replay(&self, query: ChronicleQuery) -> Result<Vec<PersistedEvent>, StoreError>;
}

trait StateRepository {
    fn apply(&mut self, transition: StateTransition) -> Result<StateRecord, StoreError>;
}

trait HandoffLedger {
    fn save_checkpoint(&mut self, checkpoint: &HandoffCheckpoint) -> Result<(), StoreError>;
    fn save_receipt(&mut self, receipt: &ProjectionReceipt) -> Result<(), StoreError>;
}
```

These interfaces may share a concrete SQLite unit-of-work so a logical
operation can append evidence and update its versioned projection in one
transaction.

Minimum table roles:

```text
chronicle_events      append-only envelope + payload, unique event_id/sequence
state_records         immutable/versioned state rows and lifecycle status
state_evidence        StateId -> EventId references
handoff_checkpoints   immutable quest/checkpoint versions
checkpoint_state_refs CheckpointId -> StateId references
projection_receipts   immutable execution projection records
receipt_state_refs    ProjectionId -> StateId references
schema_migrations     ordered storage schema version
```

### 3. Contracts

- `chronicle_events` permits insert and replay; product code never updates or
  deletes rows.
- State supersession/revocation creates a new lifecycle transition and keeps
  the prior version explainable.
- Evidence references must resolve inside the same database before commit.
- Checkpoints and receipts are immutable once used by an execution.
- JSONL/Markdown exporters read committed SQLite state and can be rerun at any
  time; edits to export files are never imported implicitly.
- FTS tables and human-readable files are derived indexes/views and may be
  rebuilt without changing durable IDs.
- Secrets are excluded/redacted before persistence according to the owning
  payload contract; exports do not weaken that boundary.
- Migration from the A1/R `facts` table is additive and idempotent. Imported
  rows receive deterministic `legacy_imported` Chronicle evidence and at most a
  low-strength state; migration never fabricates `Explicit` strength,
  Constraint kind, or a narrower scope from free-form legacy text.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Duplicate `event_id` with identical content | Idempotent duplicate result or explicit `DuplicateEvent`; no second row |
| Duplicate `event_id` with different content | Integrity error; rollback |
| State references missing Chronicle evidence | Validation error; rollback event/state unit of work |
| Attempt to mutate/delete Chronicle history | Unsupported/integrity error |
| Checkpoint references revoked/inapplicable state | Selection/validation error before receipt commit |
| SQLite commit fails | No partial state, checkpoint, or receipt is reported successful |
| Export write fails after commit | Durable operation remains committed; surface stale-export status and allow rebuild |
| Unknown/newer schema version | Migration/version error; do not open read-write |
| Legacy import reruns | Idempotent no-op for already imported rows; no duplicate event/state |
| Legacy row would require guessed hard scope/strength | Import as low-strength/unscoped reviewable state or skip with reason; never upgrade silently |

### 5. Good / Base / Bad Cases

- **Good**: one transaction appends the explicit user event, creates the GNU
  constraint and evidence link, commits, then refreshes derived exports.
- **Base**: a read-only recall queries SQLite/FTS and produces no durable write.
- **Bad**: append JSONL, write SQLite state separately, rewrite Markdown, and
  report success after only two of the three writes complete.

### 6. Tests Required

- Transaction rollback test: forced state/evidence failure leaves neither the
  transition nor a misleading partial projection.
- Append-only test: public APIs cannot rewrite a persisted Chronicle event.
- Idempotency test for duplicate event IDs.
- Reopen/replay test preserving event order, IDs, and evidence refs.
- Supersede/revoke test retaining old versions and historical receipts.
- Export rebuild test: delete derived JSONL/Markdown, regenerate it from SQLite,
  and compare normalized content.
- Migration/reopen test from the A1/R `facts` schema proving deterministic IDs,
  idempotency, preserved text, and no Explicit/Constraint escalation.

### 7. Wrong vs Correct

#### Wrong

```text
Chronicle JSONL (writer A) + SQLite state (writer B) + Markdown state (writer C)
```

Three authorities require cross-file crash recovery and can disagree about
which history is real.

#### Correct

```text
SQLite transaction
  -> append Chronicle evidence
  -> write versioned canonical/ledger rows
commit
  -> rebuildable JSONL/Markdown/FTS projections
```

Only committed SQLite rows define durable truth; exports remain transparent
and portable without becoming another write authority.

## Scenario: Guarded Windows Rollback-Journal Recovery

### 1. Scope / Trigger

This contract applies when Host opens `soul.db` on Windows while
`LocalDirectoryGuard` holds single-link, no-follow, no-delete-share handles for
`soul.db` and its rollback sidecars. It also applies to databases created by an
older build that may leave a DELETE-mode hot journal after process termination.

### 2. Signatures

The public boundary remains:

```rust
pub fn SoulStore::open(data_dir: impl AsRef<Path>) -> Result<SoulStore, SoulError>;
```

Before migration or any other schema-loading statement, the connection executes
this exact protocol on `main`:

```sql
PRAGMA main.locking_mode = EXCLUSIVE;
PRAGMA main.journal_mode = PERSIST;
PRAGMA main.locking_mode = NORMAL;
SELECT COUNT(*) FROM main.sqlite_schema;
```

### 3. Contracts

- The main database opens with `SQLITE_OPEN_NOFOLLOW`.
- Host validates and guards `soul.db`, `soul.db-journal`, `soul.db-wal`, and
  `soul.db-shm` before SQLite opens.
- EXCLUSIVE locking is set before the first statement that loads schema. A
  legacy hot journal is rolled back and finalized by zeroing its header, so the
  no-delete-share guard stays intact.
- `journal_mode` must return `persist`; `locking_mode` must return `exclusive`
  and then `normal` at their respective steps.
- The final schema read is a release barrier. Its statement must finish before
  migration starts so another correctly configured connection can access the
  database while the first connection remains alive.
- Any failed step closes the connection through RAII and surfaces a typed
  `SoulError`; Host never weakens the file guard as a recovery fallback.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Valid DELETE-mode hot journal | Roll back interrupted pages, preserve committed data, invalidate journal header, then open in PERSIST mode |
| Empty or already-cold persistent journal | Open normally through the same protocol |
| Locking or journal PRAGMA returns an unexpected mode | `SoulError::InvalidStoredValue`; no migration |
| Corrupt database or unrecoverable journal | Typed SQLite error; guard and connection handles release through RAII |
| Reparse point, hard link, UNC path, or device alias | Host `LocalPath` rejection before SQLite |
| Second connection uses PERSIST while the controller is live | Read and write succeed after the release barrier |

### 5. Good / Base / Bad Cases

- **Good**: a killed DELETE-mode writer leaves a hot journal; guarded Host open
  restores the committed value, zeroes the hot header, returns to NORMAL
  locking, and permits a second PERSIST connection.
- **Base**: a clean PERSIST database follows the same sequence without recovery.
- **Bad**: guard the journal without delete sharing, run
  `journal_mode=PERSIST` as the first schema-loading PRAGMA, and receive
  `SQLITE_IOERR_DELETE` while recovery tries to delete the journal.

### 6. Tests Required

- Windows integration test: create a committed value, kill a child during an
  IMMEDIATE transaction after `cache_flush`, verify a valid hot-journal header,
  and open through `HostProductController`.
- Assert the interrupted value rolls back, the committed value survives, and
  the journal header is no longer hot.
- Keep the first controller alive while a second PERSIST connection reads and
  writes the database.
- Retain hard-link/reparse startup-race tests and runtime sidecar replacement
  tests, proving recovery never relaxes the path capability.

### 7. Wrong vs Correct

#### Wrong

```text
open guarded files -> schema-loading journal_mode=PERSIST
                   -> DELETE recovery tries to delete guarded journal
                   -> SQLITE_IOERR_DELETE (2570)
```

#### Correct

```text
open guarded files -> locking_mode=EXCLUSIVE (no schema load)
                   -> journal_mode=PERSIST triggers in-place hot recovery
                   -> locking_mode=NORMAL
                   -> schema read release barrier
                   -> migrations and normal product operation
```
## Scenario: Hybrid State Extraction With a Deterministic Write Gate

### 1. Scope / Trigger

This contract applies whenever a user message, repeated behavior, runtime
outcome, or rule may become durable relationship state. C1 uses a hybrid
extractor: deterministic rules for structured/explicit signals and a structured
LLM extractor for free-form natural language. Neither path may write canonical
state directly.

### 2. Signatures

```rust
trait StateExtractor {
    fn extract(&self, input: &ExtractionContext) -> Result<Vec<StateDraft>, ExtractError>;
}

struct StateDraft {
    proposed_key: StateKey,
    proposed_kind: StateKind,
    proposed_scope: Scope,
    content: String,
    claimed_strength: EvidenceStrength,
    evidence_refs: Vec<EventId>,
    provenance: ExtractionProvenance,
}

trait StateWriter {
    fn apply(&mut self, draft: StateDraft) -> Result<StateWriteOutcome, StateWriteError>;
}
```

`ExtractionProvenance` records rule/extractor version and, for LLM extraction,
the provider-neutral model/config identity needed for debugging. It is not the
state's evidence; `evidence_refs` must point to Chronicle events.

### 3. Contracts

- Deterministic extractors handle structured commands/events and rules whose
  behavior can be fully tested.
- The LLM extractor returns schema-validated `StateDraft` values only. It has no
  database handle and cannot approve its own proposal.
- The StateWriter independently validates evidence existence, scope, sensitive
  content, lifecycle conflict, TTL, and allowed kind/strength combinations.
- A single inferred observation cannot produce an `Explicit` strength or hard
  `Constraint`.
- Extraction failure never blocks the user's primary task. Record an observable
  failure/skip event and continue without a new durable state.
- C1 regression tests use deterministic or recorded extractor output. At least
  one opt-in live smoke sends a free-form explicit constraint through the real
  structured LLM path.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Draft has no Chronicle evidence | Reject with `MissingEvidence` |
| LLM output fails schema validation | `ExtractError`; write nothing |
| Inferred draft proposes hard Constraint | Reject or deterministically downgrade according to an explicit rule; never accept silently |
| Draft contains credential/secret material | Reject/redact before canonical persistence and record reason |
| Scope is missing or cannot be resolved | Reject with `InvalidScope` |
| Same key/scope has compatible active state | Return explicit no-op/update outcome according to content/version rules |
| Same key/scope conflicts | Return conflict; do not overwrite |
| LLM timeout/unavailable | Continue task, record extraction skipped/failed, write no state |

### 5. Good / Base / Bad Cases

- **Good**: free-form “this project always uses GNU on Windows” becomes a
  schema-valid draft; StateWriter verifies the user event and workspace/OS
  scope before committing an Explicit Constraint.
- **Base**: a message contains no durable information; extractor returns an
  empty list and no state event is written.
- **Bad**: give the LLM a `SoulStore` handle and accept whatever memory text it
  inserts.

### 6. Tests Required

- Deterministic extractor tests for explicit structured signals and no-op input.
- Schema rejection tests for malformed recorded LLM output.
- Writer tests for missing evidence, invalid scope, secret content, inferred
  hard constraints, duplicate, conflict, supersede, and revoke.
- Property/fixture test proving every accepted state references existing
  Chronicle events.
- Integration test with recorded draft output, independent of network/model.
- Opt-in live smoke for one free-form explicit constraint; do not make the
  default CI quality gate depend on credentials.

### 7. Wrong vs Correct

#### Wrong

```text
user message -> LLM -> INSERT state_records
```

#### Correct

```text
user/structured event -> Chronicle
                      -> rule or LLM StateExtractor -> StateDraft
                      -> deterministic StateWriter validation
                      -> transactional StateRecord + evidence link
```

The extractor proposes meaning; deterministic code owns durable authority.

## Scenario: Versioned Handoff Checkpoints

### 1. Scope / Trigger

This contract applies when a runtime switch, context compression, pause,
milestone, completion, or explicit user request requires durable task state.
Checkpoint creation is low-frequency and always references existing Chronicle
and canonical-state evidence.

### 2. Signatures

```rust
struct HandoffCheckpoint {
    id: CheckpointId,
    quest_id: QuestId,
    version: u64,
    previous_id: Option<CheckpointId>,
    goal: PersistedText,
    progress: Vec<ProgressItem>,
    decisions: Vec<Decision>,
    constraint_refs: Vec<StateRef>,
    artifacts: Vec<ArtifactReference>,
    open_loops: Vec<OpenLoop>,
    open_loop_transitions: Vec<OpenLoopTransition>,
    next_actions: Vec<NextAction>,
    source_event_refs: Vec<EventId>,
    created_at: Timestamp,
    trigger: CheckpointTrigger,
}

struct CheckpointWriteRequest {
    checkpoint: HandoffCheckpoint,
    event: KernelEvent, // CheckpointCreated with matching ID/version/time/quest
}
```

Schema migration 3 owns these tables and append-only edges:

```text
handoff_checkpoints(checkpoint_id PK, quest_id, version,
  previous_checkpoint_id FK, created_at, created_event_id FK UNIQUE,
  checkpoint_json, UNIQUE(quest_id, version))
checkpoint_state_refs(checkpoint_id FK, state_id FK, state_version, position,
  PRIMARY KEY(checkpoint_id, state_id))
checkpoint_source_refs(checkpoint_id FK, event_id FK, position,
  PRIMARY KEY(checkpoint_id, event_id))
```

### 3. Contracts

- `SoulStore::save_checkpoint` validates the complete checkpoint before opening
  the write transaction.
- Version 1 has no previous ID. Each later version names the immediately prior
  checkpoint for the same quest and increments its version by one.
- Every `source_event_ref` exists in Chronicle. Every constraint `StateRef`
  resolves to the exact version and is active at `created_at`.
- Every prior open loop has exactly one `Inherited`, `Completed`, `Abandoned`,
  or `ReplacedBy` transition. Inherited IDs remain open; replacement IDs name a
  new loop; resolved IDs disappear from the new open set.
- The matching `CheckpointCreated` event, checkpoint JSON, StateRef edges, and
  source-event edges commit in one transaction.
- Update/delete triggers protect the checkpoint row and both ordered edge
  tables. Reopening must reconstruct the same ordered value.
- An idempotent retry must reuse the original `created_event_id`, identical
  checkpoint content, and the identical Chronicle envelope. A changed EventId
  or same-ID/different-envelope retry fails closed.
- Checkpoint persisted text passes bounded-field, control-character, and shared
  secret-material validation.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Empty/oversized/control-bearing field or duplicate refs | `InvalidField` |
| Persisted field resembles secret material | `SensitiveField` |
| Creation event identity/version/time/quest differs | `EventMismatch`; no Chronicle append |
| Source event or state is absent | `MissingSourceEvent` / `MissingState` |
| State version differs or state is inactive at capture | `StateVersionMismatch` / `InactiveState` |
| Previous checkpoint is absent, skips a version, or changes quest | `MissingPrevious` / `InvalidVersion` / `QuestMismatch` |
| Prior loop has zero/multiple/invalid transitions | typed open-loop error; no write |
| Existing checkpoint ID changes content or creation EventId | `ConflictingCheckpoint` |
| Original creation EventId carries changed envelope content | `ConflictingEvent` |

### 5. Good / Base / Bad Cases

- **Good**: version 2 inherits one loop, completes another, pins an active
  `StateRef`, and commits all edges with its Chronicle event.
- **Base**: version 1 contains no loops or constraints and still names at least
  one source event.
- **Bad**: omit a previous loop from the next checkpoint or rewrite an ordered
  edge after commit.

### 6. Tests Required

- `c1_checkpoint.rs`: complete loop-transition matrix, reopen equality,
  silent-disappearance rejection, unresolved-StateRef rollback, and retry
  EventId integrity.
- `c1_projection_selection.rs`: a pinned state can become inactive after the
  checkpoint and receives a deterministic projection omission.
- Schema/privacy tests assert update/delete triggers exist for the checkpoint
  row and both edge tables.

### 7. Wrong vs Correct

#### Wrong

```text
latest summary text -> overwrite checkpoint row -> lose prior loop identity
```

#### Correct

```text
prior checkpoint + explicit loop transitions + Chronicle/StateRefs
  -> validate -> atomic event/checkpoint/edge commit -> immutable new version
```
## Scenario: Projection Receipts Without Prompt Retention

### 1. Scope / Trigger

This contract applies whenever Tsukumo renders a checkpoint and selected
relationship state into the exact bytes passed to a runtime. The durable ledger
must prove which inputs and renderer produced the projection without turning
secret-bearing prompts into a second transcript store.

V0 persists metadata and digests only. Deterministic with-state/without-state
evidence uses temporary test values or reviewed redacted fixtures. Persistent
redacted prompt snapshots, seven-day expiry, explicit retain, cleanup audit,
and general artifact storage are deferred to V0.1 by
`docs/tsukumo-v0-scope-convergence-2026-07-11.md`.

### 2. Signatures

The durable contract should be equivalent to:

```rust
struct ProjectionReceipt {
    id: ProjectionId,
    execution_id: ExecutionId,
    checkpoint_id: CheckpointId,
    runtime: RuntimeBinding,
    selected_state_refs: Vec<StateRef>,
    projection_version: ProjectionVersion,
    renderer_version: RendererVersion,
    rendered_digest: ContentDigest,
    rendered_byte_count: usize,
    rendered_char_count: usize,
    sections: Vec<ProjectionSectionDigest>,
    budget: ProjectionBudgetUsage,
    omissions: Vec<ProjectionOmission>,
    redactions: Vec<RedactionRecord>,
    created_at: Timestamp,
}

struct ContentDigest {
    algorithm: DigestAlgorithm, // V0: Sha256
    value: String,              // lowercase hexadecimal
}

struct ProjectionSectionDigest {
    section: ProjectionSection,
    digest: ContentDigest,
    byte_count: usize,
    char_count: usize,
}

struct ProjectionBudgetUsage {
    used: usize,
    limit: usize,
    unit: BudgetUnit,
}

struct PreparedProjection {
    receipt: ProjectionReceipt,
    rendered_prompt: SensitiveText,
}
```

Only `PreparedProjection` crosses from the projection service to the future
host. It can be constructed only after the production receipt and selected-ref
edges commit successfully.

Schema migration 3 also owns:

```text
projection_receipts(projection_id PK, checkpoint_id FK, execution_id,
  runtime_json, projection_version, renderer_version, rendered_digest,
  rendered_byte_count, rendered_char_count, created_at,
  created_event_id FK UNIQUE, receipt_json)
receipt_state_refs(projection_id FK, state_id FK, state_version, position,
  PRIMARY KEY(projection_id, state_id))
```

Both tables have update/delete triggers. `receipt_json` serializes metadata only;
its public type contains no prompt/rendered-text field.

### 3. Contracts

- Hash the exact UTF-8 bytes passed to the runtime after canonical rendering.
  The renderer owns LF normalization and one final newline; tests freeze both.
- Store the digest algorithm with every digest. V0 uses SHA-256 and never relies
  on a language/runtime default hasher.
- Section digests aid diagnosis and never replace checkpoint identity or
  `selected_state_refs`.
- Record budget value and unit. Characters, bytes, and model tokens are not
  interchangeable; token budgets also identify a tokenizer.
- Omission entries identify candidate state and a deterministic reason such as
  scope mismatch, inactivity, comparison exclusion, or budget exhaustion.
  Redaction entries identify location/category/action without copying the
  secret value. A sensitive delegation goal records
  `delegation_goal/sensitive_material/not_persisted`.
- Persist the receipt before runtime spawn. A failed receipt transaction cannot
  return a launchable value.
- `rendered_prompt` stays an in-memory secret at the host/runtime boundary. Its
  `Debug` and `Display` representations redact content.
- Historical receipts remain immutable after state supersession/revocation;
  later projections use current state and create new receipts.
- Receipt idempotency requires identical receipt metadata, the original
  `created_event_id`, and an identical Chronicle envelope. Content-equivalent
  retries under another EventId cannot return a launchable value.
- V0 comparison helpers remove exactly one target StateId from one frozen input
  set and return selected-ref/digest differences plus an invariant manifest.
  They add no durable prompt-snapshot or artifact table.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Selected state/checkpoint reference is missing or inapplicable | Projection validation error; write no receipt and do not spawn |
| Renderer output differs with identical inputs/version | Determinism test failure; do not bless a new digest silently |
| Digest algorithm or renderer version is unknown | Compatibility error; receipt remains inspectable without a reproducibility claim |
| Receipt schema contains rendered text/raw prompt field | Schema/privacy test failure |
| Secret appears in receipt, Chronicle, error, log, or fixture | Redaction/fixture validation failure |
| Receipt or selected-ref transaction fails | Return no `PreparedProjection` |
| Existing projection ID changes receipt/EventId or reuses EventId with changed envelope | `ConflictingReceipt` / `ConflictingEvent`; return no launchable value |
| Budget unit is absent or ambiguous | Validation error |
| With/without comparison changes a non-target controlled input | Invariant failure; comparison is invalid |

### 5. Good / Base / Bad Cases

- **Good**: a GNU constraint is selected; the receipt stores its StateRef,
  checkpoint/runtime/execution IDs, versions, SHA-256 and section digests,
  budget unit, omissions, and no prompt text.
- **Base**: production execution stores only immutable receipt metadata and
  digests. Exact bytes can be verified when the original in-memory inputs are
  independently supplied to the supported renderer; the receipt alone cannot
  reconstruct a non-retained delegation goal.
- **Bad**: serialize a rendered prompt into `projection_receipts`, trace JSONL,
  a panic message, a comparison manifest, or a fixture for convenience.

### 6. Tests Required

- Golden test for canonical section ordering, LF/final-newline behavior, and
  stable SHA-256 with fixed renderer inputs/version.
- Mutation test proving a changed selected state or checkpoint changes the
  relevant section and overall digest while unrelated metadata does not.
- Serialization/schema test proving `ProjectionReceipt` has no rendered-text
  field and rows/logs exclude a sentinel secret.
- Receipt-before-launch API/integration test: forced persistence failure yields
  no `PreparedProjection` and the future fake runtime remains unstarted.
- With-state/without-state comparison test proving every non-target controlled
  input remains equal.
- Historical audit test proving revoke/supersede changes future selection and
  does not rewrite an old receipt.
- Concrete V0 lanes: `c1_projection.rs`, `c1_projection_budget.rs`,
  `c1_projection_selection.rs`, `c1_receipt.rs`, `c1_comparison.rs`, and
  `tsukumo-adapters/tests/c1_state_theater_cross_layer.rs`.

### 7. Wrong vs Correct

#### Wrong

```text
runtime prompt -> projection_receipts.rendered_prompt
              -> trace.jsonl.prompt
              -> comparison_manifest.prompt
```

#### Correct

```text
canonical rendered bytes (in memory)
  -> SHA-256 + section metadata -> immutable production receipt -> commit
  -> PreparedProjection -> runtime process

controlled comparison
  -> same frozen inputs +/- one target StateId
  -> selected-ref/digest invariant report (no prompt persistence)
```

The receipt proves selection and projection. V0 comparison evidence remains
bounded and cannot become a second durable prompt authority.

## Handoff and Projection

- A checkpoint is task state, not a bag of recalled facts.
- Hard constraints are carried by stable `StateRef`, not repeatedly rewritten
  prose.
- Every open loop is inherited, completed, abandoned, or explicitly replaced
  in the next checkpoint version.
- Every runtime projection follows the production receipt contract above.
- Store enough structured metadata for a representative removed-state
  comparison without retaining raw secret-bearing prompt text.
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

## Scenario: C1 StateWriter Trust, Time, and Derived Search

### 1. Scope / Trigger

Apply this contract to every extractor proposal, create/supersede/revoke
transition, historical selection, legacy import, and state search.

### 2. Signatures

```rust
pub struct StateScope {
    pub subject: StateSubject,
    pub applicability: StateApplicability,
}

pub struct StateApplicability {
    pub workspace: Option<WorkspaceId>,
    pub operating_system: Option<OperatingSystem>,
    pub task_tags: Vec<String>,
    pub language_tags: Vec<String>,
    pub required_capabilities: Vec<String>,
}

pub struct StateRecord {
    // identity, key, kind, scope, content, strength, status, evidence, provenance
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub deactivated_at: Option<Timestamp>,
    pub supersedes_state_id: Option<StateId>,
}
```

Applicability is capability-oriented. It never contains Claude/Codex runtime
selection. Runtime compatibility belongs to capabilities and the later
projection selector.

### 3. Contracts

- Recorded/structured DTOs contain semantic proposals only: key, kind, content,
  and optional expiry. Trusted extraction context supplies scope, current event
  evidence, inferred strength, and provenance.
- StateWriter validates key/scope alignment, bounded metadata, secret policy,
  evidence existence, non-permission evidence, spirit identity, evidence time,
  lifecycle causation, TTL, conflicts, and strength combinations.
- `Repeated` requires at least two distinct Chronicle event IDs. `Explicit`
  currently accepts only the versioned GNU rule plus an allowlist of normalized
  imperative/project-assertion sentences containing exact `gnu` and `windows`
  tokens. Negation, opposition, questions, hedges, one-off language, and
  substring lookalikes are rejected.
- `created_at <= as_of`, `expires_at > as_of`, and
  `deactivated_at > as_of` define historical selection. Supersede/revoke update
  status plus `deactivated_at`; content/evidence versions remain immutable.
  Revocation evidence stays on the lifecycle event causation chain. StateWriter
  rejects a create older than an existing version for the same key and scope.
- State lifecycle causation references one transition evidence event. A newly
  attached source must match lifecycle quest/session/spirit and causation.
- FTS is derived. `search_states(&mut self, ...)` rebuilds active-state FTS from
  canonical rows before applying rank/limit, preventing stale rows from hiding
  current results.
- Schema migration 2 adds `deactivated_at`, reconstructs superseded/revoked
  intervals from lifecycle Chronicle events (with successor creation as the
  supersede fallback), and rejects an inactive row whose interval is unprovable.
- Version-one legacy completion markers migrate as untrusted version `0`.
  Completion is recorded only when every row is imported/unchanged. Conflicting
  changed facts remain visible through the isolated fallback.
- Legacy write, snapshot, recall, and briefing boundaries enforce field budgets
  and the shared secret policy. A sensitive pre-C1 row remains available for
  explicit review while runtime-facing projections omit its content.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Negative/opposed/question/hedged GNU text or `Gnumeric` lookalike | no rule draft |
| Repeated strength with duplicate/one evidence ID | `RepeatedEvidenceRequired` |
| Evidence after transition or from another spirit | typed validation error; rollback |
| Lifecycle cause absent/unrelated | `EvidenceChainMismatch`/invalid lifecycle; rollback |
| Key names another workspace or metadata contains secrets/controls | `InvalidMetadata` |
| Query before creation / after expiry / at-or-after deactivation | state excluded |
| Create predates an existing version for the same key/scope | `BackdatedTransition`; rollback |
| Search immediately after write or after revoke | rebuilt FTS returns only active rows |
| Recorded DTO supplies scope/strength/evidence or exceeds 1 MiB | `ExtractError::Malformed` |
| Legacy rows exceed row/field/aggregate budgets | `LegacyBudgetExceeded` |
| Legacy text matches secret policy | reject new write; omit old row from snapshot/brief |
| Changed imported legacy row conflicts | skip is observable; completion stays open |

### 5. Good / Base / Bad Cases

- **Good**: current user event + trusted workspace scope -> draft -> transactional
  source/state/evidence/lifecycle write -> historical selection and fresh search.
- **Base**: irrelevant input yields no draft and no durable write.
- **Bad**: deserialize model-controlled scope/strength/evidence directly into a
  writable `StateDraft` or rely on stale FTS rows after state mutation.

### 6. Tests Required

- Explicit GNU positives plus negation/opposition/question/one-off/lookalike negatives.
- Repeated distinct-evidence, future evidence, source identity, and causation.
- Pre-creation, expiry, supersede, revoke, backdated-create rejection,
  immutable evidence refs, and historical `as_of` selection.
- Metadata/key/scope sentinel rejection with atomic rollback.
- Recorded DTO trust-field/unknown-field/input-budget rejection.
- Immediate search, stale-limit, export rebuild, and canonical reopen.
- Schema-1 database migration to schema 2, including inactive intervals and
  invalidation of pre-versioned legacy completion markers.
- Changed legacy fact conflict preserving fallback visibility.
- Sensitive pre-C1 row import skip with snapshot/brief projection exclusion.

### 7. Wrong vs Correct

#### Wrong

```text
model JSON(scope + strength + evidence) -> StateWriter -> stale FTS
```

#### Correct

```text
model semantic proposal + trusted ExtractionContext
  -> bounded StateDraft
  -> causal/temporal StateWriter gate
  -> SQLite canonical state
  -> rebuildable FTS/export
```