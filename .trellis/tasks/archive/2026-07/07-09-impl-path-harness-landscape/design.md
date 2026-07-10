# Design: 实现路径 B 与关系层切片

- **Task**: `.trellis/tasks/07-09-impl-path-harness-landscape`
- **Date**: 2026-07-09
- **Status**: 规划中（本轮已拍板项固化；未决项显式开放）
- **Sources**: 根目录 `DESIGN.md`；本任务 `prd.md`；`notes-executor-role-model.md`；`research/*`

> 本文是**本任务的技术设计**，不是替换根目录 `DESIGN.md`。根目录 `DESIGN.md` 是项目愿景北极星，**本任务及后续实现均不修改它**；执行细节与本轮拍板以本文 / `implement.md` / 任务笔记为准。未决项见 §6，禁止在实现里写死。

---

## 1. Architecture & Boundaries

### 1.1 本轮固化的产品切片

```
┌─ tsukumo TUI（公会大厅，唯一交互面）──────────────┐
│  舞台（spectacle）│ 日志/委托 │ 审批门（可 stub）   │
├─ Director（KernelEvent → StageEvent，纯函数）──────┤
├─ Soul store（canonical IR：记忆简报 / 召回索引）───┤  ← 关系自进化主战场
├─ KernelEvent bus ──────────────────────────────────┤
│     ↑                ↑                ↑            │
│  builtin loop*   drive adapter    observe watcher  │
│  （瘦宿主，非     （A1：ACP 或      （获客台阶，     │
│   完整 coding     stream-json）     灵魂不长）      │
│   agent）                                           │
└────────────────────────────────────────────────────┘
* builtin 与外部 adapter 产出同一套 KernelEvent；身份模型见 §6（未决）
```

**边界纪律（沿用根设计，本轮重申）：**

| 边界 | 允许 | 禁止 |
|---|---|---|
| theater | 消费 StageEvent | 感知 vendor / ACP 细节 |
| adapters | 生产 KernelEvent；编译委托书简报（薄） | 持有 canonical 记忆真源 |
| soul store | canonical 记忆 / 召回；容量封顶导出 | 按 vendor 分叉两套成长账本 |
| builtin loop | 轻任务 / 对话宿主（范围可后收） | MVP 内卷编码能力军备 |

### 1.2 与根设计里程碑的关系

| 根设计 | 本任务解释 |
|---|---|
| Path **B 为主** | A1（外部事件→舞台）尽早；伴生内核不做完整 coding agent |
| Path **A 并行瘦身** | S0–S1 可用 fixture / 录制事件流，不阻塞等真 runtime |
| §8 全量注入 | **后置**；A1 旁路只做委托书简报 + 召回探针 |
| M2 skill 闭环 | **后置**；仅留 SKILL.md / trait 插座 |
| GEPA / Process Fidelity 评测 | **不做进 MVP**（根设计 §16 / §18） |

---

## 2. Decided Design（本轮已拍板）

### 2.1 自进化主战场 = 关系自进化

- **做**：记忆、user model、跨会话连续、（可选）简单羁绊计数；经历可追溯。
- **不做进主叙事**：GEPA / 自动炼 skill / prompt 进化。
- **吸收姿势**：能力层当市场商品——SKILL.md、冻结快照等标准件可挂载；打出关系差异化后再充分吸收竞品能力管线。

### 2.2 关系体感时机 = 中偏早

A1 验证事件翻译时，**旁路**最小关系探针，证伪「换会话还认得主人」：

| 探针包含 | 探针不含 |
|---|---|
| 容量封顶委托书简报（MEMORY/USER 类冻结快照子集） | 完整 §8 多方言编译 |
| 一次跨会话 FTS/召回（可先本地 sqlite） | 每 tool call 提炼 |
| 可选羁绊计数（事件溯源，可后接） | 「领悟新技能」结算 UI |
| | MCP 记忆服务全量工具面（可后置，位预留） |

### 2.3 探针范围 = 记忆为主，skill 只留接口

- 目录/trait 预留 `skills/` 与 `skill_create` 插座，A1 **不暴露**沉淀招式产品面。
- 完整 skill 自沉淀对齐根设计 M2。

### 2.4 Token / 上下文预算（沿用根设计，不另开炉灶）

实现必须遵守，作为关系探针的硬约束：

