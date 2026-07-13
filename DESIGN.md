# Tsukumo — 设计稿（合并终版）

> 状态：北极星已冻结；V0 TUI 世界观、默认角色、视觉参考与内容包边界已收束，执行由 Trellis 任务管理。2026-07-13 go/no-go 复核结论：**有条件 GO**——方向不变，护城河论述更新，V0 验收纪律加严（见 §2.7、§16）。本文是 tsukumo 项目的唯一事实来源。
> 日期：2026-07-13
> 合并来源：`archive/DESIGN-A.md`（付丧神叙事主线）+ `archive/DESIGN-B.md`（事件驱动内核路线）+ 2026-07-07 讨论共识 + **2026-07-08 愿景收束讨论** + **2026-07-11 V0 前端收束讨论（舞台优先/九十九工房/栞/PresentationPack）** + **2026-07-13 go/no-go 复核（市场更新/增强层护城河边界/V0 验收纪律）**
> 命名：Tsukumo（付丧神）— 日本神话中"器物经年吸收精华化为精灵"的概念，是工具拟人化的文化根基。

---

## 1. 项目定位

### 1.1 核心命题

**一套适用于任何 agent 工具的可成长 agent 状态 —— agent 时代的关系层。**

这是 pi 与 hermes 两个灵感源的设计哲学融合：极简、模块化（pi）× 天生越用越懂你（hermes）× 不锁定模型/工具（两者共同的 provider 无关纪律）。

一句话终局定位：

> **Tsukumo 是一座"有灵魂的指挥所"：自有的公会大厅是唯一交互面，自研内核只养灵魂不卷能力，市面上所有 agent 都是可签约的雇佣冒险者——能力属于市场，关系属于主人。**

次级命题（保留自初版）：**把一个 agent 工具变得有意思。** 不一定要有多强的性能，但要足够有趣。

### 1.2 市场格局（2026-07-08 调研更新）

原"市场空白观察"（工具向无人格 / 陪伴向无能力）已过时半格，须以新格局为准：

**背景趋势**：AI 编程已从"人类盯着 AI 干活"演进为"人类委派任务给 agent 群"（CLI agent / IDE agent / 云端异步 agent 三形态并立，ACP 协议标准化，git worktree 并行编排普及）。开发者的角色变成监工/会长，同时挂着多个并行任务——**是时候把人类的注意力再拉回来了**。

**已被验证、也已被商品化的部分**：像素小人可视化 agent 已有一批产品（Pixel Agents 1.3 万+下载并获 Fast Company 报道、Agent Cockpit、The Office、AgentRoom 等）。它们证明了两件事：

1. "小人跑来跑去"不打乱心流——前提是定位为**环境化信息（ambient information）**而非表演；
2. 立住的根基是**注意力管理的真实痛点**（agent 阻塞等输入无人察觉、终端窗口泛滥）。

**仍然无人占据的部分**（tsukumo 的护城河）：

- 竞品的小人没有灵魂——不记得昨天、没有成长、换个会话就是陌生人；
- 竞品全是只读观察者皮肤，不拥有会话，做不了记忆沉淀/技能成长/羁绊；
- "养成数值 = 真实自改进数据"的同构（Hermes 闭环 × RPG 养成）仍是 tsukumo 独有洞察。

**定位结论**：入场券 = 注意力管理的实用效用；护城河 = runtime 无关的灵魂层。指挥所赛道邻居（Vibe Kanban、Conductor、Agent Cockpit 等）用效用竞争，tsukumo 用关系留存。

**2026-07-13 go/no-go 复核更新**：

- **入场券贬值加速**：Pixel Agents 装机 1.3 万 → 7.4 万+，并公布 agent / 平台 / 主题无关的架构路线（HookProvider 适配层，Codex/Gemini/Cursor 在路线图上）；
- **纯效用指挥所变现受挫**：Vibe Kanban 于 2026-04 被 Bloop 停运、转社区维护——既佐证"效用竞争难留存"，也警示该赛道商业化艰难；
- **跨工具记忆被基础设施化**：Mem0 OpenMemory MCP 把"记忆跨工具"做成标准件（本地部署、审计面板、任意 MCP client）。痛点二（记忆不通用）正被专业玩家单独解决，该点不再单独构成差异化卖点；
- **新物种"养成伙伴"入场**：Buddy（MCP 虚拟宠物，21 物种 / XP / 情绪 / 跨 client 持久，8 周 1 万+ clone）、CodePal（5 维情绪引擎桌宠）。**"竞品小人没有灵魂、不记得昨天"的论述部分失效**——但它们的养成是打卡式假进度条（Direct State Setter 对照组的现实版）、不拥有会话、无状态连续性；
- **ACP 押注兑现**：协议 stabilize、注册表上线（2026-01 上线 / 2026-03 stabilize），25+ agent 含 Cursor 接入，驱动级接入成本低于一年前预期。

**护城河修正**：从"别人没有灵魂"收窄为"**别人的灵魂是假的、而且搬不走**"。tsukumo 独占位置 = **拥有会话（驱动级接入）× 真实成长数据（事件溯源 reducer）× 跨 runtime 状态连续性（checkpoint/handoff）** 三者交集，该交集目前无人占据。代价是时间窗收窄：表层养成正被快速抢注，V0 必须尽快把"真养成 + 热交接"变成可演示。

### 1.3 "有意思"的三个时间层次（权重翻转）

| 层次 | 机制 | 验证方式 | 定位 |
|---|---|---|---|
| **第一眼：看它干活像看戏**（spectacle） | 像素舞台演出、台词、结算画面 | 自己愿意开着剧场模式干活 | **入场券**（已被市场商品化） |
| **第一个月：看它长大**（progression） | 技能沉淀、熟练度、羁绊、引用共同历史 | 能感到它比第一天更懂自己 | **核心命题** |
| **长期：它成了唯一的那一个**（attachment） | 积累的记忆/技能/理解无法被新实例替代 | 会舍不得删掉它的数据目录 | **核心命题** |

MVP 仍用第一层建立体验，但护城河投资在第二、三层。押注：agent 能力商品化之后，**依恋（attachment）成为唯一不可迁移的留存**——主人可以随时换模型换工具，换不掉那个记得所有偏好、陪着修过两百个 bug 的伙伴。

### 1.4 切入痛点

来自真实使用体验的四个痛点，tsukumo 一次性回应：

1. **注意力打散**：不同工具、不同模型擅长不同场景，来回切换时注意力碎裂、操作繁琐；
2. **记忆不通用**：各工具的历史上下文/记忆/偏好互不相通，每换一个工具就"失忆"一次；
3. **阻塞无感知**：后台 agent 卡在等审批/等输入，人类几分钟甚至几小时后才发现；
4. **中断后重入无感知**（2026-07-13 新增，真实案例）：因病离开几天后重新开工，部分状态/决策其实早已过时，但人的记忆已模糊，直到推进一段时间才发现走错，后续方向也过了许久才重新想起。约定式记录（journal/markdown）依赖 agent 与人的自律，恰恰在这种场景下最不可靠——这是"陪伴型 + 重证据交接"设计对轻量增强层的核心差异点。

### 1.5 反 Clippy 原则

- **可靠地做事，有趣地表达**——随机性只许进台词和演出，禁止进任务执行
- **主动性尊重心流**——想说话可以，别打断主人手上的活
- **有用是有趣的地基**——agent 连日常小任务都办不利索时，"有趣"会塌缩成"尬"
- **演出不阻塞执行**（2026-07-08 新增）——演出节奏不得反向控制执行节奏。工具 0.1 秒跑完，小人可以继续演 2 秒，那是舞台在追赶状态，不是内核在等舞台。舞台本质是**有损采样器**：事件过密时丢帧合并（连续 5 次 commit 演成一次 montage），而非排队慢放。

