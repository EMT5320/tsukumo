# Research: Pixel / Viz Companion Products Deep Dive

- **Query**: Harness depth, memory/persistence, permission UX, runtime lock-in for Pixel Agents and peers; what Tsukumo should not copy vs table stakes
- **Scope**: mixed (marketplace, GitHub READMEs, Fast Company); align `DESIGN.md` §1.2–1.5, §6–7
- **Date**: 2026-07-09
- **Confidence**: high for Pixel Agents / AgentRoom / The Office / Agent Cockpit primary docs; medium for roadmap claims

## Findings

### Files Found

| File Path | Description |
|---|---|
| `DESIGN.md` §1.2 | Pixel Agents cited as validated ambient attention product |
| `DESIGN.md` §1.5 / §6 | Anti-Clippy + state/narrative separation (stage as lossy sampler) |
| `DESIGN.md` §7 | Tsukumo theater (TUI HalfBlock) — different surface, same spectacle layer |

### Product deep dives

#### 1. Pixel Agents (category leader)

**Sources:** [VS Marketplace](https://marketplace.visualstudio.com/items?itemName=pablodelucca.pixel-agents) (~74k installs on fetch date — **higher than DESIGN.md's 1.3万 snapshot**; treat as moving), [GitHub](https://github.com/pixel-agents-hq/pixel-agents), [Fast Company](https://www.fastcompany.com/91497413/this-charming-pixel-art-game-solves-one-of-ai-codings-most-annoying-ux-problems).

| Dimension | Status |
|---|---|
| **Harness depth** | **Thin / observational.** Watches Claude Code JSONL transcripts; maps tool events → character FSM (idle/walk/type/read). Explicit: "No modifications to Claude Code — purely observational." HookProvider architecture planned for multi-agent; Claude Code is reference impl. |
| **Memory / persistence** | Office layout + agent↔terminal mappings in VS Code `workspaceState`. **No** episodic memory, skills, bonds, or user model. |
| **Permission / approval UX** | Speech bubbles when waiting/needs permission (heuristic). Spawn option `--dangerously-skip-permissions`. **Does not own** the approval channel — Claude Code still prompts in terminal. Known limitation: waiting/finished detection is heuristic and misfires. |
| **Runtime lock-in** | **Claude Code only in shipping product** (user note confirmed). Roadmap: Codex, Gemini, Cursor, etc. Standalone `npx pixel-agents` Fastify SPA for non-VS Code. |
| **Owns session?** | No — can spawn terminal + character; does not assemble prompts or inject soul. |

**Vision creep (README "Where This Is Going"):** Sims-like management, desks-as-directories, wall Kanban, interrupt/chat/redirect, token health bars, 3D/VR. This is **aspirational orchestration HQ**, not shipped harness depth.

#### 2. Agent Cockpit (ops-deepest viz peer)

**Sources:** [agent-cockpit.dev](https://agent-cockpit.dev/), [README](https://raw.githubusercontent.com/agent-cockpit/agent-cockpit/main/README.md).

| Dimension | Status |
|---|---|
| **Harness depth** | **Medium.** Local Node daemon + provider adapters. Claude Code via **HTTP lifecycle hooks**; Codex via app-server/CLI. Normalizes events to SQLite + WebSocket. Not a coding agent; "control layer above them." |
| **Memory / persistence** | SQLite event store; **Memory panel** edits provider-specific project memory files in one UI. Session history/compare. Not cross-runtime soul or bonds. |
| **Permission / approval UX** | **Unified approval inbox** with risk classification; diffs before write; chat/terminate for daemon-launched sessions. Externally attached sessions often **approval-only**. |
| **Runtime lock-in** | Claude Code + Codex (two-vendor). Plugin SDK for more providers on roadmap. |
| **Owns session?** | **Partial/yes** when launched from Cockpit; stronger than Pixel Agents. |

**Collision:** Highest among viz peers for "command center" — still missing Tsukumo's Process Fidelity bonds + runtime-agnostic companion kernel.

#### 3. AgentRoom

**Sources:** [liuyixin-louis/agentroom README](https://raw.githubusercontent.com/liuyixin-louis/agentroom/main/README.md).

| Dimension | Status |
|---|---|
| **Harness depth** | Observational JSONL watcher (Rust `notify`) → Tauri events → Canvas office. Ports Pixel Agents game engine lineage. |
| **Memory / persistence** | Per-project layouts; **CASS** full-text/semantic search across 11+ agent session stores; transcript viewer; AI session tagging. Search ≠ growing soul. |
| **Permission UX** | Speech bubbles + sound; "Open in iTerm2" to resume — approvals stay in agent CLI. |
| **Runtime lock-in** | **Multi-watch** Claude Code, Codex, Gemini simultaneously (stronger than Pixel Agents shipping). |
| **Owns session?** | No (observe + jump to terminal). |

#### 4. The Office (kevanwee/theoffice)

| Dimension | Status |
|---|---|
| **Harness depth** | Pixel Agents fork; Pokémon ThemePack; JSONL / Copilot lm events / Codex events. |
| **Memory / persistence** | Layout `~/.the-office/layout.json` only. |
| **Permission UX** | Bubbles + chime; same observational limits. |
| **Runtime lock-in** | Claude Code + Copilot + Codex watchers. |
| **Owns session?** | No. |
| **Extra risk** | Nintendo IP fan assets — distribution/legal caution for any "copy the Pokémon angle." |

#### 5. Other peers (brief)

| Product | Harness takeaway |
|---|---|
| **shahar061/the-office** | Electron; Claude+OpenCode; optional in-app chat (slightly thicker). |
| **KalebKE/PixelOffice** | Claude Code viz + typed activity state machine; educational/demo tone. |
| **harishkotra/agent-office** | Different category: local LLM sim with SQLite memory — not coding-agent companion. |

### Cross-cutting: harness depth ladder

```
Thin observe (JSONL → sprites)     Pixel Agents, The Office, AgentRoom
         ↓
Observe + search/history           AgentRoom + CASS
         ↓
Launch + hooks + approvals + chat  Agent Cockpit (daemon-launched)
         ↓
Own session + inject soul + bonds  Tsukumo target (DESIGN §2) — not occupied by viz peers
         ↓
Full self-evolving coding agent    Hermes (different product shape)
```

### What Tsukumo should NOT copy

1. **JSONL-only forever as the primary architecture** — Pixel Agents' own known limitations (desync, heuristic waiting) prove observation cannot unlock injection/approvals/bonds (`DESIGN.md` §2.1 rejects pure observer skin).  
2. **Spectacle arms race** (Pokémon catalog, furniture marketplace, 3D/VR) — already commodity; violates weight flip in §1.3 (spectacle = entry ticket only).  
3. **Fake RPG stats / token health bars as "growth"** without Process Fidelity (§10.3) — Direct State Setter trap.  
4. **IDE-panel-only as the product body** — Tsukumo needs owned screen (guild hall); parasitizing VS Code panel repeats "enhancement layer" fatal wound (§2.1).  
5. **`--dangerously-skip-permissions` as default UX pattern** — anti-pattern vs approval-as-theater (§20 abstract→narrative table).  
6. **Shipping "agent-agnostic" marketing while Claude-only** — user already felt lock-in; A1 must be honest about adapter maturity.

### What is table stakes (copy the job-to-be-done, not the skin)

| Table stake | Why |
|---|---|
| **Ambient blocked-agent signal** | Fast Company / all peers: waiting unnoticed is the pain |
| **One glance multi-agent spatial map** | Parallel agents → attention management |
| **Sound/chime on turn complete or need input** | Cheap, effective |
| **Sub-agent visible as linked character** | Matches real Claude Task tool behavior |
| **Layout persistence** | Users customize once |
| **Graceful degrade when signals are heuristic** | Document misfires; don't pretend perfect state |
| **Path to multi-runtime** | AgentRoom/Cockpit already multi; Claude-only is behind |

### Implications for Tsukumo theater (`DESIGN.md` §6–7)

- Peers validate **ambient information** over performance theater.  
- Tsukumo's differentiator is not better sprites — it is **KernelEvent ownership** so the same stage can later do approval gates + memory injection.  
- MVP spectacle can match table stakes; **do not** delay A1 to polish furniture editors.

## Caveats / Not Found

- Install counts diverge (DESIGN 1.3万 vs marketplace ~74k on 2026-07-09) — cite date when using numbers.  
- Did not reverse-engineer private hook payloads for Agent Cockpit; depth claims from README architecture section.  
- Pixel Agents roadmap may ship multi-runtime adapters after this research; re-check before locking differentiation narrative.