1. **拉取优于推送**（`DESIGN.md` §8.4）：每次委托只推容量封顶简报 + 相关性 top-k；长尾召回另通道。
2. **冻结快照 + 渐进披露**（§10.2）：禁止整库编年史进 prompt。
3. **消息双轨**（§5.3）：演出/结算/UiMessage 不进 LLM 上下文。
4. **沉淀触发同构台词三级**：任务结束 / 定时 nudge / 手动；**禁止**每 tool call 跑提炼 LLM。

### 2.5 A1 接入深度 = 双通道务实

**A1 成功标准（须同时满足）：**

1. 外部结构化事件 → `KernelEvent` → 舞台可演（工具态 / 等待或审批 / 结束）。
2. 委托书简报能进入本次 prompt 组装点（即使简报内容先写死/夹具）。
3. 不要求完整 ACP `fs/*` / `terminal/*` 代理。

**通道策略：**

| 条件 | 选择 |
|---|---|
| Windows 上 ACP bridge（如 Claude adapter）稳定 | 优先 ACP：session + updates + permission |
| ACP 不稳 | **自有进程** `stream-json`（或等价）→ KernelEvent；ACP 标为 A1.1 |
| 仅舞台观感 | fixture/JSONL 可服务 S 线，**不得**单独算 A1 驱动级通过 |

**诚实叙事：** 驱动级 =「够演 + 够注入」；编辑器式完整宿主后置。Advertise 的 client capability 不超过 TUI 已实现面。

### 2.6 差异化主张（可证伪，对外话术）

相对纯像素观察器 / 纯自进化 harness：

> **跨执行后端仍连续的关系资产（记忆/经历数值），公会拥有会话与审批演出；能力可换，关系不跟 vendor 走。**

少空喊「soul」；强调经历与可移植（与静态 SOUL 包商品化对打）。依据：`research/tsukumo-differentiation-implications.md`。

---

## 3. Data Flow & Contracts

### 3.1 事件主链

```
Runtime / builtin
    → Adapter（归一化）
    → KernelEvent
    → World reducer（可后置；A1 可透传）
    → Director(event, world?, lineBook?) → StageEvent
    → Theater + Log
```

- `KernelEvent` 为上层唯一契约（枚举细节可在实现前再收一版，但 **vendor 字段不得泄漏到 theater**）。
- 导演器保持纯函数；实时 = 状态映射（有损采样）；回放/结算 = 叙事剪辑（可后置）。

### 3.2 关系探针数据流（薄）

```
Soul store (canonical)
    → BriefCompiler（容量封顶、相关性 top-k）
    → 委托 prompt 组装点（adapter / session owner）
    → Runtime（无二次元人格词）

QuestEnd / 手动 / nudge
    → MemoryCurator（低频）
    → Soul store
```

- 每一次简报注入建议记一条可追溯事件（根设计 §8.6 方向）；A1 可用日志 stub，schema 可扩展。
- **化妆在出口**：StageEvent 台词与 runtime 人格无关。

### 3.3 存储（倾向，可换实现）

| 数据 | 倾向 | 扩展性 |
|---|---|---|
| 事件/session | append-only JSONL | 预留 id/parentId |
| 召回 | sqlite + FTS5 | 可换引擎，勿把 SQL 写进 theater |
| 冻结快照 | MEMORY/USER 类 markdown 或等价 IR | 单一事实源；方言只在 adapter |
| skills | 目录 + SKILL.md 插座 | A1 可空 |

---

## 4. Falsification Order（验证序）

对齐调研 F0–F5，收敛为本任务工程序：

| 序 | 问题 | 通过线索 | 失败含义 |
|---|---|---|---|
| **F0** | 目标终端上舞台愿意开着干活？ | S0/S1 帧率与观感 | spectacle 入场券不成立（不否定 Path B） |
| **F1 ★** | 外部结构化事件能否稳定驱动演出？ | A1：tool/wait/end → 舞台 | 指挥所退化壁纸；终态假设崩 |
| **F2** | 通道选 ACP 还是 stream-json？ | Windows spike 当天实测 | 仅通道分支，不否定外包 runtime |
| **F3** | 无关系锚时是否像又一个 Pixel Agents？ | 中偏早探针：跨会话一句「还记得」 | 差异化真空 → 探针不可再砍 |
| **F4** | 用户是否接受「重活外包 + 瘦宿主」？ | 访谈/自用，非纯工程 | 伴生内核定位再收 |
| **F5** | 换执行后端后关系资产仍在？ | post-A1 / M2；同一 canonical IR | runtime-agnostic 灵魂空话 |

