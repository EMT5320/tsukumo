# Trellis 强基线与可信交接效用门

> 日期：2026-07-13
> 状态：已讨论并预注册；用于 `07-10-c1-cross-runtime-evidence` 的产品与研究决策
> 决策截止：2026-07-23（AlgoCoach workshop 结果公布、简历开始投递）

## 1. 执行结论

Tsukumo 继续推进到 2026-07-23，但这是一轮有明确停止条件的验证，不代表当前已经证明产品护城河。

- **项目目标**：求职作品集优先，形成 AlgoCoach / Loomstead / Tsukumo 的“研究发现 → 产品假设 → 真实工作流验证”叙事。
- **核心能力**：若必须二选一，保留“可信热交接”；陪伴、养成和权限演出在本轮让位。
- **最强基线**：Trellis，而非无任何增强的裸 agent。周末开发已经证明仓库级规范、任务制品、上下文注入和多工具协调可以覆盖大量连续性需求。
- **当前判断**：跨工具协调、上下文连续、多模型切换本身不构成护城河；“带来源、作用范围和撤销语义的执行状态迁移，是否降低真实续接与恢复成本”仍是待证假设。
- **停止纪律**：若自动迁移相对 Trellis 没有可观察收益，或完整追溯相对隐藏追溯没有恢复收益，应收缩为轻量 sidecar，必要时停止重型 runtime / TUI 产品化。

## 2. 周末开发对原护城河叙事的冲击

周末数万行实现的事实支持以下判断：

1. 仓库内规范、任务文件、会话记录和强模型已经能完成高质量的跨工具协作。
2. Trellis 当前能力不应简化成“Markdown 约定”。上下文注入与 channel runtime 已能启动、观察、等待和中断不同模型的 worker。
3. 本仓库未找到足以重建周末过程的 channel 事件日志，因此只能把成功归因到“用户调度 + 共享制品 + Trellis 约束”的整体，不能单独声称某个 channel 机制贡献了多少。
4. 从用户体验看，这种切换已经接近热交接；从协议语义看，它主要仍是任务边界的快速冷 / 温启动。这个差异只有产生结果收益时才值得产品化。

原先声称轻量增强层“结构性做不到”的四点都需要降格：

| 原论点 | 复核结果 |
|---|---|
| 事务性捕获天然优于自觉记录 | 只有在捕获低噪声、可操作且降低续接成本时成立；journal 占位符并未阻止周末成功 |
| 拥有会话天然形成壁垒 | 是实现能力和入场券，编辑器、ACP client 与增强层 runtime 都可复制 |
| per-person 状态难以替代 | 数据作用域容易扩展，单独不构成护城河 |
| 聚合层天然抗宿主吸收 | 聚合器同样会被平台吸收；只有跨宿主中立性带来实测价值时成立 |

因此，Tsukumo 今天没有已证明的市场护城河。剩余候选是：

> **用户拥有的执行状态可自动跨独立 runtime 迁移；状态带有来源、作用范围、投影回执、行为证据、撤销效果与因果链；这些机制能够降低继续正确工作和从错误状态恢复的成本。**

## 3. Loomstead 的负面先验

复核 companion repository `ai-agent-town` 后，最重要的材料是：

- `docs/human_rating_pilot_gate.md`：Human Rating Pilot 被关闭，因为 Memory / Relationship ablation 虽进入决策路径，promoted scenarios 的 `goalToolEvents` 仍一致，差异主要停留在 evidence / integrity 指标。
- `docs/process_fidelity_eval_spec.md`：明确要求移除记忆后 winner 或分数差距发生变化；长期零变化只能证明 replay scaffold 可运行。
- `docs/archive/research_transition_audit_analysis_2026-06-05.md`：总结了 Hard Delegation metric stub、行为零分化、trace 完整不等价于可信过程，以及 toy fixture / deterministic audit 的 claim 边界。

这给 Tsukumo 一个必须遵守的证据阶梯：

1. **结构完整性**：状态有来源、回执、因果链和撤销记录。只证明实现存在。
2. **行为敏感性**：移除状态会改变工具参数或动作。只证明状态进入了决策路径。
3. **任务效用**：迁移状态让目标 runtime 更快开始正确工作、减少主人纠正，且不降低任务质量。
4. **故障恢复效用**：来源、范围与选择性撤销能更快定位坏状态、恢复正确动作，并减少误删。
5. **净期望价值**：真实异常发生率乘以单次节省成本，大于日常持续开销。

现有 GNU with-state / without-state 对照最多到第 2 层，不能单独回答“这些证据为什么值得追”。

## 4. 与 Loomstead 的根本差异

差异必须落在系统边界和因变量，不能依赖“小镇 vs 编码”或 Python vs Rust：

