# C1 Host and Runtime

## Parent and Dependencies

- Parent: `.trellis/tasks/07-10-c1-handoff-continuity`
- Depends on: `07-10-c1-contracts-chronicle`, `07-10-c1-handoff-projection`

## Goal

实现 Tsukumo composition root，把 checkpoint/state projection 真正交给一个外部 runtime，并将增量事件、权限请求、工具行动和结果写回 Chronicle 与 theater。

## User Value

主人不再需要手动复制交接文本和盯住多个终端；Tsukumo 能拥有一次真实委托的生命周期，并在阻塞、失败或等待批准时及时呈现。

## Confirmed Evidence

- workspace 目前没有 composition-root crate，adapter tests 只证明离线 JSONL 到 theater 的接线。
- 当前 Claude-like decoder 接受 `Read`，会先收集完整事件向量；live host 必须逐行消费。
- Claude 官方 CLI 的非交互权限回调需要 `--permission-prompt-tool` MCP seam；C1 禁止用跳过权限参数冒充 Safety Plane。

## Requirements

- 新增 `tsukumo-host` 或等价 crate/bin。
- 组装 State → Checkpoint → ProjectionReceipt → runtime prompt。
- 管理首个 live Claude CLI `stream-json` 子进程及取消、退出、超时和回收。
- 将 NDJSON/协议事件逐条归一化并立即流向 Chronicle/Director，而非收集完整 `Vec`。
- permission request 保留结构化工具、参数、cwd、风险与 runtime 来源。
- Safety Plane 支持本次/本会话/拒绝；模型不能批准权限。
- live C1 使用最小能力配置；尚未由 Tsukumo 接管的 vendor 权限必须标记 degraded/unsupported，不能声称已完成 live permission fidelity。
- adapter drift、进程失败、用户拒绝与正常结束是不同结果。

## Acceptance Criteria

- [ ] 首个 live runtime 收到真实 checkpoint projection，receipt 在进程启动前落盘。
- [ ] 第一条 runtime 事件可在进程结束前到达 Chronicle 与 StageWorld。
- [ ] tool start/end/outcome 可关联 execution 与 projection。
- [ ] 取消/失败路径回收子进程且写入明确终止事件。
- [ ] permission approval/denial 被记录，但不会形成 auto-approve StateRecord。
- [ ] live transport 不使用 `--dangerously-skip-permissions`；unsupported vendor permission bridge 产生明确降级结果。
- [ ] synthetic 与 live transport 共用归一化 payload 和 envelope writer。
- [ ] Claude adapter 的默认 CI fixture 与 opt-in live smoke 使用同一 decoder。

## Out of Scope

- 完整 ACP editor capabilities、第二 runtime、复杂大厅 UI。