### 1.6 付丧神叙事

每个工具（git/shell/filesystem/...）都是一个"成精的器物"，有持久人格、世界观适配外观、台词库、招式集、与主人的羁绊值。叙事在最深处与产品机制同构：付丧神既是皮肤，也是"越用越有灵魂"这个核心机制的神话表达。

---

## 2. 终态形态：有灵魂的指挥所 ★2026-07-08 拍板

### 2.1 形态抉择

三条纯路线各有致命伤，均不采用：

| 候选形态 | 致命伤 |
|---|---|
| 完整 agent 工具（Claude Code / opencode / codex 同类） | 死于能力军备竞赛。单人自研内核追不平团队工程，能力差距使"有趣"塌缩成"尬"（违反反 Clippy） |
| 纯增强层（hook/规则文件寄生，OMX/OMO/Trellis 式） | 死于不拥有屏幕。tsukumo 的产品本体就是一块交互面（舞台/结算/回放/对话），皮肤无法寄生在自己不拥有的屏幕上，灵魂沦为壁纸 |
| 自有前后端 + agent 纯黑盒 runtime | 只读接入够不着灵魂机制（记忆注入/审批演出/羁绊），退化回观察者皮肤 |

**采用第四形态**：自有前端（公会大厅）+ 自有伴生内核（灵魂宿主）+ 外部 agent 作为**可驱动**的插拔 runtime。解锁关键是 ACP（Agent Client Protocol，Zed 发起、JetBrains 共同维护、19+ agent 兼容）：tsukumo 作为 ACP client **拥有会话**——发出委托、收到结构化工具调用事件流、接管权限审批请求。

**"A 为骨"的终态含义改写**：自研内核不再肩负"有竞争力的编码 agent"，它是**灵魂的宿主**——主搭档日常对话、闲时生态、记忆检索、轻量任务（外部重型 agent 不做、也不需要顶级能力的部分）。重活全部外包给 runtime。能力军备竞赛退出，灵魂军备竞赛没人参赛。

### 2.2 架构总览

```
┌─ tsukumo TUI（公会大厅 = 唯一交互面）────────────┐
│   舞台 / 委托看板 / 结算 / 回放 / 主搭档对话        │
├─ 导演器 + 世界层（只消费归一化 KernelEvent）───────┤
├─ 灵魂层（记忆·技能·羁绊·user model）← 跨 runtime 持久 │
├─ 归一化事件总线 KernelEvent ──────────────────────┤
│    ↑            ↑              ↑                  │
│ 自有伴生内核   ACP adapter    transcript watcher   │
│ (主搭档/轻任务) (Claude Code,  (降级观察, 只演出)    │
│               Codex, ...)                         │
└───────────────────────────────────────────────────┘
```

归一化在协议层完成、不在 UI 层：所有 runtime 输出（ACP 事件 / headless stream-json / 转录文件）由各自 adapter 翻成同一套 KernelEvent，导演器和舞台对 runtime 完全无知。这正是"theater 只是事件消费者"纪律的回报——当初为"未来接 Live2D 前端"设计的解耦，反向用于"接任意后端"。

### 2.3 三级接入（UI 优雅降级）

| 等级 | 通道 | 能力 | 叙事包装 |
|---|---|---|---|
| **驱动级** | ACP / headless 结构化流 | 全功能：演出 + 审批门 + 记忆注入 + 羁绊成长 | 签约的雇佣冒险者 |
| **观察级** | 转录文件 watcher（只读） | 小人上台、演出可看，灵魂不长 | 路过的冒险者（"雇佣关系未缔结"），低门槛获客入口：先看热闹，再签契约 |
| **未接入** | — | 不出现 | — |

### 2.4 异构性即角色性 / 付丧神与雇佣兵分层

- **异构性即角色性**：不同 runtime 的脾气差异（有的沉默只交结果、有的絮叨爱解释、有的快而莽）不在 UI 里抹平——它们天然是不同性格的雇佣冒险者。集成碎片化从工程 bug 变成公会人设素材。
- **两个粒度的角色共存**：git/shell/fs 的工具付丧神属于自有内核（轻任务、常驻家人）；外部 runtime 是签约外援（重委托）。对应真实工作流"顺手小事自己的助手做、大活派给重型 agent"的分工。

### 2.5 灵魂层 runtime 无关（能力商品化，关系私有化）

灵魂层的注入通道全是现成标准件：SKILL.md（agentskills.io 兼容，各家 agent 都认）、AGENTS.md 受管区块、`--append-system-prompt`、ACP session 参数、MCP server、审批/hook 流。灵魂层做到 runtime 无关之后：

> 主人的 agent 可以随时换（今天 Codex 强就派 Codex，明天 Claude 降价就切回来），付丧神不用换——记忆、技能库、羁绊全在 tsukumo 的数据目录里原地不动。

这也让 Process Fidelity 纪律（见 §14）有了商业理由：正因为羁绊是从真实流量里长出来的，它才搬不走、抄不了。

### 2.6 已知代价（诚实清单）

1. **指挥所赛道在变挤**（Vibe Kanban、Conductor、Agent Cockpit 等）——差异化靠灵魂层撑，形态只是避免和 Anthropic 打架；
2. **adapter 维护税**——ACP 收敛了大半，但各家 headless 格式仍会漂移，接受"永远有一个 adapter 是坏的"的常态；
3. **入口迁移成本**——tsukumo 想成为"启动 agent 的那个终端"，观察级接入是给这个门槛垫的台阶。

### 2.7 与轻量增强层的护城河边界 ★2026-07-13

一次现实检验：Trellis（仓库内 markdown + 约定 + 脚本）协调多工具，在数天内产出本项目约 2.3 万行高质量代码——证明"跨工具协调 + 上下文连续性 + 质量门"的**功能性 80% 寄生层就够了**，且该层还在被双向商品化（平台内置记忆 + OpenMemory 类基础设施）。tsukumo 的增量价值必须由寄生层结构性够不着的四件事承担：

1. **事务性捕获 vs 自觉散文**：约定层的记忆是"请 agent 自觉写"。实证：本项目周末七个 session 的 journal 有六个留着 `(Add details)` 占位符，跨 session 连续性实际靠主人当人肉总线。Chronicle 的事件是发生即落库，checkpoint 是结构化 StateRef，不请求任何人的自律；
2. **拥有会话**：hook 只能观察不能拦截——审批接管、注入 receipt、执行中途热交接都要求坐在 client 位置。增强层只能做任务边界的冷交接；
3. **per-person vs per-repo**：约定层状态长在仓库里跟项目走，灵魂层跟人走（跨项目的偏好与共同历史）；
4. **寄生层会被宿主吸收（Sherlock 效应）**：增强层长在宿主地皮上，宿主内置同类功能即差异化蒸发；聚合层站在宿主之上，宿主间能力军备竞赛反而是利好。**增强层赌宿主不做，聚合层赌宿主内卷。**

**V0 demo 对照组标准**（举证责任升级）：要打败的不是裸奔用户，而是"**一个自律的 Trellis 用户**"。三条硬标准：

- **零自觉捕获**：handoff 全程不要求 agent 写 journal、不要求人读 markdown——切 runtime，对方直接带着状态开工；
- **热交接**：执行中途 checkpoint 换 runtime 接力，而非任务边界冷交接；
- **审批闭环**：权限请求弹到契约台、批准后执行继续。

若 V0 演示不出对"主人 + Trellis 人肉总线"的数量级优势，才是真正讨论 pivot 的时点——这是可证伪的产品假设。

长期观察项：真正可能威胁聚合层位置的是同样拥有屏幕与 ACP client 身份的编辑器（Zed / JetBrains / Cursor）；其激励是 IDE 生产力而非关系与依恋，短期不构成正面冲突。

---

## 3. 核心概念

### 3.1 付丧神（Tsukumo）

