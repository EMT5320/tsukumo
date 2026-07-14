# Handoff Episode 预分配账本

> 冻结日期：2026-07-13
> 最低目标：12 个 episode；stretch target：20 个。
> 人工观察预算：每天最多约 20 分钟。条件启用前不得伪造 C1/C2 数据。

## 最低 12 个 episode

| ID | 条件 | Workload block | 方向 | 类型 / fault | 状态 |
|---|---|---|---|---|---|
| E01 | C0 | delayed-resumption | Codex → Claude | natural，48–72h | seeded |
| E02 | C1 | delayed-resumption | Claude → Codex | natural，48–72h | [wait-for-C1（已预注册）](episodes/e02-c1-delayed-toolchain-claim-audit.md) |
| E03 | C2 | delayed-resumption | Codex → Claude | natural，48–72h | [wait-for-C2（已预注册）](episodes/e03-c2-delayed-evidence-freeze-audit.md) |
| E04 | C0 | mid-task-code | Claude → Codex | natural | planned |
| E05 | C1 | mid-task-code | Codex → Claude | natural | wait-for-C1 |
| E06 | C2 | mid-task-code | Claude → Codex | natural | wait-for-C2 |
| E07 | C1 | recovery | Claude → Codex | controlled contradiction | wait-for-C1 |
| E08 | C2 | recovery | Claude → Codex | controlled contradiction | wait-for-C2 |
| E09 | C0 | analysis-docs | Codex → Claude | natural | planned |
| E10 | C2 | recovery | Codex → Claude | controlled wrong-scope | wait-for-C2 |
| E11 | C1 | recovery | Codex → Claude | controlled wrong-scope | wait-for-C1 |
| E12 | C0 | mid-task-code | Claude → Codex | natural | planned |

最低分配为 C0 / C1 / C2 各 4 个。E07/E08 与 E10/E11 是两组预注册恢复对照；受控故障不得计作自然发生率。

## Stretch slots

E13–E20 只在最低 12 个不挤压代码冻结和每日观察预算时启用。优先补充自然 stale/scope/conflict、第二个延迟恢复样本和不同任务类型，禁止看到结果后只补有利条件。

## 启动纪律

1. 每个 episode 启动前复制 `handoff-episode-template.md`，先填完预注册区。
2. C1/C2 只有在对应产品路径可用且通过最小工程门后才可启动。
3. Workload 的具体任务在 episode 启动前冻结；不得根据预期难度为某个条件挑选明显更容易的任务。
4. 条件无法按期启用时记录 shortfall，保留原门槛与否证叙事。