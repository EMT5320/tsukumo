# Runtime Adapters

## Scenario: Claude First, Codex Extension

### 1. Scope / Trigger

C1 starts continuity with the owner's first owned-process runtime and preserves the same port for the second:

- Runtime A: Claude CLI own-process `stream-json`.
- Runtime B: Codex CLI non-interactive `codex exec --json`.

Both emit JSON Lines but use different vendor schemas. Tsukumo owns each child
process, prompt projection, lifecycle, and normalization boundary. The reviewed
Claude stream runs in default CI; its authenticated live execution is an
explicit opt-in smoke. Codex conformance becomes mandatory with that profile.

Evidence:

- Existing Claude subset: `crates/tsukumo-adapters/src/stream_json.rs` and the
  archived A1 Windows channel notes.
- Codex official non-interactive contract:
  `https://developers.openai.com/codex/noninteractive` (`exec --json` emits
  thread/turn/item/error JSONL events and supports explicit sandbox settings).

### 2. Signatures

The host-facing boundary should be equivalent to:

```rust
trait RuntimeProfile {
    fn binding(&self) -> &RuntimeBinding;
    fn command(&self, config: &RuntimeLaunchConfig) -> Result<RuntimeCommandSpec, RuntimeError>;
    fn decoder(&self) -> Box<dyn RuntimeEventDecoder>;
}

enum DecodeDisposition {
    Emitted,
    KnownIgnored,
    UnknownSkipped,
}

struct DecodedRuntimeLine {
    line_number: usize,
    disposition: DecodeDisposition,
    payloads: Vec<KernelEventPayload>,
}

trait RuntimeEventDecoder {
    fn decode_line(&mut self, line: &str) -> Result<DecodedRuntimeLine, AdapterError>;
    fn finish(&self) -> Result<(), AdapterError>;
}

struct RuntimeRequest {
    execution_id: ExecutionId,
    projection_id: ProjectionId,
    rendered_prompt: SensitiveText,
    launch: RuntimeLaunchConfig,
}

struct RuntimeLaunchConfig {
    working_dir: PathBuf,
    sandbox: SandboxProfile,
}

struct RuntimeCommandSpec {
    program: PathBuf,
    args: Vec<OsString>,
    env: Vec<(OsString, OsString)>,
    prompt_delivery: PromptDelivery, // C1: Stdin
}

trait ProcessRunner {
    fn spawn(
        &self,
        spec: RuntimeCommandSpec,
        prompt: SensitiveText,
    ) -> Result<Box<dyn RuntimeHandle>, ProcessError>;
}
```

The adapter-owned profile builds vendor arguments and decodes vendor output.
The host-owned process runner performs spawn, stdin delivery, cancellation, and
reaping. This split keeps vendor flags out of the host without letting adapters
own product orchestration.

Reference live commands (exact flags remain adapter-owned and version-tested):

```text
<prompt via stdin> | claude -p --output-format stream-json --verbose ...
<prompt via stdin> | codex exec --json --ephemeral --sandbox <profile> -
```

The Codex adapter may use `--ignore-user-config` in controlled comparison
smokes to reduce personal configuration drift, but must not silently ignore
repository instructions/rules that are part of the task environment.

### 3. Contracts

- Runtime bindings identify `claude_cli` and `codex_cli` independently from the
  persistent `SpiritId`.
- Read stdout incrementally line by line; do not wait for process completion to
  deliver normalized events.
- Deliver rendered prompt bytes through the child's stdin. Prompt text must not
  appear in argv, environment variables, command diagnostics, or process-list
  metadata. Close stdin after the complete prompt is written unless the adapter
  explicitly negotiates a multi-turn input stream.
- Keep stderr/process exit status as diagnostics and lifecycle evidence; do not
  parse ordinary progress text as vendor JSONL.
- Decode each vendor format once in its adapter, then emit shared payloads for
  the host to envelope and persist. Every syntactically valid line also carries
  an `Emitted`, `KnownIgnored`, or `UnknownSkipped` disposition.
- Preserve vendor event/item IDs as namespaced provenance for tool start/end
  correlation.
- Default C1 CI uses committed, redacted Claude and versioned Codex JSONL fixtures.
- Live smoke is opt-in (`TSUKUMO_RUN_LIVE_SMOKE=1`). The dual-runtime gate
  requires both CLIs plus local authentication. If explicitly enabled, a missing
  runtime or authentication failure is an error rather than a skip.
