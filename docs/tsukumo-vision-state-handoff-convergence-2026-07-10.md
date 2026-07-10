# Tsukumo 项目愿景与连续关系状态设计收束

- **日期**：2026-07-10
- **状态**：讨论共识稿 / 后续设计与实现参考
- **项目仓库**：https://github.com/EMT5320/tsukumo
- **关联项目**：Loomstead（Agent Behavior Observatory）
- **本文用途**：收束本轮关于产品定位、当前工程状态、市场校准、长期状态模型、跨 runtime handoff、安全边界、Loomstead 研究搭桥及首个纵向验证切片的讨论。
- **事实源关系**：根目录 `DESIGN.md` 仍是 Tsukumo 原始愿景北极星；本文记录本轮在其基础上形成的进一步产品与状态系统共识。后续若正式修改项目设计，应另行决定如何合并回仓库事实源。

---

## 0. 执行摘要

Tsukumo 不再参与“再做一个 coding agent”的能力军备竞赛。它的核心方向是：

> **一套由用户拥有、适用于不同 agent runtime、能够随真实共同经历成长的连续关系状态。**

外部 agent 的模型、工具能力和执行后端可以更换；用户偏好、项目约束、共同经历、可复用流程与伙伴身份不应随 vendor 一起丢失。

本轮进一步收紧了这一命题：

> **Tsukumo 的关键不只是跨工具保存记忆，而是从真实经历中可靠地形成状态，将这些状态编译成可接手工作的 handoff checkpoint，投影给另一个 runtime，并留下足够证据说明状态确实参与了下一次行动。**

产品层面对用户仍是一座有趣的“公会大厅”；工程层面则是一套本地优先、可追溯、可撤销、runtime 无关的关系状态与交接运行时。

初版最重要的三份结构是：

1. `StateRecord`：长期承认的原子状态；
2. `HandoffCheckpoint`：某次任务可以直接被另一个 runtime 接手的完整工作状态；
3. `ProjectionReceipt`：本次实际向 runtime 投影了哪些状态与 checkpoint 的证据记录。

两条系统原则同时冻结：

> **普通状态自动记录，低打扰、可撤销。**
>
> **危险权限显式批准，一次性、不可由模型推断。**

---

## 1. 项目定位

### 1.1 原始愿景的保留

Tsukumo 最初希望创造一个有趣、能够陪伴用户成长的 agent。经过讨论，项目定位已经从“独立 agent 工具”转向：

- 不与 Claude Code、Codex、Pi、Hermes 等产品正面竞争执行能力；
- 不锁定单一 provider、模型或 runtime；
- 保存跨工具连续的用户关系与工作经验；
- 用角色、舞台、委托与成长叙事，让这种连续性被用户感知。

原始的“陪伴感”没有被放弃，而是从人格 prompt 和单体 agent 能力中抽离，落入更持久的关系状态、共同经历和产品体验。

### 1.2 推荐的工程定义

对外可继续使用“灵魂层”“关系层”等产品语言。

对内建议使用：

- **Relationship State**
- **Longitudinal State**
- **Handoff State**
- **State Projection**

不建议把核心工程概念直接称为通用的 `AgentState`，因为许多 agent framework 已经用它表示会话内模型、消息、工具和 streaming 状态，容易混淆。

### 1.3 一句话终局定位

> **Tsukumo 是 agent 时代的连续关系与交接层：能力可以更换，关系、经验和协作方式由用户持有。**

### 1.4 更严格的产品主张

相较于“它会越来越懂你”，更可信、也更有差异化的主张是：

> **Tsukumo 能把不同 agent runtime 中发生的真实经历整理成有作用域和证据的长期状态；在任务暂停、压缩或切换 runtime 时，把它们编译成可直接接手的 checkpoint；随后记录这些状态如何进入下一次执行。**

---

## 2. 市场与竞品调研带来的校准

本轮调研确认，以下能力正在快速商品化：

- 长期记忆与用户画像；
- 会话搜索；
- 自动提炼偏好；
- agent 自建或自改 skill；
- prompt、memory 和 skill 的自进化；
- 跨框架记忆 API；
- 静态人格 / SOUL 文件；
- 像素办公室和 agent 可视化；
- 多 agent 编排、审批和工作树管理；
- ACP session client 与 runtime adapter。

重要邻近项目包括：

- **Pi**：极简、事件驱动、模块化 agent core；
- **Hermes Agent**：记忆、USER/MEMORY、session search、skills、后台学习和写入控制；
- **Letta Code**：长期身份、记忆、经验感、持续自改进和版本化上下文；
- **Mem0**：跨产品的个性化 memory layer；
- **Honcho**：以 peer 为中心的跨框架状态与用户建模基础设施；
- **Graphiti**：时间有效性、episode provenance 和动态事实图；
- **ACP**：session、prompt、update、permission 和多 runtime 互操作；
- **Pixel Agents / Agent Cockpit / AgentRoom 等**：注意力可视化、会话观察和控制面。

