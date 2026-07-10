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

## Scenario: Projection Receipts Without Prompt Retention

### 1. Scope / Trigger

This contract applies whenever Tsukumo renders a checkpoint and selected
relationship state into the exact bytes passed to a runtime. The production
ledger must prove which inputs and renderer produced the projection without
turning secret-bearing prompts into a second transcript store.

The default is metadata-and-digest retention. A redacted canonical snapshot is
allowed only in an explicitly requested debug/evaluation `CaseBundle`; raw
unredacted prompt text is never a normal receipt, Chronicle payload, log field,
fixture, or error value.

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
    sections: Vec<ProjectionSectionDigest>,
    budget: ProjectionBudgetUsage,
    omissions: Vec<ProjectionOmission>,
    redactions: Vec<RedactionRecord>,
    debug_snapshot: Option<DebugSnapshotRef>,
    created_at: Timestamp,
}

struct ContentDigest {
    algorithm: DigestAlgorithm, // C1: Sha256
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

struct DebugSnapshotRef {
    artifact_id: ArtifactId,
    redacted_digest: ContentDigest,
    redaction_profile: String,
    retention: SnapshotRetention,
}
```

`SnapshotRetention` supports an expiring mode with an `expires_at` timestamp
and an explicitly retained mode. C1 defaults live debug snapshots to seven
days; preserving one longer requires an explicit user/evaluation choice.
Committed runtime fixtures are separately reviewed, synthetic artifacts and
are not live debug snapshots.

### 3. Contracts

- Hash the exact UTF-8 bytes passed to the runtime after the renderer has
  produced its canonical output. The renderer owns newline normalization and
  final-newline behavior; tests freeze both.
- Store the digest algorithm with every digest. C1 uses SHA-256 and never relies
  on a language/runtime default hasher.
- Section digests cover the canonical bytes for each named section. They aid
  diagnosis but do not replace `selected_state_refs` or checkpoint identity.
- Record both budget value and unit. Character, byte, and model-token budgets
  are not interchangeable; a token budget also identifies its tokenizer.
- Omission entries identify the candidate state and a deterministic reason such
  as scope mismatch, revocation, ranking, or budget exhaustion. Redaction
  entries identify location/category/action without copying the secret value.
- Persist the production receipt before spawning the runtime. A failed receipt
  write means the execution does not start.
- `rendered_prompt` remains an in-memory secret-bearing value at the host/runtime
  boundary. Its `Debug`/`Display` representations must redact content.
- Debug/eval mode first applies the named redaction profile, then writes the
  redacted canonical snapshot to a separate `CaseBundle` artifact. The snapshot
  has its own digest because it need not equal the bytes sent to the runtime.
- Expired snapshots are deleted by deterministic cleanup; their receipt,
  artifact identity, digest, redaction manifest, and expiry metadata remain
  explainable. An explicit retain choice removes automatic expiry.
- Historical receipts remain immutable after state supersession/revocation;
  later projections use current state and create new receipts.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Selected state/checkpoint reference is missing or inapplicable | Projection validation error; write no receipt and do not spawn |
| Renderer output differs with identical inputs/version | Determinism test failure; do not bless a new digest silently |
| Digest algorithm or renderer version is unknown | Compatibility error; receipt is still inspectable but cannot be claimed reproducible |
| Production receipt contains rendered text/raw prompt field | Schema/privacy test failure |
| Secret appears in receipt, Chronicle, error, fixture, or debug snapshot | Redaction/fixture validation failure |
| Receipt transaction fails | Do not launch the runtime |
| Debug snapshot requested but redaction or artifact write fails | Fail the debug/eval run before launch; ordinary production mode is unaffected |
| Snapshot expires | Delete snapshot bytes; retain receipt and deletion/audit metadata |
| Budget unit is absent or ambiguous | Validation error |

### 5. Good / Base / Bad Cases

- **Good**: a GNU constraint is selected, the receipt stores its `StateRef`,
  checkpoint/runtime/execution IDs, renderer versions, SHA-256 and section
  digests, budget unit, and no prompt text; an opt-in CaseBundle contains a
  separately hashed redacted snapshot with expiry.
- **Base**: production execution stores only the immutable receipt metadata and
  digest. The exact text can be regenerated while its renderer version remains
  supported, but is not retained as a log.
- **Bad**: serialize the entire prompt into `projection_receipts`, trace JSONL,
  a panic message, or a fixture because it is convenient for debugging.

### 6. Tests Required

- Golden test for canonical section ordering, newline normalization, final
  newline behavior, and stable SHA-256 with fixed renderer inputs/version.
- Mutation test proving a changed selected state or checkpoint changes the
  relevant section and overall digest while unrelated metadata does not.
- Serialization/schema test proving `ProjectionReceipt` has no rendered text
  field and serialized rows/logs do not contain a sentinel secret.
- Storage-before-spawn integration test: forced receipt failure leaves the fake
  runtime unstarted.
- Debug CaseBundle test proving redaction occurs before persistence and the
  snapshot digest describes the redacted bytes.
- Retention test for seven-day expiry, cleanup audit metadata, and explicit
  retain behavior using a controlled clock.
- With-state/without-state CaseBundle test proving all variables except the
  target state projection and resulting hashes remain equal.
- Historical audit test proving revoke/supersede changes future selection but
  does not rewrite an old receipt.

### 7. Wrong vs Correct

#### Wrong

```text
runtime prompt -> projection_receipts.rendered_prompt
              -> trace.jsonl.prompt
              -> test failure message
```

#### Correct

```text
canonical rendered bytes (in memory)
  -> SHA-256 + section metadata -> immutable production receipt -> commit
  -> runtime process

explicit debug/eval only
  -> redact -> separate expiring CaseBundle snapshot + its own digest
```

The receipt proves selection and projection; the optional sanitized artifact
supports diagnosis without making raw prompts durable by default.

## Handoff and Projection

- A checkpoint is task state, not a bag of recalled facts.
- Hard constraints are carried by stable `StateRef`, not repeatedly rewritten
  prose.
- Every open loop is inherited, completed, abandoned, or explicitly replaced
  in the next checkpoint version.
- Every runtime projection follows the production receipt and optional debug
  snapshot contract above.
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
