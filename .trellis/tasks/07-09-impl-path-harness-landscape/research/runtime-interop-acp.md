# Research: ACP and Runtime Interop for Session-Owning Clients

- **Query**: ACP client responsibilities vs thin event consumers; Claude Code / Codex / Gemini CLI maturity; stream-json / transcript watcher fallbacks; implications for Tsukumo A1
- **Scope**: mixed (Zed ACP pages, adapter repos, Hermes #569 protocol notes, `DESIGN.md` §2.3, §8, §15, §19)
- **Date**: 2026-07-09
- **Confidence**: high on protocol shape and Claude-via-adapter pattern; medium on per-vendor "native" claims (moving target)

## Findings

### Local alignment

| Design section | Claim |
|---|---|
| `DESIGN.md` §2.1–2.3 | Tsukumo as ACP **client** owns session; three tiers: drive / observe / absent |
| `DESIGN.md` §8 | Soul injection prefers prompt assembly (client-owned) over file turf wars |
| `DESIGN.md` §15 A1 | First adapter → KernelEvent → stage; injection full protocol post-MVP |
| `DESIGN.md` §19 | Open: ACP vs headless stream-json for Claude Code depending on maturity |

### ACP in one paragraph

**Agent Client Protocol** (Zed-origin, JetBrains co-maintain, Apache-2.0): JSON-RPC 2.0 over stdio between an **editor/client** and a **coding agent** subprocess. Analogous to LSP for agents. Site: [agentclientprotocol.com](https://agentclientprotocol.com/); marketing hub: [zed.dev/acp](https://zed.dev/acp).

### ACP client responsibilities vs thin event consumers

From protocol notes in [Hermes issue #569](https://github.com/NousResearch/hermes-agent/issues/569) (Mar 2026 research dump) and Morph/Zed secondary explainers:

#### What a full ACP **client** must provide

| Client capability | Methods / behavior | Why it matters for Tsukumo |
|---|---|---|
| Session lifecycle | `initialize`, `session/new`, `session/prompt`, `session/cancel`, optional `session/load` | **Owns** conversation; can inject briefings into prompts (§8.3 priority #1) |
| Consume streamed updates | Handle `session/update` (message/thought chunks, tool_call, plan, …) | Maps cleanly to `KernelEvent` |
| **Permissions** | Respond to `session/request_permission` | Approval theater / confirmation gates (§5.4, §20) |
| **Filesystem** (if advertised) | `fs/read_text_file`, `fs/write_text_file` | Editor unsaved buffer fidelity; client is source of truth for files |
| **Terminal** (if advertised) | `terminal/create`, `output`, `wait_for_exit`, `kill`, `release` | Client-hosted PTY; agent does not own the glass |
| MCP pass-through | `session/new` may carry MCP server configs | Soul-as-MCP memory service later (§8 / §19) |

**Implication:** An ACP client is **not** a thin spectator. It is the host runtime for tools/permissions UX. Tsukumo's "guild hall owns the screen" thesis matches this role.

#### What a thin event consumer does instead

| Pattern | Examples | Capabilities |
|---|---|---|
| Transcript / JSONL watcher | Pixel Agents, AgentRoom, The Office | Animate from tool_use lines; **no** prompt inject, unreliable wait detection |
| Hook sidecar | Agent Cockpit Claude hooks | Richer events + approvals if hooks installed; still provider-specific |
| Headless `stream-json` pipe | Claude `--output-format stream-json`, Codex JSONL | Structured events without full ACP; client must own process lifecycle |

Thin consumers implement Tsukumo's **观察级** tier (`DESIGN.md` §2.3). They cannot grow soul honestly.

### Agent maturity: native vs bridge (as of research)

| Agent | ACP posture | Notes | Conf. |
|---|---|---|---|
| **Gemini CLI** | **Native / reference** | First external integration with Zed (Aug 2025); `gemini --experimental-acp` cited in Hermes #569 | H |
| **Claude Code / Claude Agent** | **Via adapter (bridge)** | Zed `@zed-industries/claude-code-acp` / `claude-agent-acp` wraps Claude Agent SDK; community Go/TS bridges also exist. Anthropic not described as first-party ACP server in sources reviewed. | H |
| **Codex CLI** | **Adapter / live in Zed** | Zed blog "Codex is Live in Zed" (Oct 2025); `@zed-industries/codex-acp` NPX adapter cited | H |
| **Goose** | Native (`goose acp`) | Block | M |
| **GitHub Copilot** | Native preview (`copilot --acp`) claimed Jan 2026 (secondary) | Re-verify | M |
| **OpenCode, Amp, Cline, Pi, …** | Listed on [zed.dev/acp](https://zed.dev/acp) Agents registry | Mix of native and adapter; **Pi appears on ACP agent list** | M |
| **Hermes** | ACP **agent** mode proposed (#569 open as of Apr 2026 update) | Would make Hermes a pluggable runtime under Tsukumo — strategic upside | M |

**Critical nuance for A1:** "Claude Code supports ACP" in marketing usually means **adapter process speaks ACP to the client**, while Claude underneath may still be SDK/`stream-json`. Tsukumo must budget for **adapter quirks** (session id mapping, permission mode env vars, tool category fidelity), not assume Anthropic-native ACP.

### Headless stream-json / transcript watcher as fallback

| Fallback | How it works | Drive-tier? | Soul inject? | Approval own? |
|---|---|---|---|---|
| **Claude `stream-json`** | Spawn `claude` with JSON event stream; translate to KernelEvent | **Partial–yes** if Tsukumo owns the process and stdin | Yes (prompt args / append system) | Only if hooked or permission mode controlled by parent |
| **Codex session JSONL** | Tail `~/.codex/sessions/...` | Observe unless Tsukumo launched process | Weak | Weak |
| **Claude project JSONL** | Tail `~/.claude/projects/...` | Observe (Pixel Agents path) | No | No |
| **Provider hooks** (Cockpit-style) | HTTP/lifecycle hooks installed at launch | Partial drive | Possible via launch config | Yes if hook surfaces approvals |

`DESIGN.md` §2.2 already diagrams: ACP adapter **and** transcript watcher as producers of the same KernelEvent bus — correct.

### Implications for Tsukumo A1 spike (`DESIGN.md` §15)

**A1 goal (design):** first runtime adapter → KernelEvent → drive stage. **Not** full §8 injection.

#### Recommended falsification order (interop-specific)

1. **Prove KernelEvent from a structured stream end-to-end** (even if stream-json, not ACP) — falsifies "external events can drive theater."  
2. **Measure signal quality:** tool start/end, waiting-for-permission, turn complete — compare ACP `session/update` + `request_permission` vs JSONL heuristics (Pixel Agents failure modes).  
3. **Only then** invest in full ACP client (fs/terminal capability negotiation) if permission theater is required for M2+ narrative.  
4. Keep **观察级 JSONL watcher** as onboarding path (matches §2.3), but do not call it drive-tier.

#### A1 channel decision heuristic

| If at spike time… | Prefer |
|---|---|
| `claude-code-acp` / Agent SDK adapter stable on Windows (Tsukumo host is win32) | ACP client spike against adapter |
| ACP adapter flaky on Windows / permission mapping broken | **Own-process stream-json** → KernelEvent; document ACP as A1.1 |
| Only need stage demo | JSONL watcher acceptable for **S/M theater**, but **fails** the "终态最大技术假设" if treated as success for drive-tier |

#### Client responsibility checklist (when going ACP)

- [ ] Advertise only capabilities Tsukumo TUI can honor (don't claim `fs/*` until implemented)  
- [ ] Map `request_permission` → StageEvent confirmation gate (even stub UI)  
- [ ] Session prompt assembly site ready for later memory briefing (§8.3)  
- [ ] Normalize vendor tool names → KernelEvent vocabulary once  
- [ ] Accept "one adapter always broken" tax (§2.6)

### Related ecosystem clients (session owners, not souls)

- **Zed / JetBrains**: reference ACP clients  
- **Toad**: multi-agent ACP TUI (proves terminal HQ via ACP)  
- **Neovim/Emacs plugins**: thinner clients  
Tsukumo would join this **client** class, differentiated by guild-hall + soul layer — not by inventing another agent binary.

## Caveats / Not Found

- Official `agentclientprotocol.com` overview fetch timed out during this research; protocol method list cross-checked via Hermes #569 + adapter READMEs — **re-fetch schema** before implementing.  
- "Native Claude ACP" rumors (Zed `crates/agent_servers` notes in community READMEs) may change maturity quickly — treat bridge-as-default until Anthropic docs say otherwise.  
- Windows PTY / node-pty issues noted by Agent Cockpit; Rust ACP client path may be healthier for Tsukumo stack but unproven here.  
- Did not run live ACP handshake tests in this research pass.
