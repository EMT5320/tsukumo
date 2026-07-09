# Research: Tsukumo 差异化与规划含义（综合）

- **Query**: 碰撞热点、市场空白层、MVP 证伪顺序（Path B）、商品化能力 vs post-MVP、需与主人讨论的开放风险
- **Scope**: mixed（本任务其余 research 文件 + `DESIGN.md` §1–2, §8, §15–20）
- **Date**: 2026-07-09
- **Confidence**: 综合判断 medium–high；竞品路线图会变，结论需随 A1 实测修正

## Findings

### 1. 碰撞最高的地方

| 碰撞带 | 强度 | 对手 | 说明 |
|---|---|---|---|
| **像素办公室 / 注意力可视化** | **高** | Pixel Agents、Agent Cockpit、AgentRoom、The Office | 入场券已被验证并商品化（`DESIGN.md` §1.2）；再卷家具/宝可梦皮肤是红海 |
| **多 agent 指挥所（编排/审批/diff）** | **高** | Agent Cockpit ops、Conductor、Vibe Kanban、Claude Squad | 「会长管并行任务」效用赛道拥挤；Cockpit 已做到 launch+审批+记忆面板 |
| **越用越懂你的成长 agent** | **高（哲学）/ 中（产品形）** | Hermes（+ GEPA 离线） | 记忆/技能/Curator 闭环是灵感源也是替代品：用户可能直接用 Hermes，而不要「HQ + 外挂 runtime」 |
| **可移植 soul 文件** | **中** | AgentSoul、OpenClaw SOUL.md 文化、Soul Protocol | 静态人格包正在商品化；**不是** Process Fidelity，但会稀释「有灵魂」话术 |
| **ACP 会话宿主** | **中** | Zed / JetBrains / Toad | 他们拥有会话但不做公会养成；Tsukumo 若只做薄 ACP 客户端会沦为又一个 Toad |
| **GEPA/技能自进化库** | **低（产品）/ 高（能力预期）** | gskill、DSPy+GEPA、cookbook | 能力会变标配；当作护城河会失望 |

**结论：** 单层产品（纯 spectacle / 纯编排 / 纯自进化 coding agent）都已有强玩家。Tsukumo 的赌约必须是 **多层融合**（主人原话）：公会大厅 × 跨 runtime 灵魂 × 真实经历数值 —— 任一单层都不够护城河。

### 2. 市场上仍相对空的 Tsukumo 层

对照 `DESIGN.md` §2 架构与竞品矩阵：

| Tsukumo 层 | 市场现状 | 空置程度 |
|---|---|---|
| TUI/GUI 像素舞台（spectacle） | 已挤满 | **空**（勿当护城河） |
| 多 runtime 观察 | AgentRoom / Cockpit 部分覆盖 | 薄空 |
| **拥有会话的非 IDE 指挥所 + 养成叙事** | Cockpit 接近 ops，缺养成；Hermes 有养成，缺「外挂雇佣兵」模型 | **较空** |
| **跨 runtime 同一份 canonical 记忆/技能/羁绊（§8 IR）** | 各家 MEMORY 互不相通；AgentSoul 只导出静态包 | **空** |
| **Process Fidelity：数值 = 真实事件溯源** | 几乎无人做（Loomstead 学术侧） | **很空** |
| 伴生内核「只养灵魂不卷编码能力」 | Hermes/pi 都在卷 agent 能力 | **空（定位空）** |
| GEPA 角色修行 | 库与 Hermes 离线管线已有 | 接口预留即可（§16） |

### 3. 推荐 MVP 证伪顺序（对 Path B 的支持与挑战）

**Path B（设计已写）：** 尽早验证 external runtime → KernelEvent → stage；伴生内核不做完整 coding agent（`DESIGN.md` §15 A1，§2.1）。

#### 建议证伪序（研究建议，非实现）

| 序 | 证伪问题 | 若失败意味着 | 与 Path B |
|---|---|---|---|
| **F0** | 像素舞台在目标终端上「愿意开着干活」？ | spectacle 入场券不成立 | 舞台线 S0/S1；不否定 Path B |
| **F1 ★** | 外部结构化事件能否稳定驱动演出（工具态/等待/结束）？ | 「指挥所」退化成壁纸；终态假设崩 | **支持 Path B：A1 必须早** |
| **F2** | 驱动级是否必须 ACP，还是 stream-json 已够 KernelEvent？ | 通道选型（§19 开放题） | Path B 内部分支，不否定外包 runtime |
| **F3** | 无注入时，用户是否仍感到「这是我的公会」而非又一个 Pixel Agents？ | 差异化叙事真空 → 需提前做最小记忆/点名 | **挑战「A1 只翻译事件」**：可能要极薄身份锚（角色名/会话标签），仍不做全 §8 |
| **F4** | 用户是否接受「重活外包 + 主搭档轻量」分工？ | 可能回流成「为什么不直接 Hermes/Claude」 | 挑战伴生内核定位；应用研/访谈，非纯工程 |
| **F5** | 跨一次 runtime 切换后记忆仍在？ | runtime-agnostic 灵魂是空话 | post-A1 / M2；Path B 的长期赌注 |

