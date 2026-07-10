# Runtime CLI Contracts Research

## Scope

C1 uses the owner's two primary tools as runtime bindings while keeping the
Spirit and state ledgers vendor-neutral:

- Claude Code CLI as runtime A.
- Codex CLI as runtime B.

This note records version-sensitive external facts used by planning. Adapter
implementations must still probe the installed CLI version and validate fixture
schemas; this file is not permission to assume future output compatibility.

## Claude Code CLI

Official reference:
<https://code.claude.com/docs/en/cli-usage>

Verified on 2026-07-10:

- `claude -p` is the non-interactive/print path.
- `--output-format stream-json` emits streaming JSON output in print mode.
- `--input-format stream-json` is available for JSONL user messages on stdin.
- `--verbose` exposes the full turn-by-turn stream and is required by several
  streaming features.
- `--permission-prompt-tool` delegates non-interactive permission prompts to an
  MCP tool; Tsukumo must not use `--dangerously-skip-permissions` as a shortcut
  for the Safety Plane.
- `--no-session-persistence` can suppress Claude's own session persistence for
  controlled C1 smokes, but Tsukumo's Chronicle remains independent.
- `--version` reports the installed version and should be captured in live-smoke
  metadata.

Reference command shape, subject to installed-version probing:

```text
<prompt via stdin> | claude -p --output-format stream-json --verbose ...
```

Claude was not available on the current WSL `PATH` during this planning pass.
The opt-in live smoke therefore needs an actionable prerequisite check rather
than silently skipping after the gate is enabled.

## Codex CLI

Official reference:
<https://developers.openai.com/codex/noninteractive>

Verified on 2026-07-10:

- `codex exec` is the non-interactive path.
- `--json` changes stdout to JSONL with events including `thread.started`,
  `turn.started`, `turn.completed`, `turn.failed`, `item.*`, and `error`.
- Item payloads can cover agent messages, reasoning, command execution, file
  changes, MCP calls, web searches, and plan updates.
- `--ephemeral` disables Codex rollout-file persistence for the invocation.
- The default sandbox is read-only; `--sandbox workspace-write` is the smallest
  documented mode that permits edits. `danger-full-access` is reserved for a
  controlled isolated environment.
- `--ignore-user-config` can reduce personal configuration drift in a controlled
  comparison. Repository instructions/rules remain part of the task unless the
  CaseBundle explicitly defines otherwise.
- Credentials and `auth.json` must not enter fixtures, logs, receipts, or
  CaseBundles.

Reference command shape:

```text
<prompt via stdin> | codex exec --json --ephemeral --sandbox <profile> -
```

The locally visible binary reported `codex-cli 0.144.0-alpha.4` during this
planning pass. Fixtures must record their source version and the decoder must
surface schema drift instead of treating malformed known events as noise.

## C1 Verification Boundary

- Default CI replays committed, redacted Claude and Codex JSONL fixtures through
  the same decoders used by live transports.
- Real dual-runtime execution is opt-in through
  `TSUKUMO_RUN_LIVE_SMOKE=1`, runs in a disposable repository, records both CLI
  versions, and fails with an actionable prerequisite error if either CLI or
  authentication is missing.
- Fixture replay proves adapter and host contracts. The live smoke proves that
  the currently installed tools still satisfy those contracts. Neither alone
  proves that a projected state caused an action; the removed-state CaseBundle
  supplies that narrower comparison evidence.