工具拟人化的叙事根基。每个工具都是一个"成精的器物"，有：
- 持久人格（soul）
- 世界观适配外观/称谓（guise）
- 台词库、招式集
- 与主人的羁绊值

### 3.2 任务即场景演绎

不是"调用工具打印日志"，而是工具角色在像素舞台上**演出**执行过程：
- 角色出场/退场有仪式感（走位 + 标志性台词）
- 角色对操作结果有情绪反应（成功得意/失败慌张/冲突吐槽）
- 多角色协作时有站位、互动、对话

### 3.3 极简内核 + 可替换内容包 ★2026-07-11 拍板

Tsukumo 沿用 pi 的极简纪律：内核只维护事件、状态、执行、安全与持久化契约，表现内容全部通过显式数据边界注入。

- **PresentationPack**：世界观、主搭档元数据、称谓、台词、调色、场景和精灵帧；
- **Spirit / relationship state**：跨 runtime 的身份、记忆与关系事实；
- **Skill**：兼容 agentskills.io 的能力资产，与表现包独立演进；
- **RuntimeBinding**：Claude、Codex 等执行后端，不进入角色长期身份。

V0 内置 `default-shiori`，同时支持 `--presentation-pack <directory>` 加载一个外部版本化目录。外部包是经校验的声明式表现数据，不拥有脚本、网络、Prompt 注入、进程、仓库或 Host action 权限。

### 3.4 双模式并存（架构原则）

```
任务模式（前台）：命令式 —— 可靠、直接、主人说了算
                  ↕ 同一批角色、同一套记忆
闲时模式（后台）：动机驱动 —— 需求驱动、行为涌现、生活感
```

- **MVP 只做任务模式**，闲时生态 schema 预留 `motivation_state` 字段，post-MVP 接入
- 闲时生态采用 Loomstead 的动机引擎设计资产（见 §14）
- "可靠地做事"用命令式，"有趣地活着"用动机式——反 Clippy 原则的架构落地

---

## 4. 差异化亮点

1. **runtime 无关的可成长灵魂层** ★核心差异化 — 记忆/技能/羁绊跨工具持久，agent 可换、伙伴不换
2. **TUI 像素游戏级演出** — 终端里有一座像素风工房，付丧神小人走动、干活、冒气泡；定位为环境化信息 + 注意力管理（入场券）
3. **角色即工具的持久人格** — 有记忆、成长、羁绊，越用越有温度
4. **自沉淀记忆/skill** — 从使用中沉淀经验，越用越懂主人、越懂怎么演好场景
5. **演绎引擎 + 内容包** — 引擎与内容解耦，多世界观可切换，社区可贡献角色卡
6. **冒险回放** — 事件流 append-only 日志 = 免费的"重温冒险"功能，承载"反复回看"需求

---

## 5. 系统架构

### 5.1 四层解耦 + adapter 总线

借鉴 pi 的事件驱动极简内核 + hermes 的成长闭环 + 像素游戏演出：

```
┌────────────────────────────────────────────────┐
│ L4 剧场层 theater/   像素舞台演出（纯事件消费者）  │
│ L3 世界层 world/     任务系统·好感度·结算·随机事件  │
│ L2 成长层 growth/    记忆·技能·Curator（Hermes 魂）│
│ L1 内核层 kernel/    agent loop·工具·事件流（pi 魂）│
│    + adapters/       外部 runtime → KernelEvent   │
│ L0 基础层            LLM Provider 抽象·TUI 框架    │
└────────────────────────────────────────────────┘
```

L1 有两类事件生产者：自有伴生内核（loop）与 runtime adapter（ACP client / transcript watcher）。两者产出同一套 KernelEvent，上层无感知。

### 5.2 双事件流与导演器

内核事件与游戏事件分层，避免内核被游戏逻辑污染：

```
KernelEvent (pi 式)          WorldEvent (游戏)
tool_execution_end ──┐      affinity_changed
agent_end ───────────┼──→ [世界规则引擎 reducer] ──→ quest_completed / rank_awarded
error ───────────────┘      random_event / level_up
                                   │
                    [导演器 Director] ←── 角色台词库 × 当前情绪
                                   │
                              StageEvent → 像素舞台渲染 + 日志区
```

- **世界状态 = reducer(事件日志)**：好感度、等级全部事件溯源，可重放可审计
- **导演器为纯函数**：`direct(event, worldState, lineBook) → StageEvent`，可单测、可回放
- **导演器双模式**（2026-07-08 明确）：实时模式做**状态映射**（事件 → 小人姿态/情绪，有损采样）；回放/结算模式做**叙事剪辑**（事件日志 → 有节奏的故事）。"看戏"体验的真正落点在后者。
- **台词三级策略**：模板库（角色×事件×情绪索引，零成本）为主 → 关键节点 LLM 润色 → 结算总结用完整 LLM
- **角色包 = pi extension 的拟人化重构**：`card.yaml（人设+数值） + tools/（Tool 实现） + lines/（台词库） + art/（像素 sprite）`
- **单 loop 多皮起步**：初版单 agent loop，角色是工具的"皮"；架构上队长也是角色模块，未来 sub-agent = 真正独立行动的队员

### 5.3 消息双轨制（演出不污染上下文）

`AgentMessage = LlmMessage | UiMessage`，经 `convertToLlm()` 过滤后 LLM 只见标准消息。**结算卡、随机事件、角色寒暄进 session 日志但不进 LLM 上下文**——不烧 token、不干扰任务。

### 5.4 Hook 即演出节点

`beforeToolCall / afterToolCall` 两个钩子是游戏化拦截点：
- 危险命令 → `beforeToolCall` block → 角色犹豫台词 + 确认门——**安全确认门变成角色演出**
- 失败重试 → `afterToolCall` → 受挫台词 + 换策略宣言
- **外部 runtime 同样成立**：ACP 权限审批请求流经 tsukumo → 映射为同一套确认门演出

### 5.5 headless 内核 + 多运行模式

内核不感知前端，像素舞台只是默认消费者。未来升级 Live2D 桌宠 = 事件流的另一个消费者，零重写。

### 5.6 Cargo workspace 布局（2026-07-13 与实际对齐）

```
tsukumo/
├── crates/
│   ├── tsukumo-kernel/    # L1: KernelEvent 契约、身份 newtype、脱敏、JSONL 回放
│   ├── tsukumo-adapters/  # L1: runtime adapter（Claude stream-json 已建，Codex 进行中）
│   ├── tsukumo-soul/      # L2(+L3 现状合并): Chronicle、canonical state、checkpoint/handoff、
│   │                      #   projection receipt、recall/brief、legacy 迁移
│   ├── tsukumo-theater/   # L4: Director、StageEvent、PresentationPack、HalfBlock 渲染、TUI view model
│   └── tsukumo-host/      # 组合根 + 产品 binary: orchestrator、进程生命周期、Safety Plane、
│                          #   交互 TUI、内置 default-shiori 内容包
├── docs/                  # 决策记录 + 视觉契约与参考图
└── data/                  # 默认运行时数据目录（TSUKUMO_DATA_DIR 可改）
```

> 原草案中的 `tsukumo-growth` / `tsukumo-world` 独立 crate 与顶层三模式入口 `src/bin/tsukumo.rs` 未按草案落地：growth/world 职责当前并入 soul / theater，待羁绊与世界层机制成型后再评估拆分；print / replay 目前由 examples 与测试承载，用户可见的 replay 子命令列入 post-V0。characters/ 与 worlds/ 顶层目录由 PresentationPack 机制（§3.3、§12）取代。

边界纪律不变：kernel 禁止 import 上层、theater 只消费事件、adapters 只生产 KernelEvent payload。

---

## 6. 演出节奏与注意力原则 ★2026-07-08 新增

### 6.1 原始矛盾

