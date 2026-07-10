# Research: Self-Evolving Agent Harness Landscape (2025–2026)

- **Query**: Map self-evolving agent harnesses — GEPA/gskill/optimize_anything, Hermes-style loops, OpenAI cookbook, DSPy+GEPA, MemSkill/EvoSkills, what "self-evolution" means in practice, failure modes
- **Scope**: mixed (external primary sources + alignment to local `DESIGN.md` §10, §20)
- **Date**: 2026-07-09
- **Confidence**: high on GEPA/Hermes/OpenAI cookbook primary docs; medium on secondary blog synthesis; low on "EvoSkills" as a single canonical product (name collision / sparse primary hit)

## Findings

### Files Found (local alignment)

| File Path | Description |
|---|---|
| `DESIGN.md` §10.6 / §16 | Tsukumo MVP leaves GEPA as trait interface only |
| `DESIGN.md` §20 | Explicit refs: hermes-agent, hermes-agent-self-evolution, pi, ACP |
| `DESIGN.md` §10.2–10.3 | Hermes absorption table + Process Fidelity principle |

### What "self-evolution" usually means in practice

In 2025–2026 shipping systems, "self-evolving" almost never means weight fine-tuning by default. It usually means **textual artifact evolution** under a propose → evaluate → gate loop:

| Mechanism | Artifact mutated | Typical signal | Example systems |
|---|---|---|---|
| **Prompt optimization** | system/module instructions | scalar metric + textual feedback (ASI) | GEPA, DSPy+GEPA, OpenAI cookbook metaprompt/GEPA |
| **Skill invent / patch** | `SKILL.md` (+ optional scripts) | task success, user correction, curator heuristics | Hermes `skill_manage`, gskill, agentskills.io ecosystem |
| **Memory curation** | MEMORY.md / USER.md / episodic DB | capacity caps, contradiction, idle prune | Hermes Curator, MemSkill (academic) |
| **Tool / architecture synthesis** | agent code, tool wrappers, policies | verifiable eval harness | `optimize_anything` (beyond prompts), AlphaEvolve-class systems |
| **Identity / soul file evolution** | SOUL.md / identity bundles | reflection proposals + human governance | EvoClaw, Soul Protocol, AgentSoul (portable identity, not coding harness) |

**Practical definition (2026):** self-evolution = *offline or background optimization of prompts/skills/memory policies from execution traces*, with human or automated gates — not "the agent becomes a new model overnight."

### GEPA / optimize_anything / gskill