- Live smoke runs in a controlled fixture repository with the least sandbox
  capable of the target action. Credentials and auth files never enter fixtures,
  Chronicle payloads, receipts, or test artifacts.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Unknown documented vendor event kind | Skip with observable decode metric/log; no fabricated kernel event |
| Malformed known JSONL event | `DecodeError` with runtime and line context |
| Child exits non-zero | Runtime failure event plus exit diagnostics; process is reaped |
| Prompt stdin write/flush fails | Launch failure; terminate/reap child and persist no successful runtime-start claim |
| Prompt appears in argv/env/debug output | Privacy contract failure |
| JSONL ends without terminal turn/result | Truncated-run error, not successful completion |
| Cancellation/timeout | Terminate then reap; persist explicit cancellation/timeout |
| Live gate disabled | Fixture tests run; no credential or network requirement |
| Live gate enabled but CLI/auth missing | Smoke fails with actionable prerequisite error |
| Fixture contains token/path/user secret | Fixture validation fails before commit |
| Runtime requests permission unsupported by transport | Explicit unsupported/degraded safety result; never auto-approve |

### 5. Good / Base / Bad Cases

- **Good**: the Claude fixture and opt-in live path normalize through the same
  tool/outcome, envelope, Chronicle, and Theater contract. Future Codex
  conformance must reuse those ports.
- **Base**: default CI runs the Claude decoder, Host fixture, and fake/real
  local child contracts without an external model call.
- **Bad**: make CI depend on personal CLI auth, or call a transcript watcher a
  drive-tier cross-runtime proof.

### 6. Tests Required

- Per-adapter line decoder tests for tool start/end, completion/failure, unknown
  event, malformed known event, and truncated stream.
- When Codex is implemented, add a shared conformance suite asserting both
  adapters produce equivalent normalized semantics.
- Incremental host test proving the first normalized event is observed before
  child completion.
- Command-spec/privacy test proving a sentinel prompt is absent from argv, env,
  `Debug`, errors, and logs while the fake child receives the exact bytes on
  stdin.
- Cancellation/reap test using a deterministic fake process.
- Fixture secret/redaction validation.
- Default-CI Host bundle using recorded Claude JSONL; it persists no rendered prompt snapshot.
- Opt-in live smoke using the selected locally authenticated CLI in a
  disposable repository; report its runtime version.

### 7. Wrong vs Correct

#### Wrong

```text
Claude transcript watcher + Codex session-file watcher -> claim cross-runtime drive
```

This observes history but does not prove Tsukumo owned the prompt projection or
execution.

#### Correct

```text
checkpoint/receipt -> prompt on child stdin -> host-owned Claude or Codex process
                   -> incremental vendor JSONL adapter
                   -> normalized payload -> envelope/Chronicle/theater
```

Fixtures make the contract reproducible; the opt-in live smoke makes the
product claim honest without placing secrets or model nondeterminism in CI.

## Implemented C1 stream boundary

- `BriefCompiler`, `assemble_delegation_prompt`, and `assemble_with_trace` are
  A1 compatibility surfaces. Their size trace carries no checkpoint,
  selected-StateRef, renderer, or receipt identity. New hosts must launch only
  a receipt-committed `PreparedProjection`.
- `c1_state_theater_cross_layer.rs` proves projection receipt commit, SQLite
  reopen, Chronicle replay, and Theater metadata while a secret sentinel stays
  confined to the in-memory launch value.
- `parse_stream_json_reader` uses a bounded `fill_buf` loop. It stops near the
  1 MiB line limit before allocating the rest of an unterminated runtime line.
- Known tool/result/permission shapes use typed required/optional decoders.
  Missing/wrong-type required fields and unsupported known result subtypes are
  errors; unknown top-level compatibility noise remains an explicit skip.
- Vendor labels, text, JSON keys/values, and diagnostic subtype values are
  bounded and redacted before they leave the adapter.
- Adapter payloads deliberately carry no durable envelope IDs. A host seam must
  add execution/runtime/correlation and tool/outcome projection before calling
  `validate_kernel_event` or Chronicle append.
- The conformance path must prove adapter -> enriched envelope -> Chronicle
  reopen/replay -> Theater, in addition to pure adapter -> Theater behavior.

## Implemented C1 Claude Host Boundary

### 1. Scope / Trigger

