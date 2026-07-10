# Research: Competitor Matrix (Tsukumo-adjacent, mid-2026)

- **Query**: Structured comparison of self-evolving harnesses, viz companions, orchestration HQ, memory/soul layers, ACP clients vs Tsukumo
- **Scope**: mixed (GitHub READMEs, official sites, marketplace pages; local `DESIGN.md` §1.2, §20)
- **Date**: 2026-07-09

## Findings

### Legend

| Column | Meaning |
|---|---|
| **Owns session?** | Can product create/drive prompts, approvals, inject context? yes / no / partial |
| **Runtime binding** | single vendor / multi / ACP (or ACP-capable) |
| **Growth mechanisms** | none / memory file / skills / bonds / GEPA-like |
| **Collision** | high / med / low with Tsukumo positioning (`DESIGN.md` §1–2) |
| **Conf.** | H = primary README/site; M = secondary roundup + partial primary; L = thin/uncertain |

### Comparison matrix

| Product/project | Category | Owns session? | Runtime binding | Growth mechanisms | UI surface | Collision | Why | Conf. |
|---|---|---|---|---|---|---|---|---|
| **Pixel Agents** ([marketplace](https://marketplace.visualstudio.com/items?itemName=pablodelucca.pixel-agents), [pixel-agents-hq/pixel-agents](https://github.com/pixel-agents-hq/pixel-agents)) | viz companion | no (observational JSONL) | single today (Claude Code); roadmap multi via HookProvider | none (layout persistence only) | IDE plugin + CLI SPA | **high** (spectacle/attention entry ticket) | Same ambient-office UX thesis; thin harness; vision roadmap overlaps "orchestrate any agent" | H |
| **Agent Cockpit** ([agent-cockpit.dev](https://agent-cockpit.dev/), [agent-cockpit/agent-cockpit](https://github.com/agent-cockpit/agent-cockpit)) | viz companion + ops HQ | **partial→yes** for daemon-launched (hooks, chat, approvals, terminate); external attach approval-only | multi (Claude Code + Codex) | memory **panel** edits provider files; not bonds/GEPA | GUI (browser localhost) | **high** | Closest "control room + pixel office"; owns more session than Pixel Agents; still not runtime-agnostic soul/bonds | H |
| **The Office** ([kevanwee/theoffice](https://github.com/kevanwee/theoffice)) | viz companion (Pokémon skin) | no (JSONL / VS Code lm events) | multi watch (Claude Code, Copilot, Codex) | none (layout `~/.the-office/`) | IDE plugin | **med** | Forked Pixel Agents engine; spectacle commodity; IP/theme risk; no soul layer | H |
| **AgentRoom** ([liuyixin-louis/agentroom](https://github.com/liuyixin-louis/agentroom)) | viz companion + session search | no (JSONL watcher); "Open in Terminal" resume | multi watch (Claude/Codex/Gemini + CASS index of 11+) | none for agent growth; strong **episodic search** | GUI (Tauri desktop) | **med–high** | Multi-runtime observation + CASS search is table-stakes adjacent; still read-only soul | H |
| **shahar061/the-office** | viz companion | partial (chat to Claude/OpenCode from app) | multi watch (Claude + OpenCode) | none | GUI (Electron) | **med** | Role-cast characters; still transcript-driven | M |
| **harishkotra/agent-office** | sim / other (local LLM office) | yes (owns agents) | single stack (Ollama-centric) | memory (SQLite + embeddings); hiring loop | GUI | **low** | Toy autonomous office, not coding-agent HQ | H |
| **Vibe Kanban** ([BloopAI/vibe-kanban](https://github.com/BloopAI/vibe-kanban)) | orchestration HQ | yes (workspaces, assign agents, PR flow) | multi (10+ coding agents) | none (task board, not soul) | GUI (web); **sunsetting** noted on README | **med** | Command-center neighbor; utility not relationship; project status risk | H |
| **Conductor** ([conductor.build](https://conductor.build), Melty Labs) | orchestration HQ | yes (worktrees, diff review) | multi (Claude Code, Codex) | none | GUI (macOS) | **med** | Parallel agent ops; no bonds/memory growth | M |
| **Claude Squad** ([smtg-ai/claude-squad](https://github.com/smtg-ai/claude-squad)) | orchestration HQ | yes (tmux + worktrees launch) | multi CLI | none | TUI | **low–med** | Session manager only; no theater/soul | H |
| **hermes-agent** ([NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent)) | self-evolving harness | yes | multi provider; ACP agent mode **requested** (#569), not assumed shipped | memory files + skills + Curator + optional GEPA | TUI + messaging gateways | **high** (growth philosophy) | Direct inspiration; competes if user wants one growing agent vs HQ+runtimes | H |
| **hermes-agent-self-evolution** | self-evolving harness (offline) | n/a (pipeline) | Hermes-centric | GEPA-like skill/prompt evolution | none (PR workflow) | **med** | Post-MVP pattern Tsukumo already reserved (`DESIGN.md` §10.6) | H |
| **GEPA / gskill / optimize_anything** ([gepa-ai/gepa](https://github.com/gepa-ai/gepa)) | self-evolving harness (library) | n/a | agent-agnostic optimizer | GEPA-like skills/prompts/code | none | **low** as product; **med** as capability commodity | Library everyone can bolt on; not a companion HQ | H |
| **pi / pi-mono** ([badlogic/pi-mono](https://github.com/badlogic/pi-mono), [earendil-works/pi](https://github.com/earendil-works/pi)) | coding agent harness | yes | multi provider; **Pi listed on Zed ACP agents** | skills (agentskills.io); extensibility not GEPA-default | TUI | **med** | Architectural inspiration (`DESIGN.md` §20); competes as runtime, not as soul HQ | H |
| **AgentSoul** ([agentsoul.market](https://agentsoul.market/en/books/)) | agent soul / companion (static) | no | multi (export Markdown into harnesses) | none (generated identity files) | web forge | **med** | Commoditizes **static** soul packs; empty on Process Fidelity / bonds-from-experience | H |
| **EvoClaw** ([slhleosun/EvoClaw](https://github.com/slhleosun/EvoClaw)) | agent soul evolution | partial (OpenClaw workspace) | OpenClaw-centric | SOUL reflection + governed updates + tiered memory | local web UI | **med** | Evolving identity exists; not multi-runtime coding HQ + theater | H |
| **Soul Protocol** ([qbtrix/soul-protocol](https://github.com/qbtrix/soul-protocol)) | memory / soul middleware | partial via MCP tools | multi via CLI/MCP/prompt inject | observe/reflect/evolve tools | none / MCP | **med** | Could become pluggable soul backend; overlaps §8 injection story | M |
| **OpenAI Self-Evolving Cookbook** | framework / pattern | n/a | OpenAI-centric examples | prompt opt / GEPA path | notebook | **low** | Pattern reference, not product | H |
| **MemSkill** (arXiv:2602.02474) | academic memory evolution | n/a | research | evolving memory skills | none | **low** | Post-MVP research neighbor | H |
| **Toad** (batrachianai/toad, via Hermes #569) | ACP ecosystem client (TUI) | yes (ACP client) | ACP multi-agent | none inherent | TUI | **med** | Proves ACP client can be the interaction surface; Tsukumo wants same + soul theater | M |
| **Zed / JetBrains / Neovim ACP clients** | ACP ecosystem clients | yes | ACP | none (editor features) | IDE | **low–med** | Own sessions for coding; not companion/guild hall | H |
| **Munder Difflin** (roundup blogs) | orchestration + shared memory hive | yes (claimed) | multi | shared memory (MemPalace) | varies | **med** | Coordination+memory without RPG bonds; verify primary before betting | L |

### Cluster summary

```
Spectacle / attention (commodity):     Pixel Agents, The Office, AgentRoom, Agent Cockpit office skin
Ops / parallel HQ (utility):           Vibe Kanban, Conductor, Claude Squad, Cockpit ops plane
Growing agent (full harness):          Hermes (+ GEPA offline)
Optimizer libraries:                   GEPA, gskill, DSPy
Static/portable soul:                  AgentSoul, OpenClaw SOUL.md culture
Evolving identity (narrow runtime):    EvoClaw, Soul Protocol
ACP session owners (editors/TUIs):     Zed, JetBrains, Toad, …
Claimed empty (Tsukumo):               runtime-agnostic soul + bonds/Process Fidelity + owned guild hall
```

### Related Specs / Design

- `DESIGN.md` §1.2 — market update: pixel viz validated; soul persistence empty  
- `DESIGN.md` §2 — fourth form: own UI + companion kernel + pluggable runtimes  
- `DESIGN.md` §20 — competitor shortlist matches this matrix core set

## Caveats / Not Found

- **Vibe Kanban sunsetting** (README banner): collision may shrink; do not plan as durable competitor.  
- Multiple GitHub projects named "the-office" / "agent-cockpit" — matrix uses the primary sources linked above; `daronyondem/agent-cockpit` and `cathyzhang0905/agent-cockpit` are **different** products.  
- Conductor / Munder Difflin details partly from 2026 roundup blogs — re-verify before strategic decisions.  
- ACP "19+ agents" on `DESIGN.md` is consistent with Zed ACP page listing dozens of agents (many via adapters); **native vs bridge** differs per agent (see `runtime-interop-acp.md`).  
- Star counts (Hermes ~187k on issue page metadata, pi ~69k) fluctuate; use for order-of-magnitude only.