**GEPA (Genetic-Pareto Reflective Prompt Evolution)**  
- Paper: Agrawal et al., arXiv:2507.19457 (2025); integrated as `dspy.GEPA` and standalone [`gepa-ai/gepa`](https://github.com/gepa-ai/gepa).  
- Core idea: evaluators return **Actionable Side Information (ASI)** (errors, traces, logs), not only scalar rewards; LLM reflects and mutates text; Pareto frontier retains specialists across instances.  
- Claimed niche: expensive rollouts, scarce data, interpretable optimization traces — complements RL/fine-tuning rather than replacing them.

**`optimize_anything` (GEPA blog, 2026-02-18)**  
- Extends GEPA beyond prompts to **any text artifact** (code, agent architectures, configs, skills).  
- Three modes: single-task search, multi-task search, **generalization** (train + held-out valset).  
- Positions against AlphaEvolve / OpenEvolve / ShinkaEvolve by simplifying the API while adding generalization mode.

**gskill**  
- Pipeline: [SWE-smith](https://swesmith.com) generates verifiable repo tasks → GEPA `optimize_anything` evolves skill text → deploy `best_skills.txt` into agent prompts.  
- Explicit thesis: **repo-specific skills** that transfer across models/agents (train cheap, deploy expensive).  
- Docs: [`gepa-ai/gepa` gskill guide](https://github.com/gepa-ai/gepa/blob/main/docs/docs/guides/gskill.md); blog [Automatically Learning Skills for Coding Agents](https://gepa-ai.github.io/gepa/blog/2026/02/18/automatically-learning-skills-for-coding-agents/).

**Academic/product descendants / integrations (non-exhaustive)**  
- DSPy first-class GEPA optimizer ([dspy.ai GEPA overview](https://dspy.ai/api/optimizers/GEPA/overview/))  
- OpenAI Cookbook self-evolving agents (GEPA as advanced path)  
- Comet Opik Agent Optimizer (GEPA algorithm)  
- PydanticAI / Google ADK prompt optimization mentions on GEPA site  
- Hermes offline pipeline: [`NousResearch/hermes-agent-self-evolution`](https://github.com/NousResearch/hermes-agent-self-evolution) (DSPy + GEPA; ~4.5k★ as of research date)

### Hermes-agent style memory + skill + curator loops

Primary: [`NousResearch/hermes-agent`](https://github.com/NousResearch/hermes-agent), docs at hermes-agent.nousresearch.com / Mintlify intro.

**Closed learning loop (product claim):**  
1. **Episodic**: SQLite + FTS5 session search, LLM summarization for cross-session recall  
2. **Persistent identity/facts**: MEMORY.md + USER.md (capacity-capped snapshots injected each session); SOUL.md persona in community writeups  
3. **Procedural**: agentskills.io-compatible `SKILL.md`; agent creates/patches via `skill_manage` / `skill_create` after complex tasks  
4. **Curator**: background maintenance on **agent-created** skills only — idle→stale→archive (never hard-delete by default); optional LLM consolidate; pin/exempt; cron-like interval  
5. **Offline GEPA**: separate repo validates/evolves skills from traces with test/size/semantic gates + PR review (human-in-the-loop)

**What Hermes optimizes for:** daily personal agent UX (multi-channel gateway, persistent "grows with you"), not SWE-bench leaderboard alone. Provider-agnostic discipline matches Tsukumo inspiration (`DESIGN.md` §1.1, §20).

**Collision note for Tsukumo:** Hermes is a **full coding/personal agent harness** with soul-like persistence. Tsukumo claims a **relationship layer + HQ** that outsources heavy coding to external runtimes (`DESIGN.md` §2). Overlap is philosophical (memory/skills/curator), not product surface (Hermes owns the agent; Tsukumo wants to own the session/UI and grow a companion soul across runtimes).

### OpenAI cookbook: self-evolving agents

Primary: [Self-Evolving Agents - A Cookbook for Autonomous Agent Retraining](https://cookbook.openai.com/examples/partners/self_evolving_agents/autonomous_agent_retraining) (dated Nov 4, 2025 on page).

- Domain framing: regulated healthcare documentation (accuracy + auditability).  
- Loop: baseline agent → human / LLM-as-judge feedback → metaprompt rewriting → evals/graders → promote improved prompts.  
- Compares manual iteration vs automated loops; GEPA called out on GEPA site as cookbook advanced path.  
- **Optimizes for:** production readiness, measurable graders, audit trail — **not** ambient coding UX or companion attachment.

### DSPy + GEPA, MemSkill, and other 2025–2026 systems

| System | Layer | Notes |
|---|---|---|
| **DSPy + GEPA** | Prompt/module compile | Industry-standard declarative signatures + reflective evolution; Hermes self-evolution and many cookbooks sit here |
| **MemSkill** | Memory operations as skills | arXiv:2602.02474; learns/evolves extract/consolidate/prune skills via controller + designer on hard cases; evals on LoCoMo, LongMemEval, etc. Code: `ViktorAxelsen/MemSkill` |
| **gskill** | Coding skill files | Repo-grounded, verifiable tasks |
| **Memento-Skills / SkillWeaver** (secondary mentions) | Skill library rewrite / web skills | Appear in survey blogs; treat as adjacent research, verify before citing as products |
| **EvoSkills** | — | **No single dominant primary product found under this exact name** in this pass; do not treat as a settled market category. Nearby: EvoClaw (SOUL evolution), gskill, MemSkill |
| **TextGrad / AgentScope / Karpathy autoresearch** | Broader self-improve | Prompt gradients, production fine-tune loops, overnight code rewrite — adjacent, heavier than Tsukumo MVP |

### Soul / identity adjacent (not coding harnesses, but "agent soul" market)

| Project | What it is | Relevance |
|---|---|---|
| **AgentSoul** (agentsoul.market) | Portable Markdown identity bundle (soul/identity/user/agents) for Claude Code / Hermes / Cursor / OpenClaw | Commodity **static** soul files — not Process Fidelity growth |
| **EvoClaw** | Experience → reflect → governed SOUL.md proposals for OpenClaw | Closest to "soul evolves," but OpenClaw-centric, not multi-runtime HQ |
| **Soul Protocol** | MCP + `.soul` files, observe/recall/reflect tools | Memory/identity middleware; collision with Tsukumo soul layer if it becomes runtime-agnostic HQ |

### Common failure modes

| Failure mode | Why it happens | What systems optimize instead |
|---|---|---|
| **Self-congratulation / reward hacking** | In-agent skill_edit without offline eval | Hermes docs warn; GEPA offline + tests/PR gates |
| **Benchmark overfitting** | Optimize for SWE-bench / synthetic tasks | gskill uses held-out valset; still may miss daily UX |
| **Skill library rot** | Unbounded create, contradictions, token bloat | Curator prune/archive; capacity headers on MEMORY.md |
| **Semantic drift of persona** | Unconstrained SOUL/prompt mutation | EvoClaw CORE immutability; Hermes GEPA size/semantic guards |
| **Cost blowups** | Reflection models + parallel Docker evals | GEPA marketed for few rollouts; still $2–10+/run class for Hermes GEPA anecdotes |
| **Observability ≠ ownership** | Viz products watch transcripts but cannot inject/curate | Pixel Agents class (see `pixel-viz-companions.md`) |
| **Process Infidelity** | Fake RPG stats / Direct State Setter | Tsukumo `DESIGN.md` §10.3 / Loomstead Process Fidelity — market mostly ignores this |

### What they optimize for (benchmarks vs daily coding UX)

| Camp | Optimize for | Representative |
|---|---|---|
| **Eval / research harness** | Pass@k, ASI quality, transfer to held-out tasks | GEPA, gskill, MemSkill papers, OpenAI cookbook graders |
| **Personal agent retention** | Cross-session memory, skill accumulation, multi-channel presence | Hermes |
| **Attention / ops UX** | "Which agent is blocked?", approvals, diffs | Pixel Agents, Agent Cockpit, Vibe Kanban, Conductor |
| **Identity portability** | Same persona across tools | AgentSoul, SoulSpec/OpenClaw files |
| **Tsukumo bet (claimed empty)** | Runtime-agnostic **relationship** + Process Fidelity stats + owned session | `DESIGN.md` §1.2, §2 — still sparsely occupied as of this research |

## Caveats / Not Found

- Install counts and "19+ ACP agents" move weekly; treat ecosystem size as **order-of-magnitude**, re-check at A1 spike time.  
- "EvoSkills" was requested by name; **no authoritative single project** confirmed — use MemSkill/gskill/EvoClaw as nearest neighbors.  
- Secondary LinkedIn/DEV posts sometimes overstate "Hermes = DSPy+GEPA by default"; primary Hermes product loop is skill_manage + Curator; GEPA is a **separate offline pipeline**.  
- Did not re-run GEPA paper experiments; performance claims cited from authors' materials only.
