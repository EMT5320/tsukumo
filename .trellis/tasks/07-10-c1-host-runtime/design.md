# C1 Host and Runtime — Technical Design

## Scope

This child adds the composition root and proves one host-owned Claude CLI
execution. It consumes prepared projections from soul, owns process lifecycle,
envelopes normalized payloads, and fans committed events out to Chronicle and
theater. Codex and the final product UI remain in the next child.

## New Composition Root

Add `crates/tsukumo-host` as a library plus binary. The library owns testable
orchestration; `main.rs` only parses configuration, opens storage, and starts
the event loop.

```text
tsukumo-host/src/
  lib.rs
  config.rs
  orchestrator.rs
  process.rs
  safety.rs
  clock.rs
  main.rs
```

The host depends on kernel, adapters, soul, and theater. No product crate
depends back on host.

## Runtime Ports

Adapters own vendor command profiles and line decoders; host owns generic
process mechanics:

```rust
pub trait RuntimeProfile {
    fn binding(&self) -> RuntimeBinding;
    fn command(&self, config: &RuntimeLaunchConfig) -> Result<RuntimeCommandSpec, RuntimeError>;
    fn decoder(&self) -> Box<dyn RuntimeEventDecoder>;
}

pub trait ProcessRunner {
    fn spawn(
        &self,
        spec: RuntimeCommandSpec,
        prompt: SensitiveText,
    ) -> Result<Box<dyn RuntimeHandle>, ProcessError>;
}

pub trait RuntimeHandle {
    fn next(&mut self) -> Result<RuntimeOutput, ProcessError>;
    fn cancel_and_reap(&mut self) -> Result<ExitStatus, ProcessError>;
}
```

`RuntimeCommandSpec` never contains prompt text. `ProcessRunner` pipes the
prepared prompt to stdin, flushes/closes it for one-shot C1 runs, reads stdout
and stderr concurrently, and emits output incrementally. Its debug and error
representations redact environment values and never include stdin bytes.

The initial implementation may use blocking reader threads and channels; async
types do not leak into kernel, soul, adapters, or theater. The port leaves room
for an async/process-tree implementation later.

## Claude Profile

Reference command shape:

```text
stdin -> claude -p --output-format stream-json --verbose [safe flags]
```

The profile probes `claude --version` for live-smoke metadata. The current
Claude-like decoder is refactored from `Read -> Vec<KernelEvent>` into a
stateful per-line decoder returning normalized payloads. Fixture and live paths
instantiate the same decoder. Unknown documented vendor events produce an
observable skip; malformed known events fail with runtime/line context.

## Orchestration Ordering

```text
PreparedProjection (receipt already committed)
  -> append runtime-start-requested event
  -> ProcessRunner.spawn(prompt via stdin)
  -> append runtime-started or launch-failed event
  -> for each stdout JSONL line:
       adapter decode -> normalized payload(s)
       host assigns envelope IDs/correlation
       Chronicle append
       Director -> StageWorld
  -> terminal vendor event + child exit reconciliation
  -> append outcome
```

The first normalized event must reach Chronicle and StageWorld before process
completion. Chronicle append failure stops further execution and triggers
cancel/reap; theater is not allowed to display an uncommitted event as durable
history. Stderr is bounded/redacted diagnostic evidence, not parsed as normal
JSONL progress.

## Lifecycle State Machine

```text
Prepared -> Starting -> Running -> Completing -> Reaped
                    \-> Cancelling -> Reaped
                    \-> Failed -> Reaped
```

Timeout, explicit cancellation, malformed/truncated stream, non-zero exit, and
terminal success have separate outcome variants. Every path attempts reap once
and records whether cleanup succeeded. Deterministic fake processes drive CI
tests; live smoke is opt-in.

## Safety Plane

```rust
pub struct PermissionRequest {
    pub request_id: PermissionRequestId,
    pub execution_id: ExecutionId,
    pub runtime: RuntimeBinding,
    pub tool: String,
    pub arguments: RedactedValue,
    pub cwd: PathBuf,
    pub risk_reasons: Vec<RiskReason>,
}

pub enum PermissionDecision {
    AllowOnce,
    AllowSession,
    Deny,
}
```

The deterministic controller owns pending requests and session grants. The
model can request but never construct a decision. Decision events are Chronicle
evidence but are excluded from automatic relationship-state extraction.

Claude non-interactive permission handoff requires a real MCP
`--permission-prompt-tool` bridge. C1 defines and fixture-tests the bridge-facing
port. Until that bridge is proven live, a vendor permission request returns an
explicit degraded/unsupported result; the host never passes
`--dangerously-skip-permissions` or claims that vendor-internal prompts were
approved by Tsukumo.

## Configuration

Configuration is typed and has no implicit credential discovery in fixtures:

```text
runtime executable/path
working directory
sandbox/permission profile
timeout
stdout/stderr line and byte limits
live-smoke gate
```

Secrets stay in each CLI's own authenticated installation. Tsukumo does not
copy auth files or tokens into environment snapshots, receipts, or events.

## Test Strategy

- Command profile test: prompt sentinel absent from argv/env/debug/errors and
  exact bytes observed on fake-child stdin.
- Incremental decoder test: first payload delivered while fake child remains
  running.
- Envelope/Chronicle/theater integration with stable execution/projection IDs.
- Receipt failure/spawn spy test proving no process starts without a committed
  receipt.
- Timeout, cancel, malformed line, truncated stream, non-zero exit, stderr cap,
  and exactly-once reap tests.
- Safety state-machine tests for once/session/deny and no state-extraction
  escalation.
- Recorded Claude fixture through the same decoder and orchestrator.
- Opt-in live safe smoke with version/prerequisite reporting; no default CI
  credential dependency.

## Rollback and Operations

The host is additive to the workspace and can be disabled without altering
Chronicle/state data. Failed executions append explicit outcomes; they do not
delete receipts or events. Process-tree limitations on a target are reported
as unsupported and block a production-ready claim until tested on that target.
