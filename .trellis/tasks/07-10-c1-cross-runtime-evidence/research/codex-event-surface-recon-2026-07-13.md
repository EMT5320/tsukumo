# Codex Event-Surface Recon — 2026-07-13

> Status: completed reconnaissance for implementation step 0.5
> Runtime: `codex-cli 0.135.0` on Windows
> Scope: synthetic, ephemeral, isolated prompts only; no repository task or personal prompt content entered the captured stdout stream

## Sources

- Live `codex exec --json --ephemeral` streams captured on 2026-07-13.
- [Official non-interactive JSONL contract](https://learn.chatgpt.com/docs/non-interactive-mode#make-output-machine-readable).
- [Official `codex exec` command reference](https://learn.chatgpt.com/docs/developer-commands#codex-exec).
- Local CLI help and installed binary strings for version-specific confirmation.

Official documentation names the top-level families `thread.started`, `turn.started`, `turn.completed`, `turn.failed`, `item.*`, and `error`. Documented item families include agent messages, reasoning, command executions, file changes, MCP tool calls, web searches, and plan updates.

## Live Cases

| Case | Sandbox | Observed item/result | Evidence status |
|---|---|---|---|
| Exact message | read-only | completed `agent_message`, terminal usage | valid |
| Read-only command | read-only | `command_execution` started/completed, exit `0` | valid |
| Policy-declined command | read-only | completed command with status `declined`, exit `-1` | valid |
| Missing-file command | read-only | completed command with status `failed`, exit `1` | valid |
| Unverified file request | workspace-write requested | model claimed success without a tool or file | rejected as evidence |
| Verified file request | workspace-write requested | command was policy-declined; no file or `file_change` event | valid negative observation |

Every non-empty stdout line in the retained temporary cases parsed as one JSON object. Stderr contained launcher, model-cache, plugin/skill, shell-snapshot, and analytics diagnostics while stdout remained JSONL-only.

## Observed Grammar

### Message-only turn

```text
thread.started
turn.started
item.completed(agent_message)
turn.completed(usage)
```

### Command turn

```text
thread.started
turn.started
item.started(command_execution, status=in_progress)
item.completed(command_execution, status=completed|failed|declined, exit_code)
item.completed(agent_message)
turn.completed(usage)
```

Observed command fields:

- `item.id`
- `item.type == "command_execution"`
- `command`
- `aggregated_output`
- `exit_code` (`null` while in progress; `0`, positive non-zero, or `-1` at completion)
- `status` (`in_progress`, `completed`, `failed`, or `declined`)

The stream emitted start/end granularity and final aggregated output. No command-output delta or `item.updated` event appeared in these cases.

## Normalization Decision

The first Codex decoder should be stateful and deliberately narrow:

| Codex event | Decode disposition / Kernel mapping |
|---|---|
| `thread.started` | `KnownIgnored`; retain `thread_id` in decoder state for vendor-call namespacing |
| `turn.started` | `KnownIgnored`; open one turn and reset terminal state |
| `item.started(command_execution)` | `ToolStart`; vendor ref namespace `codex_cli`, ID composed from thread and item IDs; tool name `command_execution`; reviewed command in args |
| `item.completed(command_execution)` | `ToolEnd`; correlate with the started item; `is_error` when status is not `completed` or exit code is non-zero; retain reviewed status/exit/output summary |
| `item.completed(agent_message)` | `KnownIgnored`; the current kernel has no assistant-text payload and must not persist full model text as an accidental prompt snapshot |
| documented reasoning / plan items | `KnownIgnored` with observable counters |
| documented file-change / MCP / web-search items without a captured versioned fixture | registered `KnownIgnored`; do not invent field layouts or tool success |
| unknown top-level or item type | `UnknownSkipped` with observable counters |
| `turn.completed` | terminal execution `Outcome(Succeeded)` after pending-tool validation; individual failed/declined commands remain explicit `ToolEnd` errors |
| `turn.failed` | generic `Error` plus terminal `Outcome(Failed)`; do not fabricate detail fields until a versioned failure fixture is captured |
| top-level `error` | typed adapter error or normalized `Error`, based on whether the line is a malformed contract or a valid runtime failure |

`finish()` must reject a stream that has an open turn, pending command item, or no terminal turn event. Completed-only non-tool items are valid because live `agent_message` items did not emit `item.started`.

## Safety and Input Findings

- `codex exec` is a one-turn non-interactive surface. The prompt may arrive on stdin, and stdin closes after delivery. Mid-turn user injection is not exposed by this contract; `exec resume` supplies a later-turn prompt.
- Non-interactive production should set `approval_policy="never"` explicitly through `-c` and expose `RuntimeSafetyCapability::DenyUnapproved`.
- A declined command is observable inside `command_execution`; the live stream did not emit a separate permission/control-request object. A live approval bridge therefore remains outside this V0 `exec --json` adapter and requires a different surface such as app-server/ACP.
- `--ignore-user-config` leaves authentication available and did not make stderr quiet on this installation. Stderr is diagnostic input only.
- Recon used `--ignore-rules` only inside synthetic cases. The production profile must preserve owner/project exec policies.
- `--ephemeral` skips rollout persistence but still initializes local Codex state. A read-only outer environment can fail before the model call; the Host must return an actionable launch failure.

## Windows Launch Finding

Launching the npm `codex.ps1` wrapper through `powershell -File` did not preserve the trailing stdin marker reliably. The npm `codex.cmd` application entry accepted redirected stdin/stdout/stderr. The production profile needs a Windows launcher-resolution test and must never assume that a PowerShell command alias is directly spawnable by `std::process::Command`.

## Implementation Boundary

Step 1 should implement and fixture-test the observed lifecycle, command, agent-message, ordinary failure, and policy-decline shapes first. File-change, MCP, web-search, and plan families stay observable known skips until a reviewed `0.135.0` fixture captures their actual fields. This preserves forward compatibility and satisfies the smallest-honest-normalization rule without fabricating product evidence.