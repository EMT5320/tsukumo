# Episode Runner V1 操作边界

> 状态：已实现，尚未启动 E02/E03。
> 目的：为 C1/C2 提供最小真实入口，不引入第二套评测存储。

## 两阶段命令

```text
tsukumo-host episode seed --spec <reviewed.json> --data-dir <directory>

tsukumo-host episode resume --spec <reviewed.json> --data-dir <directory> \
  --runtime-executable <path> --working-dir <directory> --confirm-live-run \
  [--workspace-write]
```

`seed` 只记录已经真实发生、由主人审阅过的 source action 摘要，并通过正式
Chronicle / StateWriter / checkpoint 路径落盘。它不会抓取当前 Codex 或 Claude
会话，也不能把计划中的动作写成已经发生的事实。

`resume` 根据同一 reviewed spec 找到 checkpoint，检查实际 seed 时间派生的
恢复窗口，先提交 ProjectionReceipt，再通过现有 RuntimeOrchestrator 启动目标
Claude/Codex。执行该命令会触发真实外部 CLI 和可能的模型费用，因此只能由主人
明确运行；测试不会调用外部模型。

## Spec V1 必填信息

- `schema_version = 1`、唯一 episode ID、C0/C1/C2、类型、workload block、fault；
- quest、source/target session、SpiritId；
- source/target runtime family 与实际版本；
- 经审阅的 source summary；可选的显式状态输入目前只接受既有 GNU 规则；
- checkpoint goal/progress/decisions/artifacts/open loops/next actions；
- projection scope、字符预算、标准化 delegation goal；
- seed 后的最小/最大延迟小时数；
- 预注册质量门。

输入最大 1 MiB，未知字段、控制字符、敏感材料和同 runtime 方向均会失败。
spec 是经审阅的实验输入，不得保存原始 prompt、凭据或聊天导出。

## 条件纪律

- C0：seed 不创建 data-dir；resume 不准备 receipt、不启动 runtime。
- C1：使用和 C2 相同的 checkpoint / StateRef / prompt；机器摘要隐藏 receipt、
  execution、digest 和 state metadata。
- C2：迁移数据面与 C1 相同；执行后摘要暴露 receipt/provenance metadata，详细
  source/scope/revoke 继续使用同一 data-dir 的现有 TUI。
- condition 在稳定 ID 指纹中被规范化，不得仅因 C1/C2 可见性不同而改变 prompt。
- 48–72 小时窗口从 committed checkpoint timestamp 计算，不允许倒填。

## 自动与人工指标边界

自动输出仅包括：

- episode 开始/结束毫秒时间；
- projection/runtime 毫秒耗时；
- SQLite 文件体积增量（无法读取时为 unavailable）；
- OutcomeStatus、execution failure 类别和事件/忽略行计数；
- C2 的 checkpoint/projection/execution ID、digest、selected/omitted 数量。

首次正确动作、owner intervention、task quality、坏状态诊断/恢复、最终 retained /
modified / rolled-back 仍由主人在 episode Markdown 中填写。stdout/stderr 内容、
工作目录、可执行文件路径和渲染 prompt 不进入 summary。

## 启动前检查

1. episode Markdown 已先冻结 workload、condition、fault、质量门；
2. source runtime 已真实完成至少一个最终保留的动作；
3. source summary 由主人审阅，且没有原始 prompt/凭据/个人路径；
4. 如需 StateRecord，使用真实 owner statement 走 extractor/writer；
5. seed 成功后才把状态改为 seeded，并按输出时间计算 +48h/+72h；
6. 恢复时主人显式确认 runtime 费用与 workspace-write；
7. C1 首次正确动作前不打开 TUI 证据面；C2 查看证据的次数与耗时计入开销。


## 2026-07-14 Independent-review hardening

This section supersedes earlier V1 details where they differ.

- Runtime identity freezes kind, reviewed version, and execution profile
  (Claude deny-unapproved, Codex read-only, or Codex workspace-write).
- The migration digest is condition/profile-neutral and only drives shared
  State/checkpoint/projection/execution IDs. A full registration digest is
  stored as non-rendered immutable checkpoint metadata and freezes the real
  condition and execution profile.
- Resume requires --confirm-live-run. --workspace-write only acknowledges a
  reviewed Codex workspace-write profile; it never selects or changes one.
- Before receipt creation, the guarded working directory and target executable
  pass a prompt-free --version probe. Adapter parsing proves the exact vendor
  family/version. Probe failure leaves zero ProjectionReceipt and zero target
  runtime execution spawn.
- The delay window is checked once before projection and again using the exact
  durable RuntimeLifecycle::Starting timestamp. That timestamp is the machine
  summary episode start; crossing the window prevents target spawn.
- Private recursively strict DTOs reject nested unknown fields. Prompt-facing
  text rejects terminal control/format characters, secrets, and personal home
  paths. Artifact locations are canonical repository-relative paths.
- Source runtime version remains an owner-reviewed fact from the real source
  action; seed V1 does not claim to re-probe an already-ended source session.
  Target version is observed by resume preflight.

Targeted gates: episode_runner_contract 12/12 and
runtime_identity_contract 2/2. External model calls remain zero.
