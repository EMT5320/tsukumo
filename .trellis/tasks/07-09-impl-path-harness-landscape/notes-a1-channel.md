# Notes: A1 Windows channel spike

- **Task**: `.trellis/tasks/07-09-impl-path-harness-landscape`
- **Date**: 2026-07-09
- **Host**: win32 10.0.22631 (PowerShell)
- **Decision**: **A1 default drive = own-process / recorded `stream-json` → `KernelEvent`**. Full ACP client deferred (A1.1+).

---

## 1. What was probed

| Probe | Result |
|---|---|
| `claude` CLI | **Available** — `C:\Users\Administrator\AppData\Roaming\npm\claude.ps1` → `@anthropic-ai/claude-code` **2.1.205** |
| `node` / `npm` / `npx` | Available (`C:\Program Files\nodejs\`) |
| `claude -p … --output-format stream-json --verbose` | **Works** — NDJSON on stdout (`system/init` → `assistant` → `result`) |
| Permission flags | `--permission-mode` present (`plan` / `manual` / `bypassPermissions` …). Interactive permission theater not fully exercised in this spike (plan-mode short prompt had no tool wait). |
| `@zed-industries/claude-code-acp` via `npx` | **Deprecated** (npm warn → migrate to `@agentclientprotocol/claude-agent-acp`). First `npx --help` **hung** (>30s) in the probe shell. |
| `@agentclientprotocol/claude-agent-acp` | Package resolves under `npx --yes`; **no live ACP `initialize` / `session/prompt` handshake** completed in this spike (deferred — avoid expanding into fs/terminal client). |

---

## 2. stream-json signal quality (observed + documented subset)

Live short run (`claude -p "reply with exactly: ok" --output-format stream-json --verbose --permission-mode plan`):

| Event | Observed | Maps to KernelEvent |
|---|---|---|
| `system` / `subtype: init` | Yes (tools list, `session_id`, `permissionMode`) | skip (session meta) |
| `assistant` + text content | Yes | skip text; **`tool_use` blocks → `ToolStart`** (parser) |
| `user` + `tool_result` | Not in this short run; documented in community cheatsheets | **`ToolEnd`** |
| `result` / `subtype: success` | Yes (`is_error: false`, `result`, cost) | **`TurnOrQuestEnd`** |
| Permission / control | Not emitted under plan + no tools | Parser accepts **`sdk_control_request`** / **`control_request`** (permission subtype) → **`WaitingPermission`**; CI uses **synthetic** lines |

Honest gap: **live waiting_permission from Claude was not captured** in this spike. F1 engineering uses the documented control-request subset + synthetic producer so CI does not require Claude or an interactive approval.

---

## 3. Channel decision (F2)

| Option | Verdict for A1 |
|---|---|
| ACP bridge (Claude adapter) | **Deferred** — package rename in flight; Windows `npx` flaky/hang risk; full client implies fs/terminal capability negotiation (out of A1 scope). |
| Own-process / fixture **stream-json** | **Selected default** — CLI present and emits structured NDJSON; parser + synthetic producer land in `tsukumo-adapters`. |
| JSONL watcher only | Remains S-line / observe-tier; **not** A1 drive pass by itself. |

**Narrative:** drive-tier = enough to act + enough to inject (briefing assembly hook stub). Not a full editor-host ACP client.

---

## 4. What's left for real Claude ACP

1. Pin `@agentclientprotocol/claude-agent-acp` (or successor) and run a **stdio JSON-RPC** smoke: `initialize` → `session/new` → `session/prompt`.
2. Map `session/update` (tool_call) + `session/request_permission` → same `KernelEvent` vocabulary (do not leak ACP types into theater).
3. Advertise **only** client capabilities the TUI can honor (no `fs/*` / `terminal/*` until implemented).
4. Re-measure permission event fidelity vs stream-json `--permission-prompt-tool stdio` / control protocol on Windows.
5. Optional: spawn live `claude … --output-format stream-json` from the adapter (process lifecycle + kill) — parser is ready; process supervisor is post-A1 polish.

---

## 5. Engineering follow-through (this turn)

- Crate: `crates/tsukumo-adapters`
- Default path: Claude-like stream-json subset parser + synthetic demo producer
- Wire: adapter → `KernelEvent` → Director → `StageWorld` (integration test + example)
- Briefing: empty `PromptAssembly` / `BriefingSource` hook for Phase R
