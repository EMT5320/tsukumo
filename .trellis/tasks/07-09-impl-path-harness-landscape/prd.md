# 实现路径B与自进化harness竞品调研

## Goal

把 Tsukumo「有灵魂的指挥所」愿景落到可执行的实现路径：以 **B（A1 adapter 尽早验证）为主、A（舞台）并行瘦身**；同时摸清 2026 自进化 agent harness 研究/产品格局，明确撞车面与差异化护城河，作为后续 `design.md` / `implement.md` 的基础参考。

## Background

- 设计事实源：`DESIGN.md`（2026-07-08 合并终版）
- 已拍板形态：自有 TUI + 伴生灵魂宿主 + 外部 runtime 可驱动插拔（ACP 为主）
- 实现路径倾向（本轮确认）：
  - **B 为主**：尽早验证「外部 runtime 事件 → KernelEvent → 舞台/灵魂」
  - **A 并行瘦身**：舞台可用 fixture / stream-json 假事件先做 S0–S1；伴生内核不做完整 coding agent
  - 完整注入协议（§8）与成长闭环（M2+）仍 post-MVP，但需用竞品调研校准「护城河何时必须可见」
- 用户补充：自进化 harness 赛道很火，Tsukumo 某种程度上也是这类设计；Pixel Agents 实测只支持 Claude Code、harness 层简陋（像素前端为主）——多层融合仍是差异化，但需要系统调研防撞车

## Confirmed Facts

- 终态不是再做一个 coding agent，而是 runtime 无关的关系/灵魂层
- MVP 双线：舞台 S0–S1 + 内核 K0–K1 → M1；A1 adapter spike 尽早
- 化妆在出口（§8.1）：拟人化不注入 runtime 人格
- 养成数值真实性 / Process Fidelity：数值来自真实经历，非假进度条
- self-evolution（GEPA）MVP 只留接口
- **自进化主战场 = 关系自进化**（2026-07-09 拍板）：记忆 / user model / 羁绊 / 跨 runtime 人格连续；能力自进化（GEPA / skill 优化）不进 MVP 主叙事，只吸收标准件（SKILL.md、冻结快照），打出差异化后再充分吸收竞品能力层
- **关系层体感时机 = 中偏早**（2026-07-09 拍板）：A1 验证事件翻译时，旁路做最小关系探针（委托书简报 + 一次跨会话召回），证伪「换会话还认得主人」；完整 §8 / M2 成长闭环仍后置。机制形态对齐 Hermes 记忆沉淀，但遵守已有 token 预算纪律：
  - §8.4 **拉取优于推送**：容量封顶简报（冻结快照 + 相关性 top-k），长尾走 MCP/`recall`，禁止把编年史整库推进每次委托
  - §10.2 冻结快照 + FTS 召回；Skills 渐进披露（L0 目录 → L1 全文）
  - §5.3 消息双轨：演出/结算不进 LLM 上下文
  - 台词三级：模板为主，关键节点才 LLM——沉淀触发也同构，**禁止每 tool call 都跑提炼**
  - 调研印证（`research/tsukumo-differentiation-implications.md`）：F3 无注入时易沦为又一个 Pixel Agents；F5 跨 runtime 记忆是长期赌注；GEPA 级能力自进化保持商品/post-MVP
- **关系探针范围 = 记忆为主、skill 只留接口**（2026-07-09 拍板）：探针做 USER/MEMORY 简报 + 跨会话召回（+ 可选羁绊计数）；`skill_create` / SKILL.md 目录与 trait 预留，A1 旁路不露「领悟新技能」UI；完整 skill 自沉淀进 M2
- **A1 接入深度 = 双通道务实**（2026-07-09 拍板）：A1 成功标准 = 事件保真 + 等待/审批可演 + 委托书简报能进；优先用届时 Windows 上最稳的通道（ACP bridge 或 `stream-json`），**不做**完整 `fs/*`/`terminal/*` 代理；完整 ACP client 面后置，协议位预留。驱动级叙事诚实为「够演+够注入」，再补全编辑器式宿主职责

## Requirements

1. 完成自进化 agent harness 研究/产品格局调研，覆盖至少：能力自进化（skill/prompt/GEPA）、记忆自沉淀、编排指挥所、像素/可视化陪伴四类
2. 对每个关键竞品/项目给出：定位、能力层、runtime 绑定、是否拥有会话、成长机制深度、与 Tsukumo 的撞车面/错位面
3. 产出可指导实现的结论：MVP 必须先证伪的假设、可延后的能力、必须避开的同质化陷阱
4. 将调研落盘到本任务 `research/`，并据此收敛实现路径决策（写入后续 design/implement）

## Acceptance Criteria

- [x] `research/` 下至少有一份格局总览 + 若干专题（竞品矩阵 / 自进化机制 / 可视化陪伴 / ACP-runtime 互操作）— 2026-07-09 由 [自进化harness竞品调研](d6c99e13-9306-4092-bd94-35afd28e8913) 落盘 5 份
- [x] 明确写出 Tsukumo 相对「纯自进化 harness」与「纯像素观察器」的差异化主张（可被证伪）— 见 `research/tsukumo-differentiation-implications.md`
- [x] 实现路径 B 的验证顺序与风险清单写入规划产物 — 见 `design.md` §4–§5
- [x] 开放问题收敛到可拍板的决策点（一次一个）— 执行层 ontology 已显式延期；其余已拍板项见 `design.md` §2；P0/S/A1/R 工程切片已落地（check: MERGEABLE）

## Research Caveats（规划输入）

- Claude Code ACP 仍以 bridge/adapter 为主；A1 须在 Windows 上实测 ACP vs `stream-json`
- Pixel Agents 安装量已远高于 `DESIGN.md` 旧数字（调研抓取约 74k）——spectacle 比设计稿快照更挤
- Vibe Kanban 标注 sunsetting；勿当持久对标
- 对外少空喊 soul，强调 Process Fidelity / 跨 runtime 经历（静态 SOUL 包正在商品化）

## Out of Scope

- 本阶段不写生产代码、不 `task.py start`
- 不把 GEPA / Process Fidelity 评测搬进 MVP
- 不拍板主搭档人设、美术风格等叙事细节（可列开放问题）
- **不拍板执行层角色 ontology**（工具付丧神 / 雇佣兵 / 主搭档）；本轮仅记录初步讨论，见 `notes-executor-role-model.md`
- **根目录 `DESIGN.md` 只读**：项目愿景北极星，本任务及后续实现均不修改

## Open Questions

- **执行层角色模型（延期专项）**：主人初步倾向「付丧神与雇佣兵是同一执行层，仅 backend 不同；自有设定须能在第三方工具上生效；不宜两套成长账本」。三分法本轮视为臃肿未采纳。详见 `notes-executor-role-model.md`；实现软约束见 `design.md` §6.1。
- 规划产物已齐（prd / design / implement）；待主人审阅后决定是否 `task.py start` 或先开执行层专项讨论