**A1 通过定义：** F1 为主门；简报组装点打通为 F3 的工程前置；F2 为 A1 内决策，不阻塞「事件能演」。

---

## 5. Risks & Mitigations

| 风险 | 等级 | 缓解 |
|---|---|---|
| Spectacle 红海（Pixel ~更高安装量） | 高 | 入场券而已；护城河日程绑 F3/F5，不绑家具皮肤 |
| Agent Cockpit 占 ops+像素 | 高 | 不拼面板完整度；拼跨后端关系 IR + 养成叙事 |
| Hermes「一个懂我的 agent」替代 | 高 | 产品叙事强调多后端 + 关系不跟 vendor；能力不宣称更强 |
| Claude ACP = bridge，Windows 不稳 | 中 | 双通道务实；stream-json 后备 |
| 只有翻译事件、无探针 → 被路线图吞并 | 中 | 中偏早记忆探针旁路 |
| 执行层 ontology 写死导致两套成长 | 高 | **本轮不拍板**；身份≠backend 预留（§6） |
| 静态 SOUL 话术通胀 | 中 | 对外 Process Fidelity / 经历，少空喊 soul |
| 求职/GEPA 评测绑架 MVP | 中 | 根设计 §18；本文不做 |

---

## 6. Open Extension Points（未决，禁止写死）

### 6.1 执行层角色模型（延期专项）

主人初步倾向（**非拍板**）：工具付丧神与雇佣兵是**同一执行层**，仅 `RuntimeBackend` 不同；自有设定须能在第三方工具上生效；不宜两套成长账本。三分法本轮视为臃肿。

**实现约束（软）：**

- 类型上预留 `SpiritId` / `ExecutorId` 与 `BackendKind`（builtin | acp | stream_json | watcher | …）分离。
- 成长/记忆事件打在执行者身份上，**不要**以 vendor 字符串为主键。
- 主搭档：可选标记或独立槽，允许日后合并为执行者或升为关系门面。
- **禁止**在本任务实现中落地「付丧神物种 vs 雇佣兵物种」两套表结构。

详情与后续问题清单：`notes-executor-role-model.md`。根 `DESIGN.md` 不修改；分歧以任务设计/笔记为准。

### 6.2 其他本轮不写死的点

| 主题 | 态度 |
|---|---|
| 主搭档人格 / 命名 / 阵容 | 开放 |
| soul/guise 两层 | 根设计倾向保留，未终裁 |
| 观察级在统一执行模型中的降级语义 | 开放（须避免再引入「两种生物」） |
| 委托看板 / 多并行大厅 UI | 开放 |
| 完整 ACP fs/terminal | 后置 |
| MCP `recall` 工具 schema | 位预留，参数不定死 |
| 简报具体 token 上限数字 | 实现时标定；原则已定 |
| Cargo crate 切分细节 | 根设计草案可参考，可调 |
| 世界观 / 美术 | MVP 冒险者公会倾向，不锁资产管线外的叙事 |

---

## 7. Compatibility & Rollout Shape

- **对根 `DESIGN.md`：** 只读北极星，不修改。本轮拍板与执行细节落在本任务产物；若与根文表述有张力，以任务 `design.md` / 笔记为执行依据，根文保持愿景层稳定。
- **对竞品：** 不绑定 Vibe Kanban（调研示 sunsetting）；Pixel/Cockpit 作 spectacle/ops 参照而非模仿对象。
- **回滚：** A1 通道可弃 ACP 保 stream-json；探针可降级为夹具简报；舞台与 adapter 经 KernelEvent 解耦，可单侧回滚。
- **卸载：** 受管区块/注入若未做则无文件尸体；若做了须遵守根设计 §8.3 标记块纪律（后置实现时）。

---

## 8. Out of Scope（本设计明确不做）

- 生产代码与 `task.py start`（仍属规划阶段）
- GEPA / gskill 管线、Process Fidelity 评测套件
- 完整 §8、多 runtime 观察级全覆盖、闲时动机引擎
- 执行层 ontology 定稿、主搭档人设定稿
- 修改竞品或依赖其云服务

---

## 9. Next Artifacts

- `implement.md`：把 F0–F3 / A1 双通道 / 探针拆成有序 checklist 与验证命令（仍保持 §6 扩展点）。
- 执行层专项：另开讨论或子任务；结论写入任务笔记/子任务设计，**不**改根 `DESIGN.md`。