人类阅读是异步的、反复的（一段聊天记录会在一段时间内反复找出来看），而 agent 执行是突发的、高吞吐的。若把舞台设计成"每个事件都演一段"，只有两种下场：演出拖慢体感，或信息流冲垮观众。

**结论：矛盾不需要"平衡"，需要"分层"。** 参照系是 RTS / 足球比赛 / Dwarf Fortress——人类从不逐条阅读事件，人类看的是状态，想细究再翻日志。

### 6.2 第一性原则：状态与叙事分离

> **舞台呈现"现在时的状态"，文字承载"过去时的叙事"，两者解耦，各自按自己的节奏走。**

三个时间尺度：

| 尺度 | 载体 | 信息特性 | 人类行为 |
|---|---|---|---|
| **100ms 一瞥** | 像素舞台 | 有损、现在时、错过无妨 | 余光扫一眼："Gina 在干活，Term 卡住了" |
| **秒~分钟** | 日志区台词流 | 完整、可回滚、永久 | 空闲时回看刚才发生了什么 |
| **事后回味** | 结算画面 + 冒险回放 | 精心编排、人类节奏 | 反复找出来慢慢咀嚼 |

推论：
1. 实时舞台不为"完整阅读"服务，它服务"我瞟一眼就知道情况"；气泡在舞台上飘 3 秒消失，同一句台词永久落进日志区（舞台和日志区消费同一条 StageEvent 流）。
2. 给人类慢慢咀嚼的叙事放在 quest_end 之后（吟游诗人吟唱战报）——此刻人类恰好有注意力，节奏矛盾天然不存在。事件溯源架构让回放几乎免费。
3. 演出永远不阻塞执行（反 Clippy 第四条），舞台做有损采样。

### 6.3 注意力分级（心流保护）

舞台平时是余光里的鱼缸，只在三类"值得打扰"的事件上升级表现：**阻塞待审批、失败、完成**。V0 落地事件驱动的注意力分级；手动演出密度三档（展开=剧场 / 折叠=精简 / 关闭=静默）列入 post-V0 路线。

---

## 7. 像素舞台（tsukumo-theater）

### 7.1 舞台优先布局 ★2026-07-11 拍板

默认屏幕是一座可操作的像素公会，舞台约占可用高度 70%，日志与快捷键保持克制。

```
┌────────────── TSUKUMO · 九十九工房 ──────────────┐
│ 委托板          Runtime 传送门          记忆柜      │
│                    栞                              │
│ 契约台                               投影卷轴桌     │
├───────────────────────────────────────────────────┤
│ 有界事实日志                                       │
├───────────────────────────────────────────────────┤
│ 状态：待命        [M] 记忆  [P] 委托  [Q] 离开     │
└───────────────────────────────────────────────────┘
```

- **Full**：`>=100x30`，保留完整舞台、设施、日志与 footer；
- **Compact**：`72-99 x 22-29`，简化装饰，Inspector 使用整页；
- **Fallback**：低于 `72x22` 时只保留事实状态、权限决策与 resize 指引；
- **Permission**：始终以阻塞 modal 覆盖舞台，明确显示 allow-once / allow-session / deny；
- **动效可访问性**：V0 的 reduced-motion 固定到语义关键帧；手动展开／折叠／关闭列入 post-V0。

批准的默认屏幕参考：`docs/visual-references/tsukumo-v0-workshop-concept-v1.png`。

### 7.2 终端像素渲染技法

| 技法 | 效果 | 用途 |
|---|---|---|
| HalfBlock `▀▄` | 1 字符 = 2 竖直像素，前景/背景双色 | **像素画主力** |
| Braille `⣿` | 1 字符 = 2×4 点阵，单色 | 粒子/轨迹/线条 |
| 四分块 `▖▚▛` | 1 字符 = 2×2 亚像素 | 高分辨率细节 |
| 24-bit 真彩 | 全色 | 现代终端普遍支持 |
| DEC 同步输出（post-V0） | 防撕裂候选 | 等取得 SSH/tmux 实机证据后评估 |

V0 固定使用通用 Ratatui Buffer + HalfBlock，并按 truecolor、ANSI-256、monochrome 解析调色。Kitty/Sixel 图形协议保留为 post-V0 研究项。

### 7.3 游戏循环 ★2026-07-11 V0 预算

```rust
// V0 采用事件驱动失效 + 有上限的分频循环。
tokio::select! {
    _ = logic_tick.tick() => { game_state.update(TICK_DELTA); }    // 10Hz 逻辑
    _ = render_tick.tick(), if dirty => { terminal.draw(render)?; } // 最高 20Hz
    Some(ev) = event_stream.next() => { apply_event(ev); dirty = true; }
    Some(input) = input_stream.next() => { reduce_input(input); dirty = true; }
}
```

- 执行流永远不等待动画，事件过密时允许丢中间帧；
- resize、输入、语义事件和动画 tick 才触发 dirty；
- reduced-motion 停在语义关键帧；
- 20Hz 是 V0 上限；确定性 17-mode 视觉回执与 Windows ConPTY 实测完成后，只允许向下收缩预算。

### 7.4 Actor 状态机（不引入 ECS）

tsukumo 舞台本质是"演出播放器"，不是真游戏——没有玩家实时操控、没有物理/碰撞/战斗。所需全部行为：

```
Actor 状态: Idle → Walking → Working → Talking → ...
+ 补间移动 + 帧动画 + 气泡
```

V0 不引入 bevy_ratatui，直接在 Ratatui Buffer 上组合逻辑像素并打包为 HalfBlock。

### 7.5 资产管线 ★2026-07-11 更新

```
Codex 生图概念稿
    ↓ 主人审阅并冻结视觉契约
手工归一化为逻辑像素场景 / SpriteFrame
    ↓ 写入版本化 PresentationPack
HalfBlock 直接打包到 Ratatui Buffer
```

V0 的运行时资产是确定性逻辑像素数据。chafa 与 Aseprite 可用于原型和人工辅助，不构成运行时依赖，也不替代人工的轮廓、色板与状态验收。

终端适配规则：粗轮廓、三档明暗、硬色块、无渐变、无 dithering、无抗锯齿；设施与角色在降采样后仍靠大轮廓识别。

### 7.6 V0 视觉契约 ★2026-07-11 拍板

- 完整契约：`docs/visual-references/tsukumo-v0-visual-contract.md`
- 默认屏幕：`docs/visual-references/tsukumo-v0-workshop-concept-v1.png`
- 栞角色五态：`docs/visual-references/tsukumo-v0-shiori-character-reference-v1.png`
- 主色：ink / dark wood / indigo / aged brass / parchment；
- 语义强调：cyan=focus/runtime，vermilion=permission/urgent，brass-gold=settlement；
- 状态不得只靠颜色表达，必须同时使用 pose、copy、边框和标签；
- truecolor、ANSI-256、monochrome、CJK、reduced-motion 都属于验收面。

生成图中的文字只用于构图。默认内容包与 typed read model 提供正式文案。

---

## 8. 灵魂注入协议 ★2026-07-08 新增

灵魂层要向外部 runtime 注入记忆/规则/技能，必然遭遇工具自带提示词、用户已有 AGENTS.md/CLAUDE.md/rules 的冲突。冲突拆成四场架，各有解法：

### 8.1 人格架 → 不打：化妆在出口，不在入口

台词三级策略的纪律推到底：**外部 runtime 永远不需要知道自己是谁扮演的**。它收到干净专业的委托，输出结构化事件流；拟人化发生在事件流出口——导演器拿到 `tool_end(git_commit, success)` 之后才配上傲娇台词。

> **演出是后期配音，不是前期夺舍。** 注入给 runtime 的内容里没有一个字的二次元，只有记忆事实、用户偏好、技能、约束。工具默认提示词爱怎么定义自己就怎么定义，互不相干。

