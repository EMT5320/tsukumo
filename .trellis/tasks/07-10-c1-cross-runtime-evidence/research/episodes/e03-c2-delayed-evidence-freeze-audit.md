# E03 — C2 延迟恢复：证据冻结与 Claim Boundary 审计

> 状态：wait-for-C2（已预注册，尚未 seeded）
> 条件：C2 自动迁移 + 来源/范围/回执/撤销控制
> 类型：natural / delayed-resumption
> Source runtime：Codex CLI（seed 时重新探测版本）
> Target runtime：Claude Code（resume 时重新探测版本）
> Seeded at：待真实 source action 与 checkpoint 成功提交后填写
> Resume window：实际 Seeded at +48h 至 +72h

## 预注册目标

在恢复窗口内，由 Claude 使用自动投影的 checkpoint，并可按需查看 source、scope
和 receipt，完成 2026-07-22 evidence freeze 审计：

1. 将现有材料分别归类为 integrity、behavioral sensitivity、migration utility、
   recovery utility；
2. 列出 E01–E03 完成后各自能支持和不能支持的结论；
3. 指出最低 12 个 episode 与故障恢复证据仍缺少的覆盖；
4. 给出最小 demo/evidence-freeze artifact 集；
5. 不把 fixture replay、connectivity smoke 或 injected fault 升级为自然效用证据。

## Source action

Codex 必须实际审查当前 evidence chain，形成至少一个可保留的 claim-boundary 判断，
并留下“7 月 22 日冻结时哪些证据有资格进入作品集叙事”这一真实 open loop。

完成后由主人审阅 source summary，再运行 `episode seed`。当前文件、fixture replay
或 synthetic live smoke 均不能替代真实 source action。

## C2 条件纪律

- 使用与 C1 相同的自动迁移数据面。
- target 可按需查看 source、scope、receipt 和 outcome。
- 每次查看追溯信息及其耗时都记录为认知开销。
- 本轮 fault 为 none；不得为了展示功能而无理由 revoke。
- 起始指令不手工复制 source 结论。
- 标准化起始指令：继续 E03：请使用 Tsukumo handoff 完成预注册的证据冻结与 claim-boundary 审计；来源、scope 和 receipt 可按需检查，本轮没有预注册 fault。

## 质量门

- 每项结论映射到具体可复核 artifact；
- 明确 capture manifest 的 causal-ineligible 边界；
- utility episode 完成前不得声称迁移或恢复效用；
- natural 与 controlled fault 分开；
- 不提前实施 release packaging；
- 不包含凭据、原始 prompt 或个人路径；
- 单纯查看 provenance 不计为首次正确动作。

## 当前工程门

- [x] C1/C2 使用相同 checkpoint / StateRef / prompt 数据面。
- [x] C2 summary 可暴露 receipt/provenance metadata，且不含 prompt/path。
- [x] 同一 data-dir 可由现有 TUI 查看 source/scope/最新 receipt 并执行正式 revoke。
- [x] 重复 execution 在第二次 spawn 前失败。
- [ ] Codex source action 已真实完成并由主人审阅。
- [ ] reviewed spec 已通过 `episode seed` 写入正式 data-dir。
- [ ] Seeded at 与恢复窗口已从机器 summary 回填。

## 观察记录

恢复时复制 ../handoff-episode-template.md 的字段到本节并填写。首次正确动作以首段
最终保留、满足质量门的 evidence-freeze 分类结论为准。