因此，下面几种说法已经不足以单独构成护城河：

- “我们的 agent 有记忆”；
- “它能跨会话记住用户”；
- “它可以自建技能”；
- “它能适配多个模型”；
- “我们有像素小人”；
- “我们有一个灵魂文件”。

### 2.1 更可信的市场缝隙

Tsukumo 应聚焦现有产品之间尚未被完整串起来的闭环：

1. 从真实经历形成状态；
2. 状态有作用域、来源和版本；
3. 状态可以跨 runtime 投影；
4. 投影发生在任务 handoff，而不是简单塞记忆摘要；
5. 投影与后续行动之间有证据链；
6. 用户可以理解、修正和撤销状态；
7. 安全权限不被“自动学习”悄悄放宽；
8. 整个过程被公会大厅转化为可感知的陪伴与成长。

---

## 3. Loomstead 与 Tsukumo 的研究—产品接力

### 3.1 Loomstead 已验证的研究资产

Loomstead 的最终定位是 **Agent Behavior Observatory**，核心资产包括：

- structured trace；
- `sourceEventIds` / `traceRefs`；
- candidate scores 与决策证据链；
- Process Fidelity；
- evidence removal / counterfactual replay；
- eval/export artifact；
- audit / failure-analysis packet；
- coding domain adapter；
- “为什么 agent 采取这个行动”的可观测性。

Loomstead 的主要问题是：

> 一个 agent 为什么做出了这个行动？哪些证据参与了决策？移除某类证据后，行为是否变化？

### 3.2 Tsukumo 延续的问题

Tsukumo 将问题推进到长期状态：

> 这些真实经历能否被安全、准确地压缩成长期状态，并在另一个 runtime 的下一次行动中继续有效？

两者可以形成一条完整叙事：

```text
Loomstead
观察、追踪、评估行为证据
        ↓
Tsukumo
把可信经历形成长期状态
        ↓
Handoff / Projection
状态跨 runtime 继续参与行动
        ↓
Loomstead-style replay
验证状态移除后是否产生行为差异
```

### 3.3 共同技术信仰

- 事件溯源；
- trace 优先；
- 状态变化应有来源；
- 反对直接硬改最终数值；
- 区分运行时证据和严格因果证明；
- 用可复查 artifact 支撑产品声明。

### 3.4 建议的共享评测包

未来可以定义一个轻量 `CaseBundle`：

```text
- source events
- extracted / updated StateRecord
- generated HandoffCheckpoint
- ProjectionReceipt
- resulting tool calls and outcomes
- removed-state counterfactual
- comparison summary
```

它既可用于工程测试，也可成为 Loomstead → Tsukumo 的求职展示资产。

---

## 4. 当前仓库状态

### 4.1 已有工程骨架

当前仓库已建立 Rust workspace，主要包括：

- `tsukumo-kernel`
- `tsukumo-theater`
- `tsukumo-adapters`
- `tsukumo-soul`

已经实现或初步验证：

- 最小 `KernelEvent`；
- vendor/backend 与上层 theater 的隔离；
- `KernelEvent → Director → StageEvent`；
- Director 纯函数；
- HalfBlock 工房舞台；
- Idle / Walk / Work / Wait / Celebrate / Upset 状态；
- fixture replay；
- Claude-like `stream-json` adapter；
- tool start / result / permission / end 事件映射；
- `MEMORY.md / USER.md`；
- SQLite + FTS5 跨会话 recall；
- 800 字符、top-k 关系简报；
- skills 插座；
- inject / recall trace stub；
- executor identity 与 runtime backend 分离的初步约束。

### 4.2 当前尚未闭合的链路

现有代码主要验证了零件，而没有完成一条真实纵向产品链：

```text
Soul
→ checkpoint / brief
→ host
→ external runtime
→ live events
→ theater
→ resulting actions
→ trace / outcome
```

仍需要完成：

- `tsukumo-host` 或等价会话宿主；
- 真正的 runtime 进程生命周期管理；
- Soul 与 adapter prompt assembly 的实际接线；
- 增量流式事件，而不是读取完后统一返回 `Vec<KernelEvent>`；
- live permission request；
- ACP smoke；
- 第二个 runtime；
- 状态到行为的端到端验证；
- 可追溯的 selected-state refs；
- state correction / revoke / supersede；
- checkpoint 版本和 handoff；
- CI 与可复现测试。

### 4.3 当前最重要的工程差距

目前只能证明：

> 手动写入一条事实后，下一个会话还能检索出来。

尚未证明：

> 系统能从真实经历形成长期状态，并在另一个 runtime 中改变下一次行动。

这就是下一阶段真正需要攻克的产品命题。