### 8.2 格式碎片化 → 单一事实源 + 方言编译

记忆、用户画像、技能只存一份 canonical 状态在 tsukumo 数据目录；各 runtime 的 adapter 把它**编译**成目标方言（`--append-system-prompt` / AGENTS.md 受管区块 / ACP session 参数）。一个 IR 对多个 backend——永远不手工维护工具专属格式，工具格式漂移只是 adapter 的小修，动不到事实源。

### 8.3 地盘架 → 客人礼仪

写入姿态按侵入性从低到高：

1. **最优先：委托书注入**。tsukumo 是 ACP client，每次委托的 prompt 本来就是自己组装的——本次任务相关的记忆简报直接编进委托消息（委托卷轴自带背景情报）。零文件接触、零地盘冲突，天然任务粒度相关性过滤。大部分注入需求在这一层消化。
2. **次选：受管区块**。确需落文件的持久内容用带标记的托管块（`<!-- TSUKUMO:START/END -->`，同 Trellis 惯例）：块外内容永不触碰、块内幂等重生成、卸载时干净移除。
3. **绝不做**：改写用户自己的内容、替换工具默认提示词。

### 8.4 预算架 → 拉取优于推送

长尾记忆不推。tsukumo 记忆库暴露成 MCP server（`recall` / `search_history` 工具），runtime 需要时自己来查——"贤者坐在档案馆，队员随时来翻编年史"，叙事与架构同构。推送只有一份**容量封顶的简报**（Hermes 容量表头 + 冻结快照照搬：`[67% — 1474/2200]`，相关性 top-k，每次委托新鲜编译）。推少拉多，不和工具自己的上下文经济打架。

### 8.5 语义架 → 让位声明

注入内容与仓库文件类别错位：AGENTS.md 讲"这个项目怎么运作"（项目公约），tsukumo 讲"你的主人是什么样的人、我们一起学到过什么"（关系记忆），正交大于冲突。但仍在简报开头显式声明优先级：**"若与项目内规范冲突，以项目规范为准。"** 记忆是参谋，仓库是军令——记忆的价值在于填空白，不在于压倒现场规则。

### 8.6 兜底：注入本身可追溯

每一次注入都是事件日志里的一条事件（注入了什么、注到哪个 surface、哪个版本、供哪次委托）：

- "agent 为什么这么做"可回答到"因为简报里有记忆 Y"（`traceRefs` 老手艺，Loomstead 血统）；
- 怀疑记忆帮倒忙时，同一委托去掉注入重放 = 现成的反事实实验；
- 卸载时按日志反向清理，不留尸体。

注入系统遵守养成数值真实性原则的姊妹版：**可解释、可撤销、不留暗桩。** 正因为灵魂状态从不占领任何工具的领土，它才真正做到"适用于任何 agent 工具"——寄居蟹不改造贝壳，所以换壳自由。

---

## 9. 角色与表现内容

### 9.1 身份、行为与表现分层 ★2026-07-11 拍板

- **Spirit identity**：长期身份、关系、记忆与行为连续性；
- **RuntimeBinding**：当前执行后端，可在 Claude、Codex 等 runtime 间切换；
- **PresentationPack**：名字、称号、外观、台词、世界术语、调色、场景与精灵；
- **Skill**：可移植能力资产，独立于表现包。

表现包不包含人格 Prompt。拟人化继续遵守“化妆在出口”，执行 runtime 只收到事实、记忆、技能与任务约束。

### 9.2 soul / guise 两层采用

- `soul` 保存跨世界、跨 runtime 的关系连续性；
- `guise` 由 PresentationPack 提供可替换外观和称谓；
- 同一 Spirit 可更换 runtime 与 guise，关系事实保持原位；
- V0 的栞是公会 UI / dialogue face，不建立第二套成长账本。

### 9.3 V0 默认角色：栞（Shiori）

| 字段 | 决策 |
|---|---|
| `actor_id` | `shiori` |
| 显示名 | 栞 |
| 称号 | 九十九工房书记官 |
| 对用户称呼 | 会长 |
| 人格 | 冷静严谨、低频温柔、一本正经的幽默 |
| 轮廓 | 银灰短发 + 书签侧辫 + 靛蓝书封披肩 |
| 核心道具 | 黄铜包角契约簿 + 青色书签灵火 + 朱红封蜡 |
| V0 五态 | Idle / Work / Wait / Urgent / Celebrate |

`actor_id` 只标识表现角色。空 Chronicle 保持事实 `source_spirit_id = None`，表现包不会合成真实执行者。

V0 舞台只生产栞一套可见精灵。当前 executor 的 Spirit 与 Runtime 通过传送门铭牌、状态栏和事实日志标注。Gina、Term、Fio 等执行者角色移入 post-V0 内容扩展。

---

## 10. 记忆系统（自沉淀闭环）

借鉴 hermes-agent 的闭环学习思想：

```
┌─ episodic 场景存档 ─┐  每次任务=一场戏，存：出场角色/动作/结果/台词
│                     │
│  periodic nudge     │──→ 提炼 ──→ procedural 招式（"这类任务该这么演"）
│  （角色内省时刻）    │──→ 提炼 ──→ user model（"主人偏好这样"）
│                     │──→ 更新 ──→ bond 羁绊值（这次合作愉快 +xx）
└─────────────────────┘
        │
        ▼
  self-evolution 角色修行（GEPA，MVP 留接口）
```

### 10.1 记忆类型

| 类型 | 内容 | 对应 hermes |
|---|---|---|
| episodic | 场景存档（每场戏的完整记录） | session history |
| procedural | 招式/skill（程序性记忆） | skills |
| user model | 主人画像（偏好/习惯） | Honcho user modeling |
| bond | 羁绊值（每角色独立） | ★独创 |

### 10.2 Hermes 机制吸收表

| Hermes 机制 | 直接照搬的细节 | RPG 化呈现 |
|---|---|---|
| `MEMORY.md`/`USER.md` 冻结快照注入 | 容量表头、substring 式 replace/remove、保 prompt cache | 「公会档案」+「会长画像」 |
| 会话记忆 SQLite FTS 跨会话检索 | `session_search` 工具 | 贤者翻阅「编年史」 |
| Skills 渐进披露三层 | L0 目录(~3k tok) → L1 全文 → L2 附件 | 技能书：书脊 → 翻开 → 附录 |
| `SKILL.md` 格式 | `When to Use / Procedure / Pitfalls / Verification` 四段 | 「战术手册」，Pitfalls = 前辈的血泪 |
| agent 自建技能 `skill_create` | 任务后主动询问"要沉淀吗" | 结算画面：「新技能领悟！」 |
| Curator 定期整理 | `interval_messages / interval_minutes` 触发 | 随机事件：「今天是档案整理日」 |

### 10.3 养成数值真实性原则

Hermes 学习闭环（经验→记忆→技能）与 RPG 养成（冒险→经验→技能解锁）**同构**：
- 技能熟练度 Lv = 真实的 skill 使用次数 / 成功率
- 羁绊等级 = 真实的 USER.md 丰富程度
- 角色升级 = 该角色（工具）的实际使用统计

**养成数值全部来自真实的自改进数据，不是假进度条**。但需用前快后慢的映射曲线（对数/里程碑解锁）重塑节奏感，避免线性慢增长导致主人感知不到成长。

学术对照（Loomstead 血统）：Direct State Setter baseline（`trust=0.8` 硬写入）就是"假进度条"的对照组，Process Fidelity 就是"真养成"的可评测定义。羁绊/熟练度全部由事件溯源 reducer 从真实经历里长出来——主人培养付丧神不是改数值，是**给它经历**。

### 10.4 沉淀触发

- 任务结束时自动提炼
- 定时 nudge（角色"内省时刻"）
- 主人手动 `/reminisce`

### 10.5 回忆检索