**对 Path B 的裁决（研究侧）：**  
- **保留 Path B 作为主路径** —— 竞品证明纯观察不够，A1 驱动级是正确的最大技术风险前置。  
- **微调：** A1 成功标准应写成「事件保真度 + 等待/审批可演」，避免用「更好看的小人」冒充通过。  
- **挑战点：** Agent Cockpit 已部分占据「launch + 审批 + 记忆编辑」；Tsukumo 若长期只有 spectacle + 薄 adapter，会被 Cockpit/Pixel 路线图吞掉。M2 记忆/技能闭环不能无限后移。

### 4. 看起来很热、但应视为商品 / 宜 post-MVP 的能力

| 能力 | 为何像护城河 | 为何其实是商品或应后置 |
|---|---|---|
| 像素小人 / 办公室编辑器 | 传播与安装量 | Pixel/Office/Cockpit 已验证；§1.3 定为入场券 |
| 多 agent Kanban / worktree 编排 | 真实痛点 | Vibe Kanban、Conductor、Squad 专打；Tsukumo 委托看板可薄做 |
| GEPA / gskill 自动炼 skill | 学术热度 ICLR/cookbook | 库可后挂；§16 已正确列为 MVP 不做 |
| 静态 SOUL.md 人格包 | 「有灵魂」话术 | AgentSoul 等正在标品化；无经历则无依恋 |
| 会话全文搜索 | AgentRoom+CASS | 可观察级附赠，非核心 |
| 3D/VR/宝可梦皮肤 | 眼球 | 法律与军备竞赛双坑 |
| Process Fidelity 评测套件 | 求职叙事好看 | §18 防绑架：勿进 MVP |
| 完整 §8 多方言注入 | 终态正确 | A1 后；先委托书注入一条路径 |

### 5. 主 agent 应与主人讨论的开放风险

1. **Hermes 替代风险：** 若目标用户要的是「一个越来越懂我的 agent」，Hermes 已闭环；Tsukumo 必须讲清「为什么要公会 + 多雇佣兵」的不可替代场景（多工具切换失忆、阻塞无感知、关系资产不跟 vendor 走）。  
2. **Agent Cockpit 路线图吞并：** 若 Cockpit 加上跨 session 养成/羁绊，指挥所+像素的中带碰撞升维；需监控其 Memory/插件 SDK 进展。  
3. **Pixel Agents 多 runtime 落地：** 一旦 HookProvider 真接 Codex/Gemini，Tsukumo 在 spectacle 上的「多 runtime」叙事优势缩小 —— 必须靠 **拥有会话 + 灵魂 IR** 拉开。  
4. **ACP 成熟度与 Windows：** Claude 仍偏 adapter；A1 在 win32 上选 ACP 还是 stream-json 需 spike 当天实测（§19）。  
5. **「有灵魂」话术通胀：** AgentSoul/EvoClaw/SOUL.md 让市场对 soul 脱敏；对外应强调 **Process Fidelity / 跨 runtime 经历**，少空喊 soul。  
6. **Vibe Kanban 日落：** 编排赛道整合中，勿绑定已日落产品做对标叙事。  
7. **法律/IP：** 避免宝可梦等粉丝资产路线（The Office 前车）。  
8. **范围纪律：** 求职 Observatory 叙事（§18）与 GEPA 评测勿绑架 MVP；多层融合也要砍到可证伪切片。  
9. **主搭档人格未定（§19）：** 无主搭档时「伴生内核」易被理解成空壳 —— 产品叙事缺口。  
10. **诚实降级：** 观察级 watcher 可获客，但若营销成驱动级，会重复用户对 Pixel Agents「harness 很薄」的失望。

### Related research in this task

- `self-evolving-harness-landscape.md` — 自进化机制地图  
- `competitor-matrix.md` — 结构化对照表  
- `pixel-viz-companions.md` — 可视化竞品深度  
- `runtime-interop-acp.md` — A1 通道含义  

## Caveats / Not Found

- 未做用户访谈；F3/F4 为假设性证伪题。  
- 竞品星标/安装量/日落状态以 2026-07-09 抓取为准。  
- 本文件是规划用研究综合，**不是** PRD 变更；不修改 `DESIGN.md` / `prd.md`。