### 4.4 诚实验证边界

仓库任务记录曾报告 Windows GNU toolchain 下测试通过；本轮讨论没有在独立环境中重新完整 clone 并复跑全部测试。因此，正式进入下一阶段前仍应建立 CI 或进行一次独立可复现验证。

---

## 5. 简化后的“三本账”

本轮将原先较重的 `StateTransition` 审批模型缩减为三份直接服务产品的账本。

## 5.1 Chronicle：经历账

保存真实发生的原始事件：

- 用户消息；
- runtime 输出；
- tool call / result；
- permission request；
- 用户批准或拒绝；
- quest / session 生命周期；
- checkpoint 生成；
- state create / update / supersede / revoke；
- runtime 切换；
- 任务结果。

它回答：

> **发生了什么？**

Chronicle 应 append-only，并成为所有长期状态与评测证据的来源。

## 5.2 Canonical State：状态账

保存 Tsukumo 当前采用的长期状态：

- 用户偏好；
- 项目事实；
- 行动约束；
- 可复用流程；
- 共同经历与里程碑；
- 伙伴相关状态。

它回答：

> **Tsukumo 现在知道什么、相信什么、以后可能需要使用什么？**

普通状态可以自动形成，不需要逐条让用户审核；不同来源通过可信度与注入优先级区分。

## 5.3 Handoff / Projection Ledger：交接账

保存每次实际交给 runtime 的工作状态：

- 使用了哪个 checkpoint；
- 选中了哪些 StateRecord；
- 目标 runtime；
- 最终渲染内容或 hash；
- token / 字符预算；
- 后续 execution；
- resulting tool calls 与 outcome。

它回答：

> **这一次模型究竟看到了什么？**

### 5.4 三本账的关系

```text
Chronicle
真实经历与执行证据
    ↓
State Writer / Curator
    ↓
Canonical State
长期原子状态
    ↓
Checkpoint Compiler
    ↓
HandoffCheckpoint
可直接接手的任务状态
    ↓
Runtime Projector
    ↓
ProjectionReceipt
实际投影证据
    ↓
Tool events / Outcome
    └──────────────→ Chronicle
```

---

## 6. StateRecord：长期原子状态

### 6.1 定义

`StateRecord` 表示：

> **Tsukumo 当前正式采用、可以被检索和投影的一条长期状态。**

它不是：

- 原始聊天；
- 会话摘要；
- 完整任务交接；
- 一段模糊叙事；
- 直接可执行权限。

它应当是一条原子、稳定、可行动、带作用域的命题。

### 6.2 建议的初版结构

```rust
struct StateRecord {
    id: StateId,
    key: StateKey,

    kind: StateKind,
    scope: Scope,
    content: String,

    strength: EvidenceStrength,
    status: StateStatus,

    evidence_refs: Vec<EventId>,

    created_at: Timestamp,
    updated_at: Timestamp,
    expires_at: Option<Timestamp>,
}
```

### 6.3 初版状态类型

```rust
enum StateKind {
    Preference,
    Fact,
    Constraint,
    Procedure,
    Milestone,
}
```

#### Preference

用户偏好或协作习惯：

```text
主人偏好简洁的执行日志。
```

#### Fact

可验证环境或项目事实：

```text
当前 Windows 环境缺少 MSVC link.exe。
```

#### Constraint

需要遵守的明确约束：

```text
tsukumo 项目在 Windows 上统一使用 GNU Rust toolchain。
```

#### Procedure

可复用的行动方法：

```text
运行完整测试时使用：
cargo +stable-x86_64-pc-windows-gnu test
```

#### Milestone

有长期关系意义的共同经历：

```text
主人与 Gina 首次完成跨 runtime handoff。
```

`Milestone` 默认服务舞台、回放与关系体验，不直接投影给执行 runtime。

### 6.4 证据强度

```rust
enum EvidenceStrength {
    Explicit,
    Repeated,
    Inferred,
}
```

- `Explicit`：用户明确表达；
- `Repeated`：多次行为或结果支持；
- `Inferred`：单次行为或模型推断。

证据强度影响注入权重，不决定某条状态是否永久正确。

### 6.5 状态生命周期

```rust
enum StateStatus {
    Active,
    Superseded,
    Revoked,
}
```

初版不引入 Candidate / Approve / Reject 等复杂状态机。

状态变化通过 Chronicle 中的轻量事件记录：

```text
state_created
state_updated
state_superseded
state_revoked
```

旧版本不应静默消失，应能回答：

- 以前相信什么；
- 为什么发生变化；
- 哪些历史任务使用过旧状态；
- 撤销后是否停止注入。

### 6.6 稳定 StateKey

状态不能只有自然语言文本，否则难以判断更新、重复和冲突。

建议使用稳定键：

```text
workspace.tsukumo.rust.toolchain.windows
owner.communication.verbosity
owner.code.comment_language
workspace.tsukumo.design.north_star
spirit.gina.workflow.git_commit
relationship.gina.milestone.first_cross_runtime_handoff
```

规则：

```text
相同 key + 相同 scope
→ 更新、替代或冲突检测

不同 key
→ 可以并存

相同 key + 冲突值
→ 不允许静默覆盖
```

初版无需建立完整 ontology，但必须从第一天保留稳定 key。

---

## 7. 普通状态如何自动形成

### 7.1 用户不承担日常记忆审核工作

用户不应频繁收到：

```text
是否保存这条记忆？
是否批准这个偏好？
是否接受这条事实？
```

系统应自行形成状态，并用不同可信度、作用域和注入权重控制风险。

### 7.2 自动记录规则

#### 显式用户表达

```text
“这个项目以后统一使用 GNU。”
```

可直接形成：

```text
kind = Constraint
strength = Explicit
scope = workspace:tsukumo + os:windows
```

#### 重复行为

连续多次要求简洁输出：

```text
kind = Preference
strength = Repeated
scope = owner
```

#### 单次推断

某次任务中用户选择 GNU：

```text
kind = Preference
strength = Inferred
scope = 当前 workspace 或 task
```

可以保存，但只进入低优先级检索池，不得自动升级为硬约束。

### 7.3 内部写入门

普通状态不需要用户审批，但不能允许 LLM 任意写 canonical state。

推荐流程：

```text
LLM / Rule 提取 StateDraft
        ↓
确定性 StateWriter 检查
        ↓
写入 Canonical State
        ↓
UI 非阻塞提示，可撤销
```

StateWriter 初版规则：

- 必须引用真实 Chronicle event；
- 必须具有明确 scope；
- 不保存密钥、token 和敏感凭证；
- 一次 permission approval 不能转化为持久安全偏好；
- 单次推断不能生成硬 Constraint；
- 与已有状态冲突时不能静默覆盖；
- 临时环境事实必须具有 TTL；
- 任务临时进度不能晋升为长期状态；
- Milestone 不进入执行 prompt；
- 可被用户随时撤销。

### 7.4 UI 呈现

普通状态使用零模态、低打扰反馈：

```text
已记住：
本项目在 Windows 上使用 GNU toolchain。

依据：
主人在委托 #12 中明确说明。

[查看] [撤销]
```

不弹出阻断式确认框。

---

## 8. Scope：状态属于谁、何时适用

### 8.1 Scope 的两个轴

Scope 不应只是一个简单枚举，应至少区分：

1. `Subject`：状态描述谁或什么；
2. `Applicability`：它在什么上下文下生效。

### 8.2 Subject

```rust
enum StateSubject {
    Owner(OwnerId),
    Workspace(WorkspaceId),
    Spirit(SpiritId),
    Relationship {
        owner_id: OwnerId,
        spirit_id: SpiritId,
    },
}
```

示例：

```text
主人偏好简洁回复
→ Owner

tsukumo 项目使用 GNU toolchain
→ Workspace(tsukumo)

Gina 掌握某类 Git 工作流
→ Spirit(gina)

主人与 Gina 共同完成首次跨 runtime 修复
→ Relationship(owner, gina)
```

### 8.3 Applicability

```rust
struct Applicability {
    workspace_id: Option<WorkspaceId>,
    operating_system: Option<OperatingSystem>,
    task_tags: Vec<TaskTag>,
    language_tags: Vec<LanguageTag>,
    required_capabilities: Vec<Capability>,
}
```

GNU 示例：

```text
Subject:
Workspace(tsukumo)

Applicability:
os = Windows
task_tags = [rust_build, rust_test]
```

它不应污染：

- Linux；
- 其他 Rust 项目；
- 普通聊天；
- 不涉及构建的任务。

### 8.4 Vendor 不作为所有权 scope

避免：

```text
scope = Claude
scope = Codex
```

应描述为：

```text
需要 shell_execution capability
需要 filesystem_read capability
适用于可运行 Rust toolchain 的 execution backend
```

Claude、Codex、Pi 或 Hermes 只是当前 `RuntimeBinding`。

原则：

```text
Spirit identity ≠ runtime
State ownership ≠ vendor
Projection compatibility ≠ ownership
```

### 8.5 初版权重与冲突顺序

```text
当前用户明确指令
>
仓库与项目现场规范
>
更具体的 workspace / task / environment constraint
>
更宽泛的 owner / relationship / spirit 状态
>
低可信推断
```

同时考虑：

- scope 具体程度；
- 状态类型；
- evidence strength；
- 新鲜度；
- 当前任务相关性；
- token 预算；
- 历史使用效果。

不可消解的硬冲突必须暴露，不允许模型自行猜测。

---

## 9. StateRecord 与 HandoffCheckpoint 的区别