sqlite + FTS5，支持"主人上次遇到类似问题是 Term 帮忙的"这类跨会话召回。对外暴露为 MCP server（见 §8.4），外部 runtime 按需拉取。

### 10.6 self-evolution（角色修行）

MVP 只留 trait 接口，后期接入 GEPA：
- 读执行轨迹 → 针对性优化角色台词/prompt
- guardrails：测试通过 / size 限制 / 语义不漂移 / 人工 review

---

## 11. 招式 / skill 系统

- 载体：**SKILL.md 式**（markdown，人可读可改，兼容 agentskills.io 标准）
- 调用：`/招式名` 或角色自主选用
- 角色"装备"招式，不同角色可有不同招式集
- 招式可被 self-evolution 优化
- **可移植性即注入资产**：SKILL.md 是各家 agent 通认的标准件，tsukumo 沉淀的技能可挂载进任意驱动级 runtime——灵魂层 runtime 无关的具体通道之一

---

## 12. 世界观与 PresentationPack

V0 唯一内置世界是**深夜九十九工房**：一座位于不同 runtime 之间、只在终端亮起时开门的跨世界冒险者公会。西式公会承担功能骨架，付丧神和风纹理承担品牌识别。

| 场景设施 | 产品语义 |
|---|---|
| 委托板 | 任务与 handoff |
| Runtime 传送门 | runtime binding / switch |
| 记忆柜 | durable state |
| 投影卷轴桌 | checkpoint / projection |
| 契约台 | permission request |

`default-shiori` 内置该世界、栞、术语、台词、调色、场景与精灵。V0 可通过 `--presentation-pack <directory>` 加载一个外部版本化目录。Host 在进入 alternate screen 前完成有界读取，Theater 通过纯函数解析并校验，随后只传播不可变的 validated pack。

post-V0 可以扩展多世界、多个角色与社区内容；内核事件、安全、持久化和 runtime prompt 始终保持世界观无关。

---

## 13. 技术选型（已定）

| 维度 | 选型 | 理由 |
|---|---|---|
| 语言 | **Rust** | 像素游戏级演出需要 <1ms 重渲染，Rust 零分配 |
| TUI | **Ratatui + crossterm** | Host 管理终端生命周期；Theater 直接组合 Ratatui Buffer + HalfBlock |
| 游戏引擎 | **不引入 bevy_ratatui**，逻辑像素面 + Actor 状态机 | 保持确定性渲染和极简依赖预算 |
| LLM Provider | **抽象层，多 provider 可切换**（rig/genai） | 不锁 provider，MVP 先接 Claude |
| Runtime 互操作 | **ACP（Agent Client Protocol）为主**，headless stream-json / 转录 watcher 为备 | 驱动级/观察级三级接入（见 §2.3） |
| 存储 | **sqlite + FTS5** | 跨会话检索 |
| session | **append-only JSONL**（id/parentId 预留分支） | 事件溯源，可重放 |
| skill 载体 | **SKILL.md**（兼容 agentskills.io） | 人可读可改，跨 runtime 可移植 |
| 分发 | cargo install / 预编译二进制 | 开发者友好 |

> 选型依据：两轮专项调研（Ink 闪烁/OOM 实锤 + TS 侧 60fps 无余量出局 + movy 先例验证终端像素游戏可行性）。详见调研记录。

---

## 14. 内核核心类型草案

```rust
// kernel/events.ts —— 事件即协议，一切外层围绕它构建
enum KernelEvent {
    QuestStart { quest_id: String, goal: String },
    TurnStart { turn: u32 },
    TextDelta { delta: String },                          // 流式输出
    ToolStart { call_id: String, tool: String, args: serde_json::Value },
    ToolEnd { call_id: String, result: ToolResult, is_error: bool },
    TurnEnd { message: AssistantMessage },
    QuestEnd { stats: QuestStats },                       // 供结算
    Error { error: AppError, recoverable: bool },
}

// kernel/tool.rs
trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn params(&self) -> &serde_json::Value;  // JSON Schema
    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext) -> ToolResult;
}

// theater/pack/model.rs —— 表现内容保持纯数据
struct PresentationPack {
    manifest: PackManifest,        // schema 版本、pack id、入口资产
    world: WorldPresentation,      // 术语、设施与场景布局
    companion: CompanionView,      // 显示身份和称谓
    lines: LineBook,               // 事件×状态 → 台词
    palette: Palette,              // truecolor 与降级映射
    scene: SceneDefinition,        // 确定性逻辑像素场景
    sprites: SpriteAtlas,          // 确定性状态帧
}

// 舞台角色与真实执行者分别记账，避免错误归因。
struct StageAttribution {
    actor_id: PresentationActorId,
    source_spirit_id: SpiritId,
}
```

loop 纪律：本体目标 **<300 行**——`while: 调 LLM → 流事件 → 收集 tool calls → (hook) → 执行 → 结果入上下文`，只留 `beforeToolCall / afterToolCall` 两个 hook 位。

KernelEvent 是唯一契约：runtime adapter（§2.2）产出同一枚举，`QuestStart..QuestEnd` 的语义对自有 loop 与外部 runtime 一致。

---

## 15. 里程碑（双线并行）

| 线 | 里程碑 | 内容 | 验证什么 |
|---|---|---|---|
| 舞台线 | S0 | 像素工房静态场景渲染（HalfBlock） | 画面观感成立？ |
| 舞台线 | S1 | 小人行走动画 + 气泡 + 分屏布局 + 折叠 | 演出感成立？帧率稳？ |
| 内核线 | K0 | agent loop + 2 工具 + JSONL session（print 模式）| 无聊但正确 |
| 内核线 | K1 | KernelEvent 流完整化 + 导演器纯函数 | 事件协议够用？ |
| 汇合 | M1 | S1 + K1 对接：真实任务驱动小人演出 | **"看 agent 干活像看戏"成立？** |
| 适配 | **A1** ★新增 | 第一个 runtime adapter（倾向 Claude Code，ACP 或 headless stream-json）→ KernelEvent → 驱动舞台 | **"外部事件流能否演好戏"——终态最大技术假设，M1 后尽早验证** |
| 成长 | M2 | MEMORY.md/USER.md 冻结快照 + SKILL.md 渐进披露 + skill_create | 成长闭环 |
| 世界 | M3 | 好感度/熟练度 reducer、随机事件、Curator | 养成可视化 |
| 打磨 | M4 | 回放模式、cargo 发布、README | — |

两线在事件协议处握手——KernelEvent/StageEvent 的 schema 是唯一的共享契约，先定协议再分头开工。A1 只验证事件翻译与演出，不做完整注入协议（§8 全量落地在 post-MVP）。

### S0 / V0 TUI 验收清单

- [x] 默认工房概念图与栞五态参考图经主人批准
- [x] 17-mode 确定性视觉回执覆盖 Full / Compact / Fallback，Windows ConPTY 完成正常交互与 resize 实测
- [x] 栞的 Idle / Work / Wait / Urgent / Celebrate 在 HalfBlock 下可辨
- [x] 中文气泡与边框宽字符对齐正确（unicode-width）
- [x] truecolor / ANSI-256 / monochrome 都能区分 Focus 与 Urgent
- [x] reduced-motion 固定到语义关键帧
- [x] 内置 `default-shiori` 与一个外部 fixture pack 通过同一校验
- [x] Windows ConPTY 正常退出与 80x24 → 100x30 resize 回执通过
- [ ] tmux 与 legacy conhost 实机回执待具备对应环境后补录

---

## 16. V0 范围与边界 ★2026-07-11 收束

完整功能门见 `docs/tsukumo-v0-scope-convergence-2026-07-11.md`。

### V0 做

