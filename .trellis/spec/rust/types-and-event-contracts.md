# Types and Event Contracts

## Scenario: C1 Durable Event Envelope

### 1. Scope / Trigger

Apply this contract whenever adapter output becomes a durable event, a fixture is
replayed, or Chronicle reads/writes an envelope. Adapters emit normalized
`KernelEventPayload`; a host or deterministic fixture seam supplies envelope
identity and projection attribution before persistence.

### 2. Signatures

Durable identities are transparent string newtypes: `EventId`, `QuestId`,
`SessionId`, `OwnerId`, `WorkspaceId`, `SpiritId`, `ExecutionId`, `StateId`,
`CheckpointId`, `ProjectionId`, `CorrelationId`, and `ArtifactId`.

```rust
pub struct RuntimeBinding {
    pub kind: RuntimeKind,
    pub mode: RuntimeMode,
}

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

The serialized field is exactly `runtime`. `ExecutorId`, `BackendKind`, and
`runtime_binding` are obsolete probe vocabulary.

### 3. Contracts

- `schema_version == KERNEL_EVENT_SCHEMA_VERSION` on JSONL load, Chronicle
  append, and Chronicle replay.
- Event/quest/session/spirit IDs are always non-empty bounded labels.
- Runtime lifecycle events require execution plus runtime binding.
- Tool start/end require execution, runtime, correlation, and a non-empty
  projection ID.
- Permission request/decision and projection creation require execution,
  runtime, and correlation.
- Projected outcomes require execution, runtime, correlation, and a non-empty
  projection ID. Pre-projection launch outcomes may omit projection context.
- State lifecycle events require causation and StateWriter binds that cause to
  transition evidence.
- Vendor IDs remain `VendorEventRef { namespace, id }`; they never become global
  event IDs without host namespacing.
- `SensitiveText` has redacted `Debug`/`Display` and no serde. `PersistedText`
  and `PersistedJson` are serializable reviewed values with redacted `Debug`.
- Durable text is bounded to 65,536 characters; JSON is sanitized to depth 32,
  64 items per collection, 512 characters per untrusted string, and 65,536
  serialized bytes.
- Shared untrusted-text sanitization removes terminal controls, bidi overrides,
  zero-width format characters, isolates, and BOM. Newline, carriage return,
  and tab become one visible space before secret detection and persistence.
- Outcome wire values distinguish `permission_denied`, `safety_unsupported`,
  and `degraded` from cancelled, failed, timeout, malformed output, non-zero
  exit, and launch failure.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Newer/unknown schema | `EventContractError::UnsupportedSchema` |
| Missing execution/runtime/correlation/projection | `MissingAttribution` |
| Empty, control/format-bearing, oversized, or sensitive durable ID/label | `InvalidField` or `SensitiveContent`; untrusted display text removes terminal-unsafe characters |
| Unredacted credential in text/JSON/metadata | `SensitiveContent`; write nothing |
| Oversized adapter or event JSONL line | typed line error with source line; stop reading near 1 MiB |
| Malformed known vendor event | typed adapter error; never fabricate success/default IDs |
| Unknown optional vendor event | documented forward-compatible skip |
| SQLite replay contains invalid stored JSON | surface JSON/event-contract error |

### 5. Good / Base / Bad Cases

- **Good**: adapter tool payload -> host adds execution/runtime/correlation and
  projection -> Chronicle validates/appends -> replay returns the same envelope.
- **Base**: system initialization vendor noise maps to no product event.
- **Bad**: drive Theater with an unattributed tool payload and later assume the
  same object can be persisted.

### 6. Tests Required

- Transparent newtype and enum wire round trips.
- Envelope round trip with runtime/correlation/projection.
- Empty semantic ID and missing attribution rejection.
- Bounded JSONL reader that proves it stops before consuming an oversized line.
- Adapter malformed-known-event and redaction sentinels.
- Shared redaction regression proving bidi, zero-width, and control characters
  cannot survive into terminal-facing persisted text.
- Adapter -> enriched envelope -> Chronicle reopen/replay -> Theater integration.
- SQLite replay revalidation after simulated stored-schema corruption.

### 7. Wrong vs Correct

#### Wrong

```text
vendor JSON -> Theater
           -> separately invented Chronicle event with guessed IDs
```

#### Correct

```text
vendor JSON -> adapter payload
            -> host envelope assignment + validate_kernel_event
            -> Chronicle append/replay
            -> Theater and Soul consume the same typed event
```