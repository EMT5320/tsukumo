# E02 — C1 延迟恢复：工具链与发布声明证据审计

> 状态：wait-for-C1（已预注册，尚未 seeded）
> 条件：C1 自动迁移、隐藏追溯
> 类型：natural / delayed-resumption
> Source runtime：Claude Code（seed 时重新探测版本）
> Target runtime：Codex CLI（resume 时重新探测版本）
> Seeded at：待真实 source action 与 checkpoint 成功提交后填写
> Resume window：实际 Seeded at +48h 至 +72h

## 预注册目标

在恢复窗口内，由 Codex 使用自动投影的 checkpoint 继续审计 2026-07-14
工具链验证事件及其后续处置：

1. 区分代码失败、依赖/缓存失败和本机编译器环境失败；
2. 列出当前证据实际支持的 Linux、Windows MSVC、Windows GNU 声明；
3. 若 GNU 已修复，引用可复核的成功门禁；若仍未修复，给出最小后续验证；
4. 形成一段可保留的 release-readiness 审计结论；
5. 不启动完整 release packaging，不编写 CI 或发布代码。

## Source action

Claude 必须先实际执行一次有用的工具链诊断，形成至少一个可保留事实，并留下
“哪些兼容性声明可进入后续 release packaging”这一真实 open loop。

完成后由主人审阅 source summary，再运行 `episode seed`。当前文件和任何计划性
summary 都不能替代真实 source action，也不能据此标记 seeded。

## C1 条件纪律

- 使用自动 checkpoint / StateRef 投影。
- target 首个正确动作形成前，主人和 target 不打开 source、scope、receipt 或 revoke。
- 起始指令不附带 source 诊断摘要。
- 自动证据密封保存在 SQLite；首个正确动作形成后再审阅。
- 本轮 fault 为 none，不使用 selective revoke。
- 标准化起始指令：继续 E02：请使用本次自动交接继续工具链与发布声明证据审计，并完成预注册目标。

## 质量门

- 引用实际命令结果或可复核 artifact；
- 没有成功重跑时不得声称 GNU 已通过；
- 明确区分本机环境问题与仓库代码问题；
- 不提前实现 release packaging；
- 不包含凭据、原始 prompt 或个人路径；
- 单纯读取文件不计为首次正确动作。

## 当前工程门

- [x] C0 不产生 Tsukumo receipt 或 runtime spawn。
- [x] C1/C2 使用同一迁移数据面和相同投影字节。
- [x] resume 在 runtime spawn 前提交并重读 ProjectionReceipt。
- [x] 延迟窗口提前调用在 projection/spawn 前失败。
- [ ] Claude source action 已真实完成并由主人审阅。
- [ ] reviewed spec 已通过 `episode seed` 写入正式 data-dir。
- [ ] Seeded at 与恢复窗口已从机器 summary 回填。

## 观察记录

恢复时复制 ../handoff-episode-template.md 的字段到本节并填写。首次正确动作以首段
最终保留、满足质量门的发布声明审计结论为准。