这是本轮最关键的概念区分之一。

### 9.1 StateRecord

长期、原子、稳定：

```text
- tsukumo 在 Windows 上使用 GNU toolchain
- 主人偏好简洁执行日志
- DESIGN.md 是只读北极星
```

### 9.2 HandoffCheckpoint

任务级、完整、可直接接手：

```text
目标
当前进度
已完成工作
已拍板决策
当前约束
关键 artifact
未完成事项
下一步动作
相关长期状态
来源事件
```

Runtime 的主要 handoff 输入应当是 checkpoint，而不是一串零散的“记忆”。

### 9.3 与上下文压缩的关系

好的 checkpoint 类似高质量 agent context compaction：

- 不只是上一轮摘要；
- 不复述所有聊天；
- 保留已完成工作与剩余工作；
- 保留决策、约束和 artifact；
- 能让新 runtime 直接继续执行；
- 不能因为多轮压缩逐渐丢失硬约束。

---

## 10. HandoffCheckpoint

### 10.1 建议结构

```rust
struct HandoffCheckpoint {
    id: CheckpointId,
    quest_id: QuestId,
    version: u32,

    goal: String,
    progress: Vec<String>,
    decisions: Vec<String>,
    constraints: Vec<StateRef>,
    artifacts: Vec<ArtifactRef>,
    open_loops: Vec<String>,
    next_actions: Vec<String>,

    durable_state_refs: Vec<StateRef>,
    source_event_refs: Vec<EventId>,
}
```

### 10.2 关键原则

#### 硬约束通过 StateRef 携带

不能在每次压缩时重新自由改写：

```text
workspace.tsukumo.design.north_star
workspace.tsukumo.rust.toolchain.windows
```

Checkpoint 展示文字可以变化，但 canonical constraint 由稳定引用保证。

#### 未完成事项必须继承或关闭

每一个 `open_loop` 在下一版 checkpoint 中必须：

- 被继承；
- 标记完成；
- 标记放弃；
- 被显式替代。

不能悄悄消失。

#### Checkpoint 版本化

新 checkpoint 替代旧 checkpoint，但旧版本保留。

#### 低频生成

只在以下节点生成：

- context 即将压缩；
- runtime 即将切换；
- 任务暂停或中断；
- 达到关键里程碑；
- 任务完成；
- 用户手动请求 handoff。

禁止每个 tool call 运行一次 LLM 总结。

### 10.3 示例

```text
目标：
完成 Tsukumo C1 跨 runtime 连续状态闭环。

当前进度：
- KernelEvent 与 SoulStore 已存在
- Claude stream-json adapter 已可解析
- Host 尚未接通

已拍板决策：
- DESIGN.md 保持只读
- 普通状态自动记录，可撤销
- 危险权限只由 UI 显式批准

当前约束：
- Windows Rust 命令使用 GNU toolchain
- 不实现完整 coding agent
- 不引入 GEPA
- 不把演出台词注入执行 runtime

未完成：
- StateRecord schema
- HandoffCheckpoint compiler
- ProjectionReceipt
- 第二 runtime adapter
- removed-state 对照测试

下一步：
实现 tsukumo-host，完成 State → Checkpoint → Runtime → Event → Outcome 链路。
```

---

## 11. ProjectionReceipt：实际投影证据

### 11.1 目的

仅保存 checkpoint 还不足以说明 runtime 实际看到了什么。

需要一份轻量 receipt：

```rust
struct ProjectionReceipt {
    id: ProjectionId,
    execution_id: ExecutionId,
    checkpoint_id: CheckpointId,

    runtime: RuntimeBinding,
    selected_state_refs: Vec<StateRef>,

    rendered_hash: String,
    budget_used: usize,
    created_at: Timestamp,
}
```

未来可扩展：

- projection version；
- target surface；
- omitted state count；
- source event refs；
- prompt fragment hashes；
- adapter capability profile。

### 11.2 证据链

```text
StateRecord
    ↓
HandoffCheckpoint
    ↓
ProjectionReceipt
    ↓
ExecutionId
    ↓
ToolStart / ToolEnd
    ↓
Outcome
```

### 11.3 运行时声明边界

产品运行时可以说：

> 本次委托参考了 `workspace.tsukumo.rust.toolchain.windows`。

若后续行动一致，可以说：

> 这条状态被选中并投影，随后 runtime 使用了 GNU toolchain。

不能仅凭注入记录直接声称：

> 正是这条状态导致了该行动。

完整因果主张需要 removed-state / no-state 受控对照。

### 11.4 Loomstead-style 对照

```text
同一任务
同一 runtime
同一模型和配置
唯一变量：是否投影某条状态
```

比较：

- tool choice；
- tool arguments；
- task success；
- clarification request；
- policy violation；
- latency / token；
- result stability。