- 版本化 checkpoint、StateRef、source EventId 与跨 runtime handoff；
- Claude / Codex 归一化语义与受控对照证据；
- 舞台优先的可交互像素 TUI；
- 栞一套可见角色、深夜九十九工房一个世界；
- workshop / state inspector / projection inspector / permission modal；
- 内置 `default-shiori` + 一个外部 `--presentation-pack` 目录；
- Full / Compact / Fallback、CJK、色彩降级和 reduced-motion；
- 可安装构建、Windows GNU / Linux 质量门与发布文档。

### V0 不做（留接口）

- 多可见角色、多世界、多 pack 合并、热重载和内容商店；
- 通用美术编辑器、PNG 自动转精灵管线；
- 闲时生态、复杂羁绊、skill self-evolution、GEPA；
- 完整 MCP 记忆服务与长期调试快照产品；
- GUI / Live2D、语音 / TTS、跨平台 messaging gateway；
- Pathfinding、碰撞、战斗和鼠标优先交互；
- Process Fidelity 评测与反事实回放研究产品化。

### V0 验收纪律 ★2026-07-13

- **时间盒**：三个活跃任务（cross-runtime-evidence、handoff-continuity 收尾、release-packaging）于 **2026-07-26** 前全部关闭，且"热交接 demo"可录屏（demo 硬标准见 §2.7）。时间盒是防"基础设施跑在产品前面"复发的机制——一旦某个 gate 长出超预算的工程纪律，以时间盒为据砍除；
- **Codex 侦察先行**：cross-runtime-evidence 开工先做纯侦察 spike，抓取 Codex CLI 真实事件流对照 KernelEvent 清单，确认语义鸿沟（工具事件粒度、权限机制、中途注入能力）后再写 decoder，避免按 Claude 的形状设计接口；
- **live smoke 周跑**：opt-in live smoke 每周至少手动跑一次，receipt/日志记录 CLI 版本号——fixture 契约测试感知不到厂商 CLI 漂移，周跑为 adapter 维护税决策（ACP 主通道迁移时机）积累实测数据。

---

## 17. Loomstead 搭桥 ★2026-07-08 升格

### 17.1 搭桥原则

搬**设计资产**（schema/机制/经验教训），不搬代码（Loomstead 是 Python，tsukumo 是 Rust）。2026-07-08 讨论后，Loomstead 资产从"闲时生态素材"升格为**三件愿景级资产**：

### 17.2 三件愿景级资产

**1. Trace 优先的可观测性 DNA —— "注意力管理"的学术前身**

Loomstead 终版定位是 Agent Behavior Observatory（`sourceEventIds` / `traceRefs` / `candidateScores` 证据链 + 反事实回放），核心问题 "Why did this agent choose this action?"。tsukumo 的舞台就是它的消费级产品化：Observer Dock 用调试面板回答"为什么"，tsukumo 用慌张举手的小人回答"现在怎么了"、用结算/回放回答"刚才发生了什么"。状态/叙事分层（§6）在 Loomstead 里已有工程原型（实时 trace 事件 vs 事后 eval artifact）。

**2. Motivational Delegation —— 关系层的理论骨架**

Director 从不直接命令 NPC，只做间接干预（`motivation_bias` / `opportunity_schedule` / `information_exposure` / `constraint_injection`），行动由 agent 自己的动机循环产生；Process Fidelity 评测验证"结果是自然长出来的，不是硬改状态"。平移到 tsukumo：**主人培养付丧神不是打开面板改数值，而是给它经历**（想让 Gina 更懂这个 repo，就把涉及它的委托派给她）。这套"动机-干预-过程保真"是路线终态"关系与成长管理"的理论骨架，不只是 post-MVP 闲时素材。

**3. Coding domain adapter 已验证 schema 可迁移**

Loomstead 跨域评测中 coding domain fixture 用同一套 GoalSpec / Intervention / Trace / Eval schema 跑出 40/40 通过、反事实工具选择变化率 0.762（55/55 确定性 seed）。tsukumo 终态本质是把 primary/secondary 翻转：编码域成为主战场，小镇退役为方法论出生地。可迁移性不是猜想，是有数据的结论。

### 17.3 可复用资产清单

| Loomstead 资产 | tsukumo 位置 | 价值 |
|---|---|---|
| NPC 闲时行为环（需求累积→规则选行为） | 工房闲时生态（post-MVP） | 已验证纯规则零 LLM 成本产生生活感 |
| 三层工具路由（生理=规则/职业=规则+LLM/社交=LLM受预算） | 台词三级策略同构 + LLM 升级触发条件 | token 预算纪律现成答案 |
| `failure_modes` 带 `emotional_charge` 的工具 schema | 失败演出化的数据基础 | 直接抄 |
| 双轨主观记忆（客观事件流 + 每 NPC 主观视图） | 每个付丧神对同一事件有不同记忆 | 角色深度来源 |
| Interrupt 机制（interruptible + 优先级阈值） | 小人被打断跑向岗位 | 工房"活着"的关键细节 |
| Director 间接干预词表 | 关系层成长机制 + 世界层随机事件编排 | 理论骨架（见 17.2） |
| trace 可观测性 / 反事实回放 | "Gina 为什么说这句台词"可追溯 + 注入可追溯（§8.6） | 演出与注入系统的调试器 |
| Process Fidelity / Direct State Setter baseline | 养成数值真实性原则的学术版定义与对照组 | "真养成"的可评测定义 |
| 美术管线（AI 生成像素资产 prompt、manifest 组织） | 像素小人资产管线 | 风格关键词/Prompt 模板/命名规范直接复用 |

### 17.4 美术管线适配

Loomstead 经验直接复用：风格定位（二次元轻幻想）、色板（3-5 主色）、头身比（2.0-2.5）、Prompt 模板结构、命名/manifest 组织。

终端字符像素适配调整：
- 轮廓线加粗（1-3 px），强调高对比色块
- 减少细节到 2-3 个最关键识别点
- 无渐变、无 dithering、无细微纹理
- Prompt 新增：`high contrast, distinct color blocks, terminal-friendly, blocky shapes, geometric silhouette, no gradients, no dithering`
- 验证流程：PNG 64x64 → chafa 转字符画 → 不同终端宽度下验证可读性

---

## 18. 求职叙事：Observatory → 产品化接力

2026-07-08 更新：从占位升级为明确的接力叙事。

> **Loomstead（研究）**：建了一个 agent 行为天文台，证明动机式委托、过程保真评测、trace 证据链这套方法论成立，coding domain adapter 证明其可迁移。
> **Tsukumo（产品）**：把这套方法论产品化，切进 2026 年正在爆发的真实痛点——人类从写代码变成指挥 agent 群之后的注意力管理与关系管理。观测层变成像素公会（消费级 Observer Dock），过程保真变成"养成数值全部来自真实数据"的产品原则，动机引擎变成付丧神的成长机制。

这补上"把评测结果转化为上线取舍"的后半句。两个项目共享同一技术信仰：事件溯源、trace 优先、反对硬改状态。

现有项目矩阵：模型 → AlgoCoach；上下文协议 → ContextGuard；Agent harness → Loomstead；推理配置 → Inference Lab；**产品闭环 → Tsukumo**。

> ⚠️ 防绑架条款（保留并强化）：求职叙事是 MVP 成果的**副产品**，不是设计输入。Loomstead 自己的教训（"指标难展示 → 前端继续调 → 需要人工 review"的循环靠冻结规则才脱身）就在眼前：不为叙事完整而把 Process Fidelity 评测、反事实回放等研究件提前搬进 MVP。MVP 只验证"看 agent 干活像看戏"与注意力管理效用，叙事材料等东西能跑了再包装。

**2026-07-13 优先级重申**：本项目初衷是求职技术展示，核心交付是 Loomstead → Tsukumo 的研究-产品接力叙事与技术深度；产品化与市场竞争需求不紧迫——分发冷启动、schema "数据永不作废"对外承诺、依恋假设外部验证（当前 n=1）均降级为 post-V0 观察项，主观取舍按主人喜好优先。防绑架条款反向同样有效：也不为 demo 说服力把研究件提前搬进 V0。

