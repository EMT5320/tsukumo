# C1 Handoff Continuity

## Goal

完成 Tsukumo 第一条真实、可追溯、可反事实验证的跨 runtime 连续性闭环：系统从真实用户表达与执行事件形成长期状态，将任务状态编译成可接手的 checkpoint，投影给另一个 runtime，并记录该状态是否进入后续工具行动。

## User Value

主人更换模型或 agent 工具后，不需要重新解释已经明确的项目约束、工作偏好和未完成事项；新的 runtime 能拿到一份足以继续工作的交接卷轴，同时主人能查看、纠正和撤销系统记住的内容。

## Background

- 北极星：`DESIGN.md`。
- 领域共识：`docs/tsukumo-vision-state-handoff-convergence-2026-07-10.md`。
- V0 收口决议：`docs/tsukumo-v0-scope-convergence-2026-07-11.md`。
- 已完成：版本化事件协议、Chronicle、Canonical State、StateWriter、迁移、导出与 theater replay。
- 尚未闭合：checkpoint/projection receipt、host 与增量 runtime 生命周期、第二 runtime、可交互 TUI、发布包装和状态到行动证据链。

## Product Invariants

1. Spirit 身份与 RuntimeBinding 分离，成长不按 Claude/Codex/builtin 分账。
2. 关系状态必须引用真实 Chronicle 事件；单次推断不能成为硬约束。
3. runtime 主要接收任务 checkpoint 与相关状态，不接收演出人格。
4. 普通状态可自动形成、低打扰、可撤销；执行权限必须显式批准。
5. 产品运行时只声明“状态被选中并投影、随后发生了什么”；因果主张需要 removed-state 对照。
6. Theater 是事件消费者和解释面，不控制执行节奏。

## Requirements

### R1 — Identity and Event Envelope

- 引入 `SpiritId`、`ExecutionId`、`SessionId`、`QuestId`、`EventId` 等稳定 newtype。
- `KernelEvent` 持有版本化 envelope：事件 ID、时间、quest/session/spirit/execution、runtime binding、causation/correlation 与 payload。
- adapter 只归一化 vendor payload；host/Chronicle writer 统一分配持久事件身份与顺序。
- live、persisted replay 和 fixture 使用同一 typed contract。

### R2 — Chronicle

- 保存用户消息、runtime/tool/permission、状态生命周期、checkpoint、projection、runtime 切换与 outcome。
- Chronicle 为 append-only；事件可按 quest/session/execution/correlation 重建证据链。
- 单一 SQLite 数据库是 Chronicle、Canonical State、Checkpoint 与 Receipt 的持久事实源；相关写入共享事务边界。
- JSONL、`MEMORY.md`、`USER.md` 与 FTS 是可重建导出/索引，不接受隐式反向导入。
- schema migration 必须幂等；现有 A1/R `facts` 数据若迁移，只能形成带 `legacy_imported` 证据的低强度状态，不能自动升级为 `Explicit` 或硬 `Constraint`。
- 写入失败必须向调用者暴露，不能静默丢失证据。

### R3 — StateRecord and StateWriter

- 支持 stable key、kind、Subject + Applicability scope、evidence strength、status、evidence refs、时间与 TTL。
- 支持 create、supersede、revoke；冲突不得静默覆盖。
- 确定性 StateWriter 检查 evidence、scope、敏感信息、安全边界与硬约束升级规则。
- State extraction 与 deterministic write gate 分离：结构化/显式信号走规则，自由自然语言走结构化 LLM；两者都只能产生 `StateDraft`。
- 默认回归测试使用确定性或录制草稿，另提供一条可选 live LLM smoke；C1 至少覆盖一条显式用户约束。

### R4 — HandoffCheckpoint

- checkpoint 包含 goal、progress、decisions、constraints、artifacts、open loops、next actions、durable state refs 与 source event refs。
- checkpoint 版本化、低频生成；硬约束通过稳定 StateRef 继承。
- 每个 open loop 在下一版本中必须继承、完成、放弃或显式替代。

### R5 — ProjectionReceipt

- 每次投影记录 checkpoint、selected state refs、target runtime、execution、renderer/projection version、整体及分段 SHA-256、字符/字节长度、诚实预算单位、遗漏原因与脱敏清单。
- tool/outcome 事件可关联回 projection。
- 生产 Receipt、Chronicle、日志与错误默认不保存完整或原始 prompt；raw prompt 只作为 host/runtime 边界的内存秘密值存在。
- V0 生产模式仅持久化 Receipt 元数据与 digest；确定性 removed-state 对照使用临时或经审查的脱敏 fixture，不建立通用快照留存生命周期。
- runtime 必须在 Receipt 成功落盘后启动；Receipt 写入失败不得产生无证据执行。

### R6 — Host and Incremental Runtime

- 新增 `tsukumo-host` 或等价 composition root，实际连接 soul/state、checkpoint/projector、adapter、runtime process、Chronicle、Director 与 StageWorld。
- runtime 输出按行/事件增量处理，不等待整个进程完成后再返回完整 `Vec<KernelEventPayload>`。
- 进程取消/退出必须回收；permission request 进入独立 Safety Plane。