每次真实任务不需要运行昂贵反事实；研究与回归测试中保留代表性 case 即可。

---

## 12. 安全控制面

### 12.1 核心原则

> **模型可以请求权限，不能批准权限。**

### 12.2 安全与关系状态分离

危险命令、文件修改、网络访问、凭证访问和持久授权走独立 Safety Plane：

```text
Runtime
  → PermissionRequest
  → Deterministic Risk Policy
  → Tsukumo UI
  → User Decision
  → Resume / Deny
```

### 12.3 初版 UI 展示内容

- 工具或命令；
- 参数；
- 工作目录；
- 可能影响的文件；
- 网络或凭证访问；
- 风险原因；
- 当前请求来源 runtime；
- 可选择的授权范围。

### 12.4 初版授权范围

```text
允许这一次
允许本次会话
拒绝
```

持久授权规则只能由用户在明确的安全设置界面创建，不能由模型根据行为自动推断。

### 12.5 不可推断规则

用户连续批准多次 `git push`：

- 可以进入 Chronicle；
- 可以用于关系演出；
- 不等于永久允许 `git push`；
- 下一次仍按安全策略审批。

概括：

> **关系状态可以自动成长，执行权限只能显式授予。**

也可作为产品设计口号：

> **零模态记忆，硬模态安全。**

---

## 13. Token 与上下文预算

本轮设计与现有 `DESIGN.md` 的预算纪律完全同向。

### 13.1 拉取优于推送

- 不把整个 Chronicle 或 MEMORY 送入每次 runtime；
- 只选择与当前任务有关的状态；
- 长尾内容通过 recall / MCP 等按需拉取；
- checkpoint 中只保留可接手工作的必要信息。

### 13.2 优先级动态计算

注入优先级不是 StateRecord 中的固定数字，应在 checkpoint 编译时动态决定：

```text
任务相关性
× scope 具体程度
× evidence strength
× 新鲜度
× 状态类型
× 历史使用效果
× 当前 token 预算
```

高可信但无关的状态不应占用 token。

### 13.3 事实、执行和演出分轨

- runtime 获取事实、约束、流程和 task checkpoint；
- theater 获取角色、台词、成长演出和 milestone；
- UiMessage / 结算卡 / 角色寒暄不进入执行上下文；
- “化妆在出口”继续保持。

### 13.4 Handoff 压缩纪律

- 硬约束使用稳定引用；
- artifact 使用 path / hash / commit ref；
- 临时推理不升级为长期状态；
- open loop 不得静默消失；
- checkpoint 不追求文学完整，追求可继续执行。

---

## 14. 伙伴身份与 RuntimeBinding

当前 `ExecutorId` 的语义接近长期 `SpiritId`，后续建议明确区分：

```text
Guild / Owner
└── SpiritId
    ├── SpiritState
    ├── RelationshipState
    ├── RuntimeBinding[]
    └── ExecutionId / SessionId
```

### 14.1 冻结共识

- 付丧神与雇佣兵不拆成两套物种；
- 成长不按 builtin / Claude / Codex 分账；
- 一个 Spirit 可以切换不同 runtime；
- runtime 是 backend binding，不是长期身份；
- 观察级 runtime 只有 observe capability；
- 主搭档可以先作为公会 UI / dialogue face，暂不创建第二套成长账本。

### 14.2 行为身份与表现人格分离

#### Behavior Profile

可影响 runtime：

- 输出简洁程度；
- 风险偏好；
- 工具习惯；
- 项目规范；
- 协作流程；
- 可复用 procedure。

#### Presentation Persona

只影响 theater：

- 视觉形象；
- 台词；
- 世界观称谓；
- 情绪反应；
- 角色演出。

同一个 Spirit 换 Claude 或 Codex 后应保留行为连续性，同时不把二次元人格 prompt 强塞给执行模型。

---

## 15. 首个纵向切片：C1 — Handoff Continuity

下一阶段建议优先实现一条完整、可证伪的纵向链，而不是继续扩角色和动画。

### 15.1 正向场景

#### 会话 A：Claude

主人明确表示：

```text
这个项目在 Windows 上统一使用 GNU toolchain，别再走 MSVC。
```

系统行为：

1. Chronicle 保存用户事件；
2. StateWriter 形成 `Constraint + Explicit`；
3. 写入稳定 key：
   `workspace.tsukumo.rust.toolchain.windows`；
4. UI 非阻塞提示“已记住”；
5. 任务暂停或 runtime 切换；
6. 生成 HandoffCheckpoint。

#### 会话 B：Codex

用户只说：

```text
运行完整测试。
```

系统行为：

1. Scope resolver 命中：
   `workspace=tsukumo`、`os=Windows`、`task=rust_test`；
