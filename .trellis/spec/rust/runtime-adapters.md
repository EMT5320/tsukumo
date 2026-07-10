# Runtime Adapters

## Scenario: Claude and Codex C1 Runtime Pair

### 1. Scope / Trigger

C1 validates continuity across the owner's two primary tools:

- Runtime A: Claude CLI own-process `stream-json`.
- Runtime B: Codex CLI non-interactive `codex exec --json`.

Both emit JSON Lines but use different vendor schemas. Tsukumo owns each child
process, prompt projection, lifecycle, and normalization boundary. Recorded
vendor streams run in default CI; authenticated live cross-runtime execution is
an explicit opt-in smoke.

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

trait RuntimeEventDecoder {
    fn decode_line(&mut self, line: &str) -> Result<Vec<KernelEventPayload>, DecodeError>;
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
  the host to envelope and persist.
- Preserve vendor event/item IDs as namespaced provenance for tool start/end
  correlation.
- Default CI uses committed, redacted Claude and Codex JSONL fixtures.
- Live smoke is opt-in (planned gate: `TSUKUMO_RUN_LIVE_SMOKE=1`) and requires
  both CLIs plus local authentication. If explicitly enabled, missing runtime or
  authentication is a failure rather than a skip.
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

- **Good**: Claude and Codex fixture streams normalize to the same tool/outcome
  contract; an opt-in local smoke proves a GNU constraint crosses between the
  real installed tools.
- **Base**: default CI runs both decoders and the full CaseBundle without any
  external model call.
- **Bad**: make CI depend on personal CLI auth, or call a transcript watcher a
  drive-tier cross-runtime proof.

### 6. Tests Required

- Per-adapter line decoder tests for tool start/end, completion/failure, unknown
  event, malformed known event, and truncated stream.
- Shared conformance suite asserting both adapters produce equivalent normalized
  semantics for the C1 case.
- Incremental host test proving the first normalized event is observed before
  child completion.
- Command-spec/privacy test proving a sentinel prompt is absent from argv, env,
  `Debug`, errors, and logs while the fake child receives the exact bytes on
  stdin.
- Cancellation/reap test using a deterministic fake process.
- Fixture secret/redaction validation.
- Default-CI CaseBundle using recorded Claude + Codex JSONL.
- Opt-in live smoke using both locally authenticated CLIs in a disposable test
  repository; report runtime versions with the artifact.

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
