# C1 Cross-Runtime and UI

## Parent and Dependencies

- Parent: `.trellis/tasks/07-10-c1-handoff-continuity`
- Depends on: `07-10-c1-contracts-chronicle`,
  `07-10-c1-handoff-projection`, `07-10-c1-host-runtime`.

## Goal

用第二 runtime 与最小产品界面完成 C1 纵向验收，证明同一关系状态跨 backend 延续并在受控 removed-state 对照中改变后续工具参数。

## User Value

主人可以在最常用的 Claude 与 Codex 之间切换而不重新解释项目约束，并直接查看“记住了什么、为何记住、本次用了什么”。

## Confirmed Evidence

- Claude CLI 与 Codex CLI 是主人确认的 C1 双运行时；Codex 官方 `exec --json` 输出 JSONL，当前本地可见版本为 `0.144.0-alpha.4`。
- 现有 theater 已有纯 Director、StageWorld 和 ratatui 渲染，可扩展最小解释/确认界面，无需新建另一套前端。
- 当前仓库没有 CI、toolchain pin 或 tracked `Cargo.lock`，且本地检查受目标/网络环境影响。

## Requirements

- 接入 Codex CLI `codex exec --json` 作为第二 runtime，不创建第二套 spirit/state 账本。
- 默认 CI 使用脱敏 Claude/Codex JSONL fixtures；真实双 runtime 路径使用显式 opt-in smoke，不把凭证放入 CI。
- 跑通 GNU toolchain 正向 handoff 场景及无状态对照。
- 生成完整 CaseBundle 与比较摘要，区分运行时证据和因果评测。
- 提供状态形成通知、查看/撤销、handoff 状态、selected-state refs 和 permission 确认的最小 UI。
- theater 仅表现已有状态/权限/完成事件，不扩角色和美术系统。
- 建立 Linux + Windows GNU 的干净环境 CI；跟踪 `Cargo.lock`，在实测依赖 MSRV 后固定 Rust toolchain/target，完成最终质量门。

## Acceptance Criteria

- [ ] 同一 Spirit 在两个 RuntimeBinding 间切换，状态与 checkpoint 连续。
- [ ] with-state 与 without-state 仅在目标状态投影上不同，并产生可观察 tool argument 差异。
- [ ] receipt/tool/outcome 能回溯到 runtime A 的原始用户事件。
- [ ] revoke 后再次执行不再投影旧状态。
- [ ] 危险请求始终需要显式用户决策。
- [ ] 最小 UI 可以解释“记住了什么、为什么、本次用了什么”。
- [ ] fmt/check/clippy/tests/CI 在记录工具链上通过。
- [ ] opt-in live smoke 记录 Claude/Codex 版本并在缺少显式前置条件时给出可操作错误。
- [ ] 默认 CI 不需要 CLI 凭证，Linux 与 Windows GNU 均执行记录的格式/check/clippy/test 门。

## Out of Scope

- 完整角色/世界观、复杂羁绊、GEPA、完整 Loomstead evaluator。