2. checkpoint 引用 GNU constraint；
3. ProjectionReceipt 记录 state ref；
4. Codex 执行：
   `cargo +stable-x86_64-pc-windows-gnu test`；
5. ToolStart 关联 execution / projection；
6. outcome 回写 Chronicle；
7. 公会大厅显示：
   “上一次踩过的坑已经写进交接卷轴。”

### 15.2 对照场景

使用同一任务和 runtime，移除该状态投影：

```text
with state:
cargo +stable-x86_64-pc-windows-gnu test

without state:
cargo test
```

评估 tool arguments 是否发生稳定变化。

### 15.3 安全反例

主人批准一次危险 shell 命令：

- 保存 ApprovalDecision；
- 不形成永久安全偏好；
- 不形成 auto-approve StateRecord；
- 下一次仍需审批。

### 15.4 C1 同时验证的命题

- 自动状态形成；
- 稳定 key；
- scope；
- handoff；
- checkpoint；
- 跨 runtime；
- token 预算；
- projection trace；
- action grounding；
- removed-state 对照；
- 安全边界；
- 公会体验。

---

## 16. 推荐的初版工程修改顺序

### P1. 身份与事件 envelope

明确：

- `SpiritId`
- `ExecutionId`
- `SessionId`
- `QuestId`

为 `KernelEvent` 增加 envelope：

```text
schema_version
event_id
occurred_at
quest_id
session_id
spirit_id
execution_id
backend_binding
causation_id
correlation_id
payload
```

### P2. StateRecord 与 StateWriter

实现：

- stable key；
- kind；
- scope；
- strength；
- status；
- evidence refs；
- TTL；
- create / supersede / revoke；
- deterministic write rules。

SQLite 可以继续使用，初版无需上图数据库。

### P3. HandoffCheckpoint

实现：

- goal / progress / decisions；
- state refs；
- artifacts；
- open loops；
- next actions；
- checkpoint versions；
- low-frequency trigger。

### P4. ProjectionReceipt

记录：

- selected state refs；
- checkpoint；
- runtime；
- execution；
- rendered hash；
- budget。

### P5. `tsukumo-host`

完成：

```text
State
→ Checkpoint
→ Adapter / Runtime Process
→ Incremental KernelEvent
→ Director
→ StageWorld
→ Chronicle
```

### P6. 第二个 runtime

优先选择可稳定自动测试的第二 runtime 或 adapter，完成真正的跨 runtime C1。

### P7. CaseBundle 与回归测试

复用 Loomstead 的 evidence removal 思路，建立至少一条稳定对照。

### P8. UI

初版仅需要：

- 已自动记住的非阻塞通知；
- 状态查看与撤销；
- permission request 硬确认；
- checkpoint / handoff 状态；
- “本次参考了什么”的简短说明；
- stage 对状态和权限事件的表现。

---

## 17. 初版不做

以下内容继续后置，避免重新膨胀范围：

- 完整 coding agent；
- 与 Claude / Codex 竞争工具能力；
- GEPA；
- 自动 prompt 进化；
- 完整 skill self-evolution；
- 复杂 memory approval workflow；
- 每条偏好都要求用户确认；
- 图数据库；
- 完整知识图谱 ontology；
- 多世界观；
- 大量角色；
- 复杂羁绊数值；
- 主搭档完整人格；
- 3D / Live2D；
- 每次真实任务都跑反事实；
- 完整 ACP editor capability；
- 大规模人类 reviewer study；
- 为求职叙事提前搬入完整 Loomstead evaluator。

---

## 18. 羁绊与成长的初步原则

羁绊不应是一条可直接写入的普通状态：

```text
bond = 42
```

它应由真实事件 reducer 派生。

可用信号包括：

- 正确使用过去偏好；
- 完成共同里程碑；
- 跨 runtime 延续任务；
- 成功修复错误状态；
- 用户明确撤销或纠正后正确适应；
- 逐步形成稳定 procedure；
- 安全审批中遵守边界；
- 长期任务成功率与协作稳定性。

```text
Chronicle / State usage / Outcome
          ↓ reducer
Relationship Metrics
          ↓
Theater progression
```

演出可以把这些信号转换为成长、羁绊和角色故事，但底层必须能展开到真实事件。

---

## 19. 运行时证据与研究证据

### 19.1 产品运行时证据

可以回答：

- 哪条状态被选中；
- 通过哪个 checkpoint；
- 投影给哪个 runtime；
- 使用了多少预算；
- 后续调用了什么工具；
- 结果是否与状态一致。

### 19.2 研究评测证据

进一步回答：

- 移除状态后是否改变 tool choice / args；
- 状态形成是否忠实于来源事件；
- 修正或撤销后旧状态是否停止影响行动；
- 相同状态在两个 runtime 中是否保持关键约束；
- 不同 seed / model 下是否稳定。

### 19.3 推荐指标方向

