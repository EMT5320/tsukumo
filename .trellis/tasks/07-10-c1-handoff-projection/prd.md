# C1 Handoff and Projection

## Parent and Dependency

- Parent: `.trellis/tasks/07-10-c1-handoff-continuity`
- Depends on: `07-10-c1-contracts-chronicle`

## Goal

把 Chronicle 与 Canonical State 编译成可直接接手的版本化 HandoffCheckpoint，并为每次 runtime 投影生成可解释的 ProjectionReceipt。

## User Value

主人切换工具时，新 runtime 能接到真正可继续工作的任务卷轴；之后仍能解释本次参考了哪些长期状态，同时不会把完整 prompt 长期沉积为隐私负担。

## Confirmed Evidence

- 当前 `BriefCompiler` 只有字符容量与事实文本，没有 checkpoint/open-loop 语义或稳定 StateRef。
- 当前 inject trace 只记录 brief/goal 字符数，无法关联 checkpoint、state、runtime 或 execution。
- C1 已确认生产 receipt 只留结构化元数据与 digest，调试快照走显式、脱敏、限期的 CaseBundle。

## Requirements

- 实现 checkpoint goal/progress/decisions/constraints/artifacts/open loops/next actions/state refs/source refs。
- 实现 checkpoint 版本与 open-loop 继承/关闭规则。
- 实现基于任务、scope、strength、新鲜度和预算的确定性 state selection seam。
- 实现 projection renderer、版本、整体/分段 SHA-256、selected refs、runtime/execution、字符/字节长度、明确预算单位、遗漏原因与脱敏清单。
- checkpoint、receipt 及其 state refs 与 Chronicle/Canonical State 共用 SQLite 事实源和事务完整性约束。
- 生产 receipt、Chronicle、日志与错误不保存完整或原始 prompt；raw prompt 仅以可脱敏调试表示的内存秘密值传递。
- 仅显式 debug/eval CaseBundle 保存脱敏规范化快照；快照使用独立 hash，默认七天过期，长期保留必须显式选择。
- receipt 在 runtime 启动前提交；持久化失败不得继续执行。
- 建立 deterministic CaseBundle 基础，可在无真实 runtime 时验证 with-state/without-state 输入差异。

## Acceptance Criteria

- [ ] 新 checkpoint 能完整继承或显式关闭上一版本 open loops。
- [ ] 硬约束使用 StateRef，展示文案变化不改变引用。
- [ ] 不相关或 revoked 状态不进入 projection。
- [ ] receipt 可精确重建选中状态、checkpoint、runtime、execution、renderer/projection 版本、hash、长度、遗漏/脱敏信息和预算单位。
- [ ] 同输入与 renderer 版本产生稳定 hash。
- [ ] receipt 序列化、数据库行、Chronicle 和错误中不出现 sentinel 原始 prompt/秘密。
- [ ] opt-in 快照先脱敏后落盘，具有独立 hash、默认七天 expiry 与显式 retain 路径。
- [ ] synthetic CaseBundle 可移除单条状态并保持其他变量不变。

## Out of Scope

- 真实子进程、permission UI、第二 runtime、完整 MCP recall、原始 prompt 归档和加密 transcript 仓库。
