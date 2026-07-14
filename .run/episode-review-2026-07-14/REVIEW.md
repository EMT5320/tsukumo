# Episode 种籽包审阅 — 2026-07-14

**状态：待老师审阅（PENDING_TEACHER_REVIEW）**
**本目录禁止在审阅通过前执行 `episode seed` / `episode resume`。**
审阅状态只写在本文件；`c1.review.json` / `c2.review.json` 仅含 `EpisodeSpecV1` 认识字段。

## 材料清单

| 文件 | 用途 |
|---|---|
| `REVIEW.md` | 证据、claim 边界、待确认项、seed 前检查表 |
| `c1.review.json` | C1 审阅候选（`condition=c1`） |
| `c2.review.json` | C2 审阅候选（`condition=c2`） |

两份 JSON 除 `condition` 外语义一致；延迟窗口 `48–72h`；source→target 为 Codex CLI → Claude CLI。

## 证据来源

1. **Schema / 枚举 / profile 约束**（只读核对）
   - `crates/tsukumo-host/src/episode.rs`：`EpisodeSpecV1`、`EpisodeCondition`、`EpisodeRuntimeKind`、`EpisodeExecutionProfile`、`validate_spec`
   - `crates/tsukumo-host/tests/episode_runner_contract.rs`：`episode_spec` 夹具与 Codex→Claude 目标样例
   - 根 `README.md`：reviewed JSON 形状说明（示例非本 episode 经历）
2. **Runtime 版本探针（本机，2026-07-14）**
   - `codex --version` → `codex-cli 0.135.0`（JSON 写入 `0.135.0`）
   - `claude --version` → `2.1.205 (Claude Code)`（JSON 写入 `2.1.205`）
   - 两者均成功；**无 NEEDS_REVIEW 版本字段**；Claude target **不构成阻塞**。
3. **真实 source action（主人给定，本轮未重新执行测试/seed）**
   - 日期：2026-07-14
   - source：当前 Codex CLI 协调会话
   - **source profile**：JSON 冻结为 `codex_workspace_write`。当前 Codex 协调会话具备 workspace 写能力，但实际编码委派给 Cursor CLI；本 profile 描述协调会话能力边界，不表示本会话亲自写入了全部产物。
   - 已完成：委派 Cursor CLI 根 README 产品化；审查 `8616e5f` / `f5f88ad` 白天 episode hardening；补齐 C0 resume、live confirmation、workspace-write 两类拒绝、Claude target 共 5 类契约测试；**targeted episode_runner_contract suite reported 17 passed**（不暗示全量测试）
   - open loop / checkpoint.goal：评估 48–72h Codex→Claude delayed resumption 的 C1/C2，并与外部 Trellis-only C0 强基线比较；恢复后记录首次正确动作时延、干预次数与结果质量
   - 工作树仍有**未提交** `README.md` 与 `episode_runner_contract.rs`；材料中明确其为未发布/未合并

## Claim 边界

**可以写进种籽材料的 claim**

- 本机探测到的 Codex / Claude 版本字符串
- 协调会话具备 `codex_workspace_write` 能力，且实际编码委派给 Cursor
- 协调会话已完成的工作条目与 open loop（按主人陈述记录）
- targeted `episode_runner_contract` 报告 17 passed（非全量 suite）
- C1/C2 仅 condition 可见性不同；迁移数据面应保持一致
- 未提交路径存在，且不得被描述为已发布或已合并

**本材料不声称**

- 已 seed / 已进入 delay window / 已 resume
- 未提交改动已合并或已发布
- 全量测试通过；仅称 targeted episode_runner_contract suite 17 passed（沿用主人给定，本轮未重跑）
- C0 Trellis-only 基线数值已齐全或已完成比较
- live smoke / 跨 runtime 效用已证明

## 需要老师确认（最多 5 项）

1. **Source 叙事是否可冻结**：上述 2026-07-14 Codex 协调会话工作与 open loop 是否准确、可作正式 source summary？
2. **Source execution_profile**：确认冻结为 `codex_workspace_write`（协调会话具备 workspace 写能力；实际编码委派 Cursor）是否可接受。
3. **Target 冻结**：确认 target 为 Claude CLI `2.1.205` / `claude_deny_unapproved`（本机版本已探测成功）。
4. **未提交边界**：确认 seed 前仍须把 README / `episode_runner_contract.rs` 视为 working-tree 证据，不得写入“已发布/已合并”表述。
5. **C0 比较入口**：外部 Trellis-only C0 强基线的对照入口与度量口径是否同意按 checkpoint.goal / open loop 在 resume 后记录（首次正确动作时延、干预次数、结果质量）？

## Seed 前检查表（审阅通过后执行；本轮禁止）

- [ ] 老师对上述 ≤5 项全部签字/口头确认
- [ ] `c1.review.json` / `c2.review.json` 经 `read_episode_spec` 校验通过，且除 `condition` 外一致
- [ ] 使用**两个独立** `--data-dir`；不得混用同一 store
- [ ] 确认不会把未提交文件描述成已发布
- [ ] 仅在确认后运行 `episode seed`；本目录当前 **不得 seed**
- [ ] seed 成功后再等待 48–72h 窗口；resume 需 `--confirm-live-run` 与审阅一致的 Claude 可执行文件（target 为 Claude，不因 source 的 `codex_workspace_write` 追加 `--workspace-write`）

## 本轮只读验证（未 seed、无 data-dir）

- Python：两份 JSON 可解析；去掉 `condition` 后结构相等；`delay=48..72`；source profile=`codex_workspace_write`
- `tsukumo_host::read_episode_spec`（临时离线 crate，不改仓库）：C1/C2 均 `OK`
- `.run/episode-review-2026-07-14/` 下仅有本审阅三文件，未产生 episode data-dir

## 本轮结论

| 项 | 结果 |
|---|---|
| 材料修订 | 三项修正已写入 |
| 版本探针 | Codex / Claude 均成功（沿用既有探测） |
| Claude target 阻塞 | 无 |
| Parser 校验 | 通过（只读重跑） |
| **能否进入 seed** | **否** — 待老师审阅通过后方可 |