Use this contract whenever tsukumo-host launches a receipt-committed Claude
projection, records a permission decision, or changes owned-process limits.
Claude and Codex now share the owned-process Host port. Each adapter keeps its
own vendor command and stateful JSONL decoder while Host retains process, receipt,
and durable event ownership.

### 2. Signatures

~~~rust
impl ClaudeRuntimeProfile {
    const fn isolated_smoke() -> Self;
}

trait RuntimeProfile {
    fn binding(&self) -> RuntimeBinding;
    fn command(&self, launch: &RuntimeLaunchConfig)
        -> Result<RuntimeCommandSpec, RuntimeProfileError>;
    fn decoder(&self) -> Box<dyn RuntimeEventDecoder>;
    fn safety_capability(&self) -> RuntimeSafetyCapability;
}

trait RuntimeEventDecoder {
    fn decode_line(&mut self, line: &str)
        -> Result<DecodedRuntimeLine, AdapterError>;
    fn finish(&self) -> Result<(), AdapterError>;
}

trait ProcessRunner {
    fn spawn(&self, launch: ProcessLaunch)
        -> Result<Box<dyn RuntimeHandle>, ProcessError>;
    fn process_tree_capability(&self) -> ProcessTreeCapability;
}

trait RuntimeHandle {
    fn next(&mut self, wait: Duration) -> Result<RuntimeOutput, ProcessError>;
    fn cancel_and_reap(&mut self) -> Result<ProcessExit, ProcessError>;
}

