# Types and Event Contracts

## Identity Types

Use opaque newtypes for durable identifiers instead of passing unrelated
strings through APIs. `ExecutorId` in
`crates/tsukumo-kernel/src/identity.rs` is the current reference shape:

- tuple newtype;
- `#[serde(transparent)]`;
- `Display`, `From<&str>`, and `From<String>` where useful;
- equality/hash/serde derives.

For C1, use the design vocabulary `SpiritId`, `ExecutionId`, `SessionId`,
`QuestId`, `EventId`, and state/checkpoint/projection IDs. A spirit ID is
persistent; a `RuntimeBinding` identifies the current backend/transport.
Vendor is compatibility metadata, never state ownership.

## Serialized Enums

The local wire convention is:

```rust
#[serde(tag = "type", rename_all = "snake_case")]
enum EventPayload { /* ... */ }
```

Use `rename_all = "snake_case"` for fieldless enums such as `BackendKind` and
`ActorPose`. Optional legacy/probe fields use `default` plus
`skip_serializing_if = "Option::is_none"`; identity and correlation fields in
new live C1 envelopes should be required once the host has assigned them.

References:

- `crates/tsukumo-kernel/src/event.rs`
- `crates/tsukumo-theater/src/stage.rs`
- `crates/tsukumo-soul/src/trace.rs`

## Normalize Once at the Boundary

`crates/tsukumo-adapters/src/stream_json.rs` owns the Claude-like NDJSON subset.
Follow that ownership model for every runtime:

1. Decode raw input at the adapter boundary.
2. Validate the known event shape.
3. Produce a vendor-neutral payload.
4. Let the host/event writer assign the durable envelope and Chronicle order.
5. Make theater, state reducers, filters, and replay consume the shared typed
   event rather than reparsing raw JSON.

Unknown vendor event kinds may be skipped only when the adapter explicitly
documents them as forward-compatible noise. A malformed known event must
surface an error with source location; do not silently fabricate durable IDs
such as `unknown` or `perm` for Chronicle records.

## C1 Event Envelope

New C1 work follows the frozen envelope fields from the convergence design:

```text
schema_version, event_id, occurred_at,
quest_id, session_id, spirit_id, execution_id,
runtime_binding, causation_id, correlation_id, payload
```

- Assign `event_id`, time, and Chronicle sequence in one event-writer path.
- Preserve source/vendor IDs as payload provenance; do not reuse them as global
  IDs without namespacing.
- Tool start/end and permission request/decision events share correlation IDs.
- Projection/tool/outcome events carry the execution and projection references
  needed to reconstruct the evidence chain.

## Replay Compatibility

- Live and fixture paths must deserialize the same persisted contract.
- Add `schema_version` before the second incompatible event shape lands.
- Update central types, adapter normalization, fixture JSONL, reducers,
  director mapping, and replay tests together.
- A schema change requires round-trip tests and at least one fixture replay.