### R7 — Cross-Runtime and Counterfactual Case

- Runtime A 固定为 Claude CLI own-process `stream-json`；Runtime B 固定为 Codex CLI `codex exec --json`，对应主人的两种主要工具。
- 正向场景：Claude 中明确“Tsukumo 在 Windows 上统一使用 GNU toolchain”；Codex 只收到“运行完整测试”后选择 GNU 命令。
- 对照场景：相同任务/runtime/config，唯一变量为是否投影该状态。
- 默认 CI 使用脱敏录制的 Claude/Codex JSONL；真实双 runtime handoff 作为本地凭证驱动的 opt-in smoke。
- live smoke 由 `TSUKUMO_RUN_LIVE_SMOKE=1` 显式开启，记录两端 CLI 版本；开启后缺 CLI/认证属于可操作失败，不能静默 skip。
- 形成确定性的 comparison bundle，记录 source events、state、checkpoint、receipt、tool calls/outcome 与比较摘要；V0 不把它升级为通用持久化产品面。

### R8 — Minimal Product Surface

- 非阻塞显示“已记住”及证据来源，并允许查看/撤销。
- permission request 使用硬确认，至少支持“本次允许 / 本会话允许 / 拒绝”。
- 显示 checkpoint/handoff 状态与“本次参考了什么”。
- 复用现有 theater 表现状态形成、等待审批和完成，不扩角色/美术系统。

### R9 — Reproducible Verification

- 建立可在干净环境执行的 Rust toolchain/dependency 路径与 CI，至少覆盖通用 Linux 检查和 Windows GNU 目标。
- 解决当前 rustfmt 漂移；二进制 workspace 跟踪 `Cargo.lock`，并在验证依赖 MSRV 后固定实际可用的 Rust toolchain/target。
- 格式、check、clippy、workspace tests 的结果必须可复现；环境阻塞与代码失败分开报告。
- 提供可安装 `tsukumo` 二进制、README、MIT LICENSE、完整 Cargo metadata、真实限制与数据/隐私说明。

## Acceptance Criteria

- [ ] AC1：真实用户事件可追溯地形成 `Constraint + Explicit` StateRecord，且可查询其证据、scope 和版本。
- [ ] AC2：runtime 切换生成版本化 checkpoint，关键约束以 StateRef 保留，open loops 不静默丢失。
- [ ] AC3：ProjectionReceipt 精确记录本次选中状态、目标 runtime、execution、版本、整体/分段 hash、长度、遗漏/脱敏信息与明确预算单位，且 schema 中不存在原始渲染文本字段。
- [ ] AC4：第二 runtime 的 tool call/outcome 能沿 execution/projection/state/event refs 回溯到原始用户表达。
- [ ] AC5：removed-state CaseBundle 在受控条件下产生可观察的 tool argument 差异，且产品声明不越过证据边界。
- [ ] AC6：revoke/supersede 后旧状态停止进入新 projection，历史 receipt 仍可解释。
- [ ] AC7：一次或多次 permission approval 不产生 auto-approve StateRecord，后续危险请求仍走硬确认。
- [ ] AC8：live 与 replay 共用事件 contract；关键链路有 unit、persistence/reopen、fixture replay 和 cross-crate integration 测试，并证明 Receipt 先于 runtime 启动持久化。
- [ ] AC9：最小 UI 能展示状态形成、撤销、handoff、permission 和 selected-state refs，不扩展角色/美术范围。
- [ ] AC10：可安装二进制、README、LICENSE、tracked lockfile、工具链声明与 Linux/Windows GNU CI 在同一 release candidate 上通过。

## Task Map

父任务持有完整需求、跨子任务验收和最终集成审查，默认不直接承载产品实现。

1. `c1-contracts-chronicle`：R1–R3；身份、事件 envelope、Chronicle、StateRecord/StateWriter。
2. `c1-handoff-projection`：R4–R5；checkpoint、scope selection、projection receipt 与 deterministic comparison seam。
3. `c1-host-runtime`：R6 + Safety Plane；增量 runtime、进程生命周期、真实投影接线。
4. `c1-cross-runtime-evidence`：R7；Codex runtime、removed-state 对照与跨 runtime 证据链。
5. `v0-mvp-tui`：R8；状态、projection、handoff 与 permission 的最小交互面。
6. `v0-release-packaging`：R9；可安装入口、README、许可、锁文件、工具链、CI 与发布验收。

子任务按 1 → 2 → 3 → 4 → 5 → 6 顺序推进；父任务在每个边界检查 contract 与证据引用一致性。

## Out of Scope

- 完整 coding agent、完整 ACP editor/fs/terminal client。
- GEPA、自动 prompt/skill 自进化、完整 MCP recall 产品面。
- 多世界观、大量角色、复杂羁绊、主搭档完整人格。
- 图数据库和完整知识图谱 ontology。
- 每次真实任务都运行昂贵反事实。
- 为求职展示提前搬入完整 Loomstead evaluator。
- V0.1 才实现通用 debug/eval prompt snapshot、七天 expiry、显式 retain、cleanup audit 与 artifact-management UI。