impl RuntimeOrchestrator {
    fn execute(&mut self, request: ExecutionRequest<'_>)
        -> Result<ExecutionReport, HostError>;

    fn record_permission_resolution(
        &mut self,
        receipt: &ProjectionReceipt,
        context: ExecutionContext,
        resolution: PermissionResolution,
    ) -> Result<AppendOutcome, HostError>;
}
~~~

### 3. Contracts

- RuntimeCommandSpec is immutable outside adapters and has no prompt field.
  ProcessLaunch carries SensitiveText; StandardProcessRunner writes exact bytes
  to stdin, flushes, closes stdin, and redacts prompt/output diagnostics.
- Before spawn, Host reloads ProjectionReceipt from the selected HostLedger and
  requires byte-for-byte receipt equality plus matching runtime binding.
- Host assigns execution/runtime/correlation/projection attribution. Each event
  enters Chronicle before drive_kernel_event; failed append never reaches
  Theater.
- ExecutionReport retains aggregate `known_ignored_lines` and
  `unknown_skipped_lines` counts. It never persists raw skipped vendor payloads.
- Claude C1 uses: -p --input-format text --output-format stream-json --verbose
  --no-session-persistence --permission-mode dontAsk. Permission bypass flags
  are forbidden.
- Default process limits are 1 MiB per stdout line, 64 KiB total stderr, and 32
  queued signals. Hard maxima are 16 MiB, 1 MiB, and 4096 respectively.
- StandardProcessRunner reports DirectChildOnly. Product claims must not say
  descendant process trees are managed until another runner proves it.
- A human permission resolution is durable only when execution/runtime/session
  scope matches the receipt and a matching PermissionRequested vendor
  reference already exists in Chronicle.
- TSUKUMO_RUN_LIVE_SMOKE=1 plus explicit --ignored is the only live model gate.
  Default tests compile the live path and perform no model call.
- The live profile runs from an empty temporary directory with `--safe-mode`,
  `--tools ""`, a fixed minimal system prompt, disabled prompt suggestions, and
  `--max-budget-usd 0.05`. The exact synthetic handoff is allowlisted by an
  ordinary non-live test before any external call can be authorized.

### 4. Validation & Error Matrix

| Condition | Result |
|---|---|
| Missing/different receipt or runtime mismatch | HostError; zero spawn |
| Duplicate deterministic start event | AlreadyExecuted; zero second spawn |
| Invalid executable/cwd/environment key | RuntimeProfileError before spawn |
| Output/channel limit is zero or above maximum | ProcessConfigError |
| Poll deadline cannot fit Instant | WaitDurationTooLarge |
| Malformed known line / missing terminal result | MalformedOutput with typed adapter detail |
| Timeout / cancellation / non-zero exit / launch failure | Distinct status and failure |
| Chronicle append fails while live | Cancel/reap; Theater sees no event; cleanup error retained |
| Permission request with unwired bridge | Durable request then SafetyUnsupported |
| Decision scope mismatch or request evidence absent | HostError; no decision append |
| Repeat identical decision append | Chronicle duplicate; no second Theater fan-out |
| Live gate disabled | Ignored smoke; fixture and fake paths still run |
| Live projection differs from its reviewed literal | Offline allowlist test fails; do not launch Claude |

### 5. Good / Base / Bad Cases

- Good: allowlisted synthetic receipt -> isolated tool-free Claude process ->
  incremental JSONL -> Chronicle acknowledgment -> Theater -> one outcome.
- Base: reviewed claude_c1_success.jsonl and fake/real local child tests run
  without credentials or model cost.
- Bad: spawn before receipt reload, display before Chronicle acknowledgment,
  accept unbounded output, or fabricate PermissionDecided from model output.

### 6. Tests Required

- runtime_profile_contract.rs: safe flags, redacted diagnostics, invalid env
  keys, stateful terminal enforcement, and reviewed fixture.
- process_contract.rs: exact stdin, concurrent bounded output, cancellation,
  idempotent reap, configuration maxima, and deadline overflow.
- orchestrator_contract.rs: receipt-first, incremental commit, attribution,
  Chronicle-before-Theater, and cleanup-error evidence.
- runtime_failures_contract.rs: malformed, truncated, timeout, cancellation,
  non-zero exit, and launch failure distinctions.
- runtime_permission_contract.rs and safety_contract.rs: once/session/deny,
  stale/duplicate requests, durable request prerequisite, scope checks,
  unsupported bridge, and no automatic StateRecord extraction.
- claude_live.rs: ordinary outbound-payload allowlist test plus ignored opt-in
  execution through the isolated profile, decoder, envelope writer, Chronicle,
  and Theater path.

### 7. Wrong vs Correct

#### Wrong

~~~rust
controller.decide(&vendor_request, PermissionDecision::AllowSession)?;
runner.spawn(command_with_prompt_in_args)?;
drive_kernel_event(&mut world, &event, &director);
store.append_event(&event)?;
~~~

#### Correct

~~~rust
let prepared = store.prepare_projection(write)?;
let report = host.execute(receipt_checked_request(&prepared))?;
let resolution = controller.decide(&vendor_request, human_decision)?;
host.record_permission_resolution(&prepared.receipt, context, resolution)?;
~~~

Host owns process mechanics and event order; adapters own vendor flags and
decoding; Soul remains the durable authority.

For a live smoke, never use the repository working directory or the standard
profile. Construct an allowlisted synthetic projection and run
`ClaudeRuntimeProfile::isolated_smoke()` in an empty temporary directory.

## Implemented C1 Codex Profile and Removed-State Evidence

### 1. Scope / Trigger

Use this contract when changing `CodexRuntimeProfile`, `CodexJsonDecoder`,
Codex fixtures, the controlled Claude-to-Codex comparison, or the dual-runtime
live gate. The reviewed schema version is `codex-cli 0.135.0`; future schema
changes require a new fixture or an explicit compatibility decision.

### 2. Signatures

```rust
impl CodexRuntimeProfile {
    const fn read_only() -> Self;
    const fn workspace_write() -> Self;
    const fn isolated_smoke() -> Self;
    fn version_command(&self, launch: &RuntimeLaunchConfig)
        -> Result<RuntimeCommandSpec, RuntimeProfileError>;
}

impl RuntimeProfile for CodexRuntimeProfile {
    fn binding(&self) -> RuntimeBinding; // codex_cli/owned_process
    fn command(&self, launch: &RuntimeLaunchConfig)
        -> Result<RuntimeCommandSpec, RuntimeProfileError>;
    fn decoder(&self) -> Box<dyn RuntimeEventDecoder>;
    fn safety_capability(&self) -> RuntimeSafetyCapability; // DenyUnapproved
}

struct CodexJsonDecoder {
    thread_id: Option<String>,
    turn_open: bool,
    terminal_seen: bool,
    pending_commands: HashSet<String>,
    tool_error_seen: bool,
}
```

Reference command shape:

```text
<prompt on stdin> | codex exec --json --ephemeral --color never   --sandbox read-only|workspace-write -c approval_policy="never" -
```

`isolated_smoke()` additionally uses `--ignore-user-config`,
`--skip-git-repo-check`, disables apps, remote plugins, multi-agent, and
memories, and disables web search. It runs from an empty temporary directory.

### 3. Contracts

- `read_only()` is the default. `workspace_write()` is an explicit capability
  choice; no profile exposes `danger-full-access` or a dangerous bypass flag.
- The final `-` is required so the rendered projection stays on stdin and out
  of argv, environment variables, and process diagnostics.
- On Windows, use a directly spawnable `codex.cmd` or native executable.
  Forwarding the final `-` through `powershell -File codex.ps1` was not reliable
  in the 0.135.0 recon.
- Stdout is the JSONL protocol. Stderr remains diagnostic-only even when it
  contains model-cache, plugin, skill, shell-snapshot, or analytics warnings.
- `thread.started` stores the bounded thread ID; `turn.started` opens one turn.
  Command item starts and completions must pair by item ID before a terminal
  turn is accepted.
- `command_execution` maps to namespaced `ToolStart`/`ToolEnd` payloads.
  `completed` with exit zero is non-error; `failed`, `declined`, or non-zero
  exit remains an explicit tool error. The decoder retains `tool_error_seen`;
  a later vendor `turn.completed` closes the stream but normalizes to
  `Outcome(Failed)` with a reviewed summary whenever any tool error occurred.
  This fail-closed rule prevents model-turn completion from becoming task
  success in Host, product read models, or Theater.
- `agent_message`, reasoning, file-change, MCP, web-search, and plan items are
  `KnownIgnored` until a versioned fixture supports a smaller honest payload.
  Future item or top-level families are `UnknownSkipped`.
- `turn.failed` emits a generic `Error` plus `Outcome(Failed)` without inventing
  unobserved vendor details. Known malformed shapes, unpaired items, duplicate
  terminals, and EOF without a terminal are typed failures.
- Vendor refs use namespace `codex_cli` and `<thread_id>:<item_id>` correlation.
  Host supplies durable EventId, execution, runtime, projection, and Chronicle
  ordering.
- The reviewed GNU pair contains policy-declined commands. Its versioned
  capture manifest binds fixture SHA-256 values, repository fixture SHA-256,
  and replay projection SHA-256 values. Exact capture model, model/user
  configuration digests, and capture prompt digests were not retained, remain
  explicit `null`, and force `causal_claim_eligible=false`.
- The pair proves only that reviewed captured command intents differ under the
  replay bindings. A completed vendor turn does not upgrade a declined Cargo
  action into task success.
- `cross_runtime_live.rs` and `claude_live.rs` are connectivity smokes. They
  prove receipt -> stdin -> decoder -> Chronicle -> terminal outcome plus
  version/prerequisite gates. Assistant text remains `KnownIgnored`, so these
  tests must not claim exact-response or checkpoint-semantic compliance.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Missing/wrong-type thread or item field | line-scoped `DecodeError` |
| Command completion without matching start | invalid item-ID sequence |
| Command update without pending start | invalid item-ID sequence |
| Unsupported command status | unsupported-known error |
| Any failed/declined/non-zero command followed by `turn.completed` | terminal `Outcome(Failed)`; Host report/product/Theater require review |
| Documented non-tool item | `KnownIgnored`, zero fabricated payloads |
| Unknown future family | `UnknownSkipped`, zero fabricated payloads |
| Second terminal turn | `AdapterError::DuplicateTerminal` |
| EOF with open turn/pending command/no terminal | `AdapterError::TruncatedStream` |
| JSONL line over 1 MiB | typed line-too-large error |
| Prompt or secret reaches args/output fixture | privacy/fixture test failure |
| Capture fixture/repository/replay digest differs from manifest | comparison test failure |
| Original capture control is unavailable | explicit `null`; causal claim remains ineligible |
| Dual live gate enabled with missing CLI/auth | explicit test failure |
| Connectivity smoke is described as exact-response proof | contract/documentation failure |

### 5. Good / Base / Bad Cases

- Good: receipt-committed projection -> stdin -> Codex owned process ->
  incremental JSONL -> adapter payload -> Host attribution -> Chronicle ->
  Theater/outcome.
- Base: default CI replays the versioned success and GNU comparison fixtures,
  verifies the capture manifest, and compiles allowlisted connectivity smokes;
  no CLI credential or model call is required.
- Bad: parse stderr as JSONL, treat a declined tool as successful, fill missing
  capture controls with assumptions, infer file or MCP field layouts without a
  capture, persist raw prompts, or claim exact-response/user value from replay
  and connectivity evidence.

### 6. Tests Required

- `codex_runtime_contract.rs`: command flags, stdin, sandbox selection, version
  probe, fixture hygiene, and Claude/Codex normalized conformance.
- `codex_decoder_contract.rs`: completed/failed/declined commands, sticky
  tool-error aggregation, item pairing, known/unknown items, terminal failure,
  truncation, duplicate terminal, and redaction.
- `cross_runtime_comparison_contract.rs`: source -> state -> checkpoint ->
  receipt -> Codex tool/outcome trace, fail-closed Host/product/Theater status,
  receipt invariants, capture-manifest digests and unavailable controls,
  normalized command difference, Host revoke, and historical receipt
  immutability.
- `cross_runtime_live.rs`: ordinary connectivity-payload allowlist plus ignored,
  explicitly gated dual-runtime version probes and owned-process executions;
  no assistant-text assertion is implied.
- Fixture scanning rejects user-home paths, temporary paths, auth material, and
  secret sentinels before commit.

### 7. Wrong vs Correct

#### Wrong

```text
with-state prompt -> hard-coded "success" event
without-state prompt -> hard-coded "success" event
=> claim GNU state improved the task
```

#### Correct

```text
same checkpoint + same goal + one excluded StateId
-> invariant-checked replay receipts
-> manifest-checked reviewed Codex JSONL command intents
-> production decoder/Host/Chronicle replay
-> report paired-capture difference and both failed outcomes
-> keep causal eligibility false while original capture controls are unavailable
```


## Implemented C1/C2 Evidence Episode Runner

### 1. Scope / Trigger

Use this contract when adding or changing the bounded handoff evidence CLI,
reviewed episode specs, delayed-resume gates, or C1/C2 machine summaries. The
runner closes the production gap between persisted checkpoints and the existing
receipt-first Host execution port. It is not a general evaluation platform.

### 2. Signatures

```text
tsukumo-host episode seed \
  --spec <reviewed.json> --data-dir <directory>

tsukumo-host episode resume \
  --spec <reviewed.json> --data-dir <directory> \
  --runtime-executable <path> --working-dir <directory> [--workspace-write]
```

```rust
struct EpisodeSpecV1 {
    schema_version: u16,
    episode_id: String,
    condition: EpisodeCondition, // C0 | C1 | C2
    source_runtime: EpisodeRuntimeV1,
    target_runtime: EpisodeRuntimeV1,
    source_summary: PersistedText,
    explicit_state_input: Option<PersistedText>,
    checkpoint: EpisodeCheckpointV1,
    projection: EpisodeProjectionV1,
    delay: EpisodeDelayV1,
    quality_gate: Vec<String>,
}

fn seed_episode(...) -> Result<EpisodeSeedSummaryV1, EpisodeError>;
fn resume_episode(...) -> Result<EpisodeRunSummaryV1, EpisodeError>;
```

### 3. Contracts

- The reviewed JSON input is bounded to 1 MiB and rejects unknown fields. It is
  a preregistration/source-review artifact, never a raw conversation export.
- C0 remains Trellis-only. Seed returns `manual_baseline_required=true` without
  creating a data directory; resume returns `ManualBaseline` without a receipt
  or process spawn.
- C1/C2 seed appends a reviewed `UserInput`, optionally sends one exact
  deterministic state statement through `RuleStateExtractor` and
  `StateWriter`, then saves an immutable checkpoint through `SoulStore`.
- Seed does not claim to capture a currently running external session. The
  source action and `source_summary` require human review before seed.
- C1 and C2 share one migration data plane. The durable-ID fingerprint
  canonicalizes condition to C1 so changing evidence visibility alone cannot
  change StateIds or rendered projection bytes.
- Stable IDs plus reuse of an already committed source-event timestamp make an
  identical seed retry idempotent even after interruption before checkpoint
  commit. Changing the reviewed
  migration input changes the fingerprint and cannot silently rewrite an
  existing checkpoint.
- The resume window is derived from the committed checkpoint timestamp.
  Too-early and expired calls fail before projection preparation or spawn.
- Resume calls `prepare_projection` before `RuntimeOrchestrator::execute`.
  The same SQLite database owns source events, state, checkpoint, receipt, and
  execution events.
- C1 summaries omit checkpoint/receipt/execution IDs, digests, and
  source/state counts. C2 exposes
  those metadata controls after execution. Neither summary contains rendered
  prompt bytes, stdout/stderr bodies, executable paths, or working-directory
  paths.
- Projection/runtime/storage timing and execution counters are machine
  observations. First-correct-action, owner intervention, task quality,
  diagnosis, recovery, and retained/modified/rolled-back remain manual fields.
- `--workspace-write` is explicit. The current Claude deny-unapproved profile
  has no matching workspace-write representation and rejects that combination.

### 4. Validation & Error Matrix

| Condition | Required result |
|---|---|
| Spec exceeds 1 MiB, has unknown fields, secrets, controls, or bad schema | typed spec error; no durable write |
| Source and target runtime families match | invalid spec |
| Optional explicit state input yields zero/multiple rule drafts | `ExplicitStateNotExtracted`; no checkpoint |
| C0 seed/resume | no Tsukumo storage/spawn; manual baseline result/error |
| Checkpoint absent for reviewed fingerprint | `MissingCheckpoint`; no receipt/spawn |
| Resume before/after committed window | `ResumeTooEarly` / `ResumeWindowClosed`; no receipt/spawn |
| Claude plus `--workspace-write` | fail before projection/spawn |
| Identical seed retry | same checkpoint/timestamp summary |
| Identical completed resume retry | Host `AlreadyExecuted`; zero second spawn |
| C1/C2 condition-only difference | identical rendered prompt; visibility-only summary difference |

### 5. Good / Base / Bad Cases

- Good: reviewed real source work -> source event/state/checkpoint -> delayed
  receipt -> owned target process -> Chronicle outcome -> manual utility review.
- Base: C0 records only the existing Markdown protocol and ordinary Trellis
  continuation; Tsukumo creates nothing.
- Bad: copy a chat transcript into the spec, mark a Markdown file seeded without
  a committed checkpoint, prepare C0 receipts, or launch before receipt commit.

### 6. Tests Required

- `cli_parse_contract.rs`: typed seed/resume flags and actionable missing
  runtime inputs.
- `episode_runner_contract.rs`: C0 zero-write/no-spawn, C1/C2 identical prompt,
  evidence visibility, seed idempotency, duplicate-execution no-spawn, delayed
  window no-spawn, SQLite reopen, and summary prompt/path absence.
- Existing product-controller tests remain the C2 selective-revoke authority;
  the episode runner must not implement a second revoke path.

### 7. Wrong vs Correct

#### Wrong

```text
copy conversation -> hand-written checkpoint DB row -> launch CLI
condition=C2 -> render a different prompt than C1
```

#### Correct

```text
reviewed source fact -> Chronicle/StateWriter -> immutable checkpoint
                    -> committed delay window
C1/C2 shared migration input -> ProjectionReceipt -> owned runtime
C1 hides metadata; C2 exposes the same receipt/provenance controls
```


## Episode registration and runtime preflight hardening

This section supersedes earlier episode-runner details where they differ.

The evidence runner keeps two digests with separate authority:

- the migration digest canonicalizes C1/C2 and execution profile so equivalent
  migration data produces identical StateIds and prompt bytes;
- the registration digest covers the complete reviewed spec, including the real
  condition and execution profile, and is stored as non-rendered immutable
  checkpoint metadata.

Seed and resume compare the registration digest before exposing evidence,
preparing a receipt, or probing a target runtime. A missing registration on an
older ordinary checkpoint remains readable by Soul but is not a valid episode
registration.

episode resume derives the runtime profile only from the reviewed spec.
--workspace-write acknowledges a frozen Codex workspace-write profile and never
selects capability. --confirm-live-run is mandatory before target execution.

Before receipt preparation, Host guards the working directory and runs the
adapter-owned prompt-free version command. Vendor-specific parsing proves the
selected family and exact reviewed target version without retaining raw probe
output or process paths. Probe failure creates no receipt and no target
execution lifecycle.

The inclusive delay window is checked again against the exact timestamp used
for durable RuntimeLifecycle::Starting. ExecutionReport returns that timestamp,
and the machine summary uses it as the sole episode-start time. A Starting
timestamp outside the window is rejected before target process spawn.

Episode JSON converts private recursively strict input DTOs into shared domain
types. Prompt-facing text rejects terminal controls, invisible format
characters, secrets, and personal home paths. Artifact locations are canonical
portable repository-relative paths.