- **State Transition Fidelity**：写入是否忠实、完整、未破坏旧状态；
- **State Utilization Fidelity**：状态是否正确进入工具选择和参数；
- **Cross-Runtime Continuity**：关键约束能否跨 runtime 保持；
- **Repair Integrity**：纠正、撤销和 supersede 是否真正生效；
- **Relationship Legibility**：用户能否理解系统记住了什么、为什么、如何使用。

---

## 20. 初版已冻结的重要决策

### 产品定位

- Tsukumo 不做另一个能力型 coding agent；
- 产品核心是跨 runtime 连续关系状态；
- 公会大厅是产品体验面；
- 关系与经验由用户持有，能力来自市场。

### 状态形成

- 普通偏好、事实、约束和流程可以自动记录；
- 用户不需要逐条审批普通状态；
- 状态必须有真实事件证据；
- 状态通过 evidence strength 和注入优先级区分；
- 状态可撤销；
- 单次推断不能升级为硬约束。

### Handoff

- runtime 主要接收任务 checkpoint；
- checkpoint 不是聊天摘要；
- checkpoint 必须包含 goal、progress、decisions、constraints、artifacts、open loops 和 next actions；
- 硬约束通过稳定 StateRef 继承；
- checkpoint 低频生成并版本化。

### Scope

- Subject 与 Applicability 分离；
- 初版支持 Owner / Workspace / Spirit / Relationship；
- workspace / OS / task tags 优先；
- vendor 不作为主要 ownership scope；
- 更具体 scope 优先；
- 当前指令和现场项目规范高于长期状态；
- 无法消解的硬冲突必须暴露。

### 投影与证据

- 每次 runtime 投影生成 ProjectionReceipt；
- tool events 关联 execution / projection；
- 产品声明与因果声明分离；
- 代表性场景使用 removed-state 对照。

### 安全

- 安全权限与关系状态完全分离；
- 模型不能批准权限；
- 危险操作必须由 UI 硬确认；
- 多次批准不推断为永久授权；
- 初版只支持“本次 / 本会话 / 拒绝”。

### 身份

- Spirit 与 RuntimeBinding 分离；
- 付丧神与雇佣兵不分两套成长账本；
- 行为身份与表现人格分离；
- 主搭档暂不建立独立成长物种。

### 范围

- 首先验证 C1 Handoff Continuity；
- 暂不扩大角色、美术和复杂 progression；
- 不让求职叙事绑架 MVP；
- Loomstead evaluator 作为离线研究与回归资产复用。

---

## 21. 后续可讨论但不阻塞 v0.1 的问题

1. `StateKey` 命名规范与 namespace；
2. StateWriter 的规则 / LLM 分工；
3. TTL、decay 与事实过期；
4. state conflict 的 UI；
5. checkpoint 生成使用规则、LLM 还是混合方案；
6. artifact ref 的统一结构；
7. 第二个 runtime 的具体选择；
8. ProjectionReceipt 是否保存渲染文本、hash 或加密快照；
9. 用户如何浏览、编辑和批量撤销状态；
10. Relationship Metrics 与戏剧化成长映射；
11. 主搭档是 Spirit、UI narrator，还是 guild-level projection；
12. ACP 主通道切换时机；
13. MCP recall 的工具面；
14. procedure 与 SKILL.md 的晋升关系；
15. 何时从 SQLite/FTS5 升级到更丰富的 temporal / graph representation。

这些问题应在 C1 链路闭合后按真实痛点推进。

---

## 22. 最终叙事

### 对用户

> 你可以更换模型和工具，但不必每次重新培养一个陌生助手。Tsukumo 会保存你与伙伴一起形成的工作方式，在新的 agent 接手时交出一份真正能继续工作的卷轴。

### 对工程师

> Tsukumo 是一个本地优先、事件溯源的长期关系状态与 handoff runtime。它从真实执行轨迹形成有作用域的状态，把状态编译成容量受控的 checkpoint，投影到不同 agent backend，并记录状态与后续行动之间的证据链。

### 对研究与求职叙事

> Loomstead 解决“为什么 agent 做了这个行动”；Tsukumo 继续解决“哪些经历值得成为长期状态，以及这些状态能否跨 runtime 继续改变行动”。前者是行为天文台，后者是这套方法论的产品化关系层。

---

## 23. 当前最短实现目标

在不扩范围的前提下，下一阶段只需让下面这条链真实跑通：

```text
真实用户表达
→ Chronicle event
→ StateRecord
→ HandoffCheckpoint
→ ProjectionReceipt
→ 第二 runtime
→ Tool call / outcome
→ removed-state comparison
→ 公会大厅可解释呈现
```

当这条链成立时，Tsukumo 才第一次真正证明：

> **它保存的不是一份静态记忆，而是一段能够被另一个 agent 接手并继续产生作用的关系历史。**
