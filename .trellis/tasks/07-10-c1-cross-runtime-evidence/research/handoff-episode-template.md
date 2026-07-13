# Handoff Episode 观察模板

> 每个 episode 复制一份。只记录经过审阅的指标与引用；禁止粘贴原始 prompt、凭据、个人路径或未脱敏 runtime 输出。

## 1. 预注册

- Episode ID：
- Condition：`C0 | C1 | C2`
- 类型：`natural | controlled-fault`
- Workload block：`delayed-resumption | mid-task-code | analysis-docs | recovery`
- Source runtime / version：
- Target runtime / version：
- Repository / task artifact：
- 声明目标：
- 标准化起始指令摘要：
- 是否延迟恢复：`no | 48h | 72h`
- 预定窗口：
- 预注册 fault：`none | wrong-scope | stale | contradiction`
- 现有质量门：

### 条件纪律

- C0：允许正常读取 Trellis、Git 与仓库制品；禁用 Tsukumo 自动迁移、receipt 与 selective revoke。
- C1：开启自动 checkpoint / StateRef 投影；用户侧隐藏 provenance、完整因果链与 selective revoke。
- C2：使用与 C1 相同迁移，并开放 source、scope、receipt、outcome 与 selective revoke。
- 启动后不得因观察到的结果更改 condition、阈值或 fault；偏差写入“协议偏差”。

## 2. 时间点

- `episode_start`：
- 首个候选推进动作：
- 首个最终保留的正确动作：
- 发现坏状态：
- 定位具体坏状态：
- 恢复正确动作：
- Episode 结束：

## 3. 迁移效用

- `time_to_first_correct_action`：
- `owner_interventions`：
- `stale_state_error`：`yes | no`
- `context_reading_tokens`：数值或 `unavailable`
- 首个动作结果：`retained | modified | rolled-back`
- `task_quality`：现有验收、测试或 review 结论

## 4. 恢复效用（有 fault 时填写）

- `time_to_identify_bad_state`：
- `time_to_correct_action`：
- `mistaken_or_collateral_revokes`：
- `recurrence_next_handoff`：`yes | no | pending`
- 使用的 source / scope / receipt / revoke 引用：

## 5. 持续开销

- 启动与投影延迟：
- 捕获 / 选择 / 注入 token：数值或 `unavailable`
- Durable storage 增量：
- 主人阅读与处理时间：
- Adapter / launcher 异常：
- 当日人工观察累计时间：

## 6. 结果与边界

- 可支持的最强结论：`integrity | behavioral-sensitivity | migration-utility | recovery-utility | expected-value`
- 不能支持的结论：
- 自然事件与注入事件是否分开：`yes | no`
- 协议偏差：
- 脱敏与隐私检查：
- Reviewer：