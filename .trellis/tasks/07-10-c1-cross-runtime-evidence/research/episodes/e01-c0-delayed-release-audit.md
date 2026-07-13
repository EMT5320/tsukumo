# E01 — C0 延迟恢复：Release Packaging 门控审计

> 状态：seeded
> 条件：C0 Trellis-only
> 类型：natural / delayed-resumption
> Source runtime：Codex CLI 0.135.0
> Target runtime：Claude Code 2.1.205
> Seeded at：2026-07-13T21:54:36+08:00
> Resume window：2026-07-15T21:54:36+08:00 至 2026-07-16T21:54:36+08:00

## 预注册目标

在恢复窗口内，从 Claude Code 开启一次新会话，仅依赖当前 Git、Trellis 和仓库文档恢复上下文，审计 `07-11-v0-release-packaging`：

1. 列出哪些 checklist 项依赖 2026-07-23 trusted-handoff 决策；
2. 区分 GO、自动 handoff PIVOT、evidence-sidecar PIVOT、重型形态 NO-GO 下仍值得保留的最小包装工作；
3. 产出一段可保留的审计结论，不实施 packaging 代码或 CI。

## C0 条件纪律

- 允许目标 runtime 按日常方式读取 .trellis/spec/、任务 PRD/design/implement、DESIGN.md、journal 与 Git 历史。
- 禁用 Tsukumo 自动 checkpoint / StateRef 投影、ProjectionReceipt、来源浏览和 selective revoke。
- 标准化起始指令：继续 E01：请按当前 Trellis 制品恢复任务并完成预注册目标。
- 起始指令不附带本轮讨论摘要；额外背景补充计入 owner_interventions。

## 质量门

- 结论必须引用当前仓库制品并区分四种决策结果；
- 不提前启动或实现 release packaging；
- 不把 demo capture 与完整 v0.1.0 包装混为一项；
- 输出不得包含凭据、原始 prompt 或个人路径。

## 观察记录

恢复时复制 ../handoff-episode-template.md 的字段到本节并填写。首次正确动作以首段最终保留、满足质量门的审计结论为准。