| 维度 | Loomstead | Tsukumo 本轮假设 |
|---|---|---|
| 系统边界 | 一个受控、由研究 harness 拥有的 agent runtime 内部 | Claude、Codex 等相互独立的外部 runtime 边界之间 |
| 核心问题 | 一个动作为什么被选中，证据链是否完整、对移除是否敏感 | 用户拥有的状态能否安全迁移，并降低真实续接、纠错与恢复成本 |
| 主要因变量 | provenance coverage、counterfactual sensitivity、process metrics | 首次正确动作时间、主人纠正次数、错误率、恢复时间、持续开销 |
| trace 角色 | 解释与评测 artifact | 会影响未来投影与撤销效果的执行控制面 |
| 失败判据 | evidence 完整但行为不变时收缩 claim | 行为变化但用户成本不降时同样收缩或停止产品化 |

面试时可直接回答：

> Loomstead 研究受控 runtime 内“为什么选了这个动作”，最后发现 trace 完整并不自动带来行为或用户价值。Tsukumo 把这个负面结果当作设计约束：状态必须跨 Claude / Codex 这类独立 runtime 边界，并在 Trellis 强基线之上减少续接和故障恢复成本。两者共享事件溯源与反事实方法；Tsukumo 更换了系统边界和因变量。若没有操作成本差异，就应将它诚实地描述成工程原型与一次否证，不宣称新的研究贡献。

## 5. 预注册条件

### C0 — Trellis-only 强基线

- 使用当前 `.trellis/spec/`、任务 PRD / design / implement、journal、普通 Git 与人工 runtime 选择。
- 不使用 Tsukumo 自动状态迁移、ProjectionReceipt 或选择性 revoke。
- 允许主人按日常方式告诉目标 agent 去读任务制品；这正是要打败的现实工作流。

### C1 — 自动迁移，隐藏追溯

- 使用 Tsukumo 自动选择并投影 checkpoint / StateRef。
- 用户只看到交接结果，不使用来源浏览、完整因果链或选择性撤销界面。
- `C1 - C0` 隔离“自动状态迁移”本身的效用。

### C2 — 自动迁移 + 完整追溯与选择性撤销

- 与 C1 使用同一迁移机制。
- 开放 source EventId、scope、receipt、行为 outcome、revoke 及后续投影效果。
- `C2 - C1` 隔离“可追溯 / 可撤销”带来的诊断与恢复效用。

条件顺序尽量交叉，固定可控的 repository、任务目标、runtime 版本与质量门。真实 dogfood 无法满足严格随机对照，因此结果定位为 n=1 产品决策证据，不作总体统计推断。

执行预算在 2026-07-13 经主人确认：真实工作可按预注册条件轮换，人工观察与填表总开销每天最多约 20 分钟。产品执行、既有测试和质量门耗时单独记账，不挤入观察预算。

## 6. 场景与样本单位

样本单位是 **handoff episode**，不是 token 总量。主人日均 1–2 亿 token 能快速积累 episode，但不能用 token 数替代独立交接。

到 2026-07-22 的目标：

- 最低记录 12 个可复核 handoff episode，20 个为 stretch target；
- 条件可用后尽量按 C0 / C1 / C2 各 4 个组成最低样本，并按任务类型分块或轮换，避免把某类简单任务集中给单一条件；
- 至少 4 个真正的中途 runtime 切换；
- 至少 2 个延迟 48–72 小时后的恢复；
- 至少 2 个 stale / scope / conflict 故障案例；
- 正常 dogfood 与受控故障分开报告。

受控故障至少覆盖：

1. 内容正确、作用范围错误；
2. 曾经正确、当前已经过期；
3. 两条状态互相冲突。

故障注入只能证明恢复机制存在，不能证明真实世界发生频率。是否值得常驻还要看自然 dogfood 中的异常率。

## 7. 指标

### 7.1 迁移效用（C1 vs C0）

- `time_to_first_correct_action`：目标 runtime 启动到第一个与当前任务一致且可保留的动作。
- `owner_interventions`：主人补充背景、纠正方向或要求重读材料的次数。
- `stale_state_error_rate`：使用过期 / 错域 / 冲突状态导致的 episode 比例。
- `context_reading_tokens`：为恢复上下文而读取的 token；与任务执行 token 分开。
- `task_quality`：相同项目质量门、review 结论或最终任务结果，防止用速度换质量。

操作定义在首个结果产生前冻结：

- `episode_start`：目标 runtime 接收到标准化交接目标且进程开始处理的时间；启动器准备时间另记为系统开销。
- `time_to_first_correct_action`：从 `episode_start` 到首个最终被保留、与声明目标一致的推进动作。单纯读取文件不计入；其结论被后续保留工作实际采用时可以计入。
- `owner_interventions`：启动后由主人补充缺失背景、纠正方向或要求重读材料的消息次数。标准化起始指令、预注册实验控制和普通权限决定单独记录。
- `task_quality`：沿用该任务已有验收条件、测试与 review 结论，并记录首个动作最终保留、修改或回滚；本轮不新建主观评分模型。
- `context_reading_tokens`：仅记录 CLI 或 runtime 能直接给出的可复核数据；无法拆分时标记 `unavailable`，禁止人工估算。

### 7.2 追溯与撤销效用（C2 vs C1）

- `time_to_identify_bad_state`：发现错误行为到定位具体 StateRef / 来源。
- `time_to_correct_action`：发现错误行为到目标 runtime 恢复正确动作。
- `mistaken_or_collateral_revokes`：误撤销或为解决一个问题而删除无关状态的数量。
- `recurrence_next_handoff`：相同坏状态是否在下一次交接复发。

### 7.3 持续开销

- 启动与投影延迟；
- 捕获、选择与注入 token；
- durable storage 增量；
- 用户阅读回执、解释状态与处理误报的认知时间；
- adapter 漂移和维护时间。

净价值采用：

```text
expected_value = natural_incidence × saved_recovery_cost - always_on_overhead
```

## 8. 7 月 23 日决策门

以下数字是单用户产品门槛，不是统计显著性声明：

- **迁移门**：C1 相对 C0 的首次正确动作时间中位数改善约 30%，或主人干预次数改善约 50%，且任务质量无下降。
- **恢复门**：在预注册故障中，C2 相对 C1 的定位 / 恢复时间改善约 50%，或明显减少 collateral deletion 与下一次交接复发。
- **工程门**：设计内 trace 场景 100% 保持 source → state → checkpoint → receipt → execution → outcome / revoke 链完整。
- **开销门**：正常 episode 的总延迟、token 与认知开销控制在约 5–10% 的小预算内；具体以主人主观可接受为最终约束。
- **发生率门**：记录自然 stale / scope / conflict 的发生频率；只有注入故障通过时，不宣称常驻追溯具有净产品价值。

决策：

| 结果 | 2026-07-23 决策 |
|---|---|
| 迁移门和恢复门均通过 | **GO**：扩大可信交接开发，TUI 作为可选控制 / 展示面 |
| 迁移门通过、恢复门不通过 | **PIVOT**：保留轻量自动 handoff，缩减完整 Chronicle / revoke 产品面 |
| 迁移门不通过、恢复门通过 | **PIVOT**：做按需故障审计 sidecar，不接管日常入口 |
| 两门均不通过或常驻开销过高 | **NO-GO（重型形态）**：停止扩大 runtime / TUI；保留原型与否证叙事 |

## 9. 追溯何时值得存在

完整追溯不应覆盖所有记忆。优先级启发式：

```text
追溯必要性 ∝ 生命周期 × 影响范围 × 自动化程度 × 推断不确定性
```

自动产生、长期存在、跨 runtime、生效范围大且由模型推断的状态，才进入完整 source / receipt / revoke 链。短期、低影响、用户显式且容易重新输入的信息可用轻量 checkpoint，避免把 Tsukumo 变成高成本日志系统。

## 10. 时间表与优先级

- **7/13–7/14**：冻结本协议与 Trellis C0 基线；种下一个 48–72 小时后恢复的延迟任务。
- **7/14–7/17**：打通真实 Claude → Codex / ACP 路径；完成版本化事件面侦察。
- **7/17–7/21**：高强度 dogfood，交叉运行 C0 / C1 / C2，执行预注册故障。
- **7/22**：证据、代码和 demo 冻结；形成正向与否证两套简历叙事。
- **7/23**：结合 AlgoCoach 结果开始投递，并作 GO / PIVOT / NO-GO 决策。

开发优先级：

- **P0**：可信交接、Trellis 强基线、续接与恢复指标；
- **P1**：真实 Claude / Codex 双 runtime 与可复核 demo；
- **P2**：权限审批闭环、陪伴 / 养成扩展与 TUI 入口强化。

## 11. 作品集叙事

正向结果：

> Loomstead 揭示了 trace 完整与用户价值之间的断层；Tsukumo 把研究方法带到真实 Claude / Codex 工作流，用事件溯源的用户状态实现跨 runtime 可信交接，并相对 Trellis 基线测得续接与故障恢复收益。

否证 / 收缩结果：

> 构建约数万行 Rust 原型后，用自己的 Trellis 工作流作为强基线做消融，发现 repo-native 增强层覆盖了大部分连续性价值；据此把重型产品收缩为 evidence sidecar，明确了 80/20 边界与不值得继续投入的部分。

两种结果都保留研究品味。最终说法由数据决定。