---

## 19. 开放问题

### 2026-07-08 已化解

- [x] 演出节奏 vs 效率 → 状态/叙事分层（§6）
- [x] 项目终态形态 → 有灵魂的指挥所（§2）
- [x] 注入冲突 → 注入协议四解法（§8）

### 2026-07-11 已化解

- [x] 默认界面 → 舞台优先，Inspector 按需展开，Permission 强制 modal
- [x] V0 世界 → 深夜九十九工房
- [x] 主搭档 → 栞（Shiori），九十九工房书记官，称呼用户“会长”
- [x] 主搭档人格 → 冷静严谨、低频温柔、一本正经的幽默
- [x] 主搭档外观 → 书签侧辫、书封披肩、契约簿、青色灵火、朱红封蜡
- [x] soul / guise → 身份、RuntimeBinding、PresentationPack、Skill 分层
- [x] V0 内容包 → 内置 `default-shiori` + 一个外部目录
- [x] 默认界面与栞五态视觉参考 → `docs/visual-references/`

### 2026-07-13 已化解

- [x] MVP go/no-go → 有条件 GO：方向不变、不大改；V0 以"热交接 demo 三硬标准"（§2.7）为验收核心
- [x] 轻量增强层（Trellis 式）是否消解护城河 → 否，但护城河位置修正为寄生层结构性够不着的四件事（§2.7）
- [x] adapter 维护税 → Codex + Claude 两条 headless 真实打通后以实测数据再定 ACP 主通道迁移时机
- [x] 项目优先级 → 求职技术展示优先，产品化/市场竞争非紧迫（§18）

### 仍开放

- [ ] web/panel 第二消费端形态（KernelEvent 消费者，架构零重写；对照 Pixel Agents 的 VS Code 分发红利）
- [ ] 编辑器类 ACP client（Zed / JetBrains / Cursor）对聚合层位置的长期威胁跟踪
- [ ] 分发冷启动：预编译二进制 + 热交接录屏渠道（post-V0，非紧迫）
- [ ] `soul.db` / KernelEvent schema 的"数据永不作废"对外承诺与迁移纪律（post-V0，非紧迫）
- [ ] 依恋假设外部验证（当前 n=1，V0 发布即验证手段；非紧迫）
- [ ] post-V0 执行者角色阵容与内容包迁移（Gina / Term / Fio 等）
- [ ] 多并行任务的完整大厅委托看板
- [ ] ACP 主通道切换时机与观察级 watcher 覆盖范围
- [ ] MCP recall/search 工具面与长期调试快照生命周期
- [ ] 简报容量上限与相关性过滤策略
- [ ] 多角色协作的站位、遮挡与对话编排
- [ ] 记忆沉淀触发阈值与频率
- [ ] post-V0 pack 继承、合并、热重载与社区分发
- [ ] “有意思”的可评测定义
- [ ] 情绪变量、衰减曲线与主动驻留规则

---

## 20. 参考项目与吸收要点

### pi（badlogic/pi-mono）— 内核骨架
- 分层 monorepo：`pi-ai`（零 agent 依赖）→ `pi-agent-core`（事件驱动 loop、steering、并行工具）→ 应用层
- 类型化事件流贯穿每一层；`AgentMessage` 自定义类型 + `convertToLlm` 过滤
- `beforeToolCall / afterToolCall` 钩子；session JSONL 树结构（id/parentId 分支）
- 同一 AgentSession 支撑 interactive / print / RPC / SDK 四种模式

### hermes-agent（Nous Research）— 成长闭环
- 三层记忆：会话（SQLite FTS 跨会话检索）/ 持久（MEMORY.md + USER.md 冻结快照注入、容量表头）/ 技能（SKILL.md）
- Skills 渐进披露三层加载省 token；frontmatter 条件激活
- agent 经 `skill_create / skill_edit` 自建技能，任务后主动建议沉淀
- Curator 定期整理：合并冗余、清理矛盾、把重复流程建议为技能

### hermes-agent-self-evolution — 角色修行
- DSPy + GEPA 读执行轨迹 → 针对性优化 skill/prompt/tool
- guardrails：测试 / size / 语义不漂移 / PR review

### Loomstead — 三件愿景级资产 + 设计资产搭桥
- Trace 可观测性 DNA、Motivational Delegation、coding adapter 迁移数据（见 §17.2）
- NPC 动机引擎、主观记忆、failure_modes 情绪电荷、美术管线（见 §17.3）

### 竞品格局（2026-07-08 调研）
- **Pixel Agents**（VS Code 扩展）：每个 Claude Code 会话一个像素小人，办公室可编辑；1.3 万+下载、Fast Company 报道。验证了环境化信息定位与注意力管理痛点
- **Agent Cockpit**：像素办公室 + 运维面板（审批收件箱/diff/时间线回放/记忆编辑）
- **The Office** / **AgentRoom**：同类变体（宝可梦皮 / 会话全文检索）
- 共同短板 = tsukumo 护城河：无人格持久性、只读观察不拥有会话、无成长机制
- 指挥所赛道邻居：Vibe Kanban、Conductor、Claude Squad 等（管编排、无灵魂）

### 竞品格局增补（2026-07-13 复核）
- **Pixel Agents**：装机 1.3 万 → 7.4 万+；公布 HookProvider 适配层与 agent/平台/主题无关路线（Codex/Gemini/Cursor 在路线图）；仍纯观察、无灵魂——入场券贬值加速的实证
- **Vibe Kanban**：2026-04 被 Bloop 停运，转 Apache-2.0 社区维护——纯效用指挥所变现受挫的实证
- **Buddy / CodePal**（新物种：养成伙伴）：MCP 虚拟宠物 / 桌宠，XP·物种·情绪·跨 client 持久（Buddy 8 周 1 万+ clone）；养成为打卡式假进度条（`buddy_remember` 自标 "Not yet robust"、`buddy_dream` 占位），不拥有会话、无状态连续性——"真养成 vs 假进度条"叙事牌的现实对照组
- **OpenMemory MCP（Mem0）**：本地跨工具记忆基础设施（审计面板、任意 MCP client）——"记忆跨工具"本身不再是差异化卖点
- **ACP 进展**：注册表 2026-01 上线、2026-03 stabilize，25+ agent（含 Cursor）接入——驱动级接入成本低于预期；同时注意编辑器类 ACP client（Zed/JetBrains）自身即拥有会话，列为长期观察项（§2.7）

### ACP（Agent Client Protocol）— runtime 互操作标准
- Zed 发起（2025-08），JetBrains 共同维护，Apache 协议，19+ agent 兼容（Claude Code / Codex / Gemini CLI / opencode / Goose 等）
- tsukumo 作为 ACP client 拥有会话：发委托、收结构化事件、接管权限审批——驱动级接入的主通道

### 抽象 → 叙事 转译表

| hermes 抽象 | tsukumo 拟人化转译 | 叙事含义 |
|---|---|---|
| skill（程序性记忆） | 角色招式/技能 | 角色学会的动作套路，可成长 |
| memory curation | 角色回忆整理 | 付丧神"沉淀修为" |
| user modeling | 羁绊系统 | 角色越来越懂主人 |
| personality | 付丧神人格卡 | 每个工具一个角色 |
| self-evolution (GEPA) | 角色修行 | 台词/演出自动优化，越演越好 |
| periodic nudge | 角色内省时刻 | 定期"打坐回忆"，提炼经验 |
| ACP 权限审批 | 角色犹豫 + 确认门 | 安全机制变成演出 |
| runtime 异构性 | 雇佣冒险者的性格差异 | 集成碎片化变成人设素材 |
