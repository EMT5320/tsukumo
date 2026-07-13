# Journal - EMT5320 (Part 1)

> AI development session journal
> Started: 2026-07-08

---



## Session 1: Path B 骨架与关系探针落地

**Date**: 2026-07-10
**Task**: Path B 骨架与关系探针落地
**Branch**: `main`

### Summary

完成 KernelEvent、theater、stream-json adapter 与 soul recall/brief 探针；确认下一阶段转向 C1 Handoff Continuity。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `0f5fee4` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 2: 重建 Tsukumo Rust Trellis 规范

**Date**: 2026-07-10
**Task**: 重建 Tsukumo Rust Trellis 规范
**Branch**: `main`

### Summary

移除误判的 backend/frontend 模板，建立 Rust 架构、事件、状态、theater、错误处理与质量规范，并归档 bootstrap 任务。

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `17794a2` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 3: C1 Handoff Continuity 规划收敛

**Date**: 2026-07-10
**Task**: C1 Handoff Continuity 规划收敛
**Branch**: `main`

### Summary

完成 C1 需求收敛、Rust code-spec、父子任务设计与执行计划；任务保持 planning，等待回家后从 Contracts/Chronicle 开始实现。

### Main Changes

- 冻结 SQLite 单一事实源、混合状态提取与确定性写入门。
- 冻结 Claude CLI + Codex CLI 双运行时、stdin prompt 与 fixture/live smoke 边界。
- 冻结 ProjectionReceipt 分层留存策略，原始 prompt 不落盘。
- 建立一个父任务、四个顺序子任务及完整 PRD/design/implement 制品。
- 补充 runtime adapter、state/persistence code-spec 与官方 CLI research。

### Git Commits

| Hash | Message |
|------|---------|
| `6bcb20d` | (see git log) |

### Testing

- [OK] 五个 Trellis task context validation 全部通过。
- [OK] PRD convergence、未决标记、生成制品空白和 `git diff --check` 通过。
- [OK] 产品代码未修改；规划提交范围排除了无关工具目录。

### Status

[PAUSED] **Planning complete; implementation not started**

- Local `main` contains the planning and journal commits.
- `git push origin main` was attempted and rejected with GitHub HTTP 403; the
  configured credential needs repository write access before remote sync.

### Next Steps

- 回家后运行 `trellis-continue`，审阅提交 `6bcb20d`。
- 重新授权 GitHub 凭据后运行 `git push origin main`；当前本地提交不得丢弃。
- 启动 `07-10-c1-contracts-chronicle`，加载 `trellis-before-dev` 后开始实现。


## Session 4: Complete C1 contracts and Chronicle

**Date**: 2026-07-11
**Task**: Complete C1 contracts and Chronicle
**Branch**: `main`

### Summary

Implemented and verified frozen event contracts, Chronicle persistence, deterministic StateWriter, safe legacy migration, derived exports, and cross-layer replay; five-lane review passed with 94 offline tests.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `50bed37` | (see git log) |
| `1cf9cfa` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 5: C1 handoff and projection MVP gate

**Date**: 2026-07-11
**Task**: C1 handoff and projection MVP gate
**Branch**: `main`

### Summary

Converged V0 scope, implemented immutable checkpoints and receipt-first projections, added deterministic comparison and privacy guards, passed 105 offline workspace tests, and advanced C1 to Host/Runtime.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `64113fb` | (see git log) |
| `537e2d4` | (see git log) |
| `60bef70` | (see git log) |
| `369beae` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 6: Complete C1 Host runtime

**Date**: 2026-07-11
**Task**: Complete C1 Host runtime
**Branch**: `main`

### Summary

Implemented and verified the receipt-first Claude Host, bounded process lifecycle, incremental Chronicle-before-Theater flow, fail-closed Safety Plane, observable vendor skips, and an isolated allowlisted live smoke.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `4f8231c` | (see git log) |
| `b22b71a` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 7: Complete V0 MVP product TUI

**Date**: 2026-07-12
**Task**: Complete V0 MVP product TUI
**Branch**: `main`

### Summary

Implemented the stage-first pixel TUI, modular presentation packs, durable Host actions, guarded SQLite recovery, adaptive rendering, terminal lifecycle, and complete validation evidence.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `802d0d4` | (see git log) |
| `aa68693` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 8: V0 go/no-go 复核与护城河收束

**Date**: 2026-07-13
**Task**: （无 Trellis 任务——review/决策 session，经主人确认直接修订设计文档落地）
**Branch**: `main`

### Summary

对 MVP 现状做全面 review（代码盘点 + 四赛道市场复核 + 增强层护城河质询），结论：**有条件 GO**——方向不变、不大改、不放弃；护城河论述更新，V0 验收纪律加严。

### Main Changes

- **代码盘点**：C1 纵向切片扎实（KernelEvent/Chronicle/checkpoint/receipt/Claude host/栞 TUI，~2.3 万行，170+ 契约测试）；缺口 = Codex adapter、TUI→执行通路、live 权限桥、成长机制、发布包装。
- **市场复核**：Pixel Agents 1.3 万→7.4 万装机；Vibe Kanban 2026-04 停运；OpenMemory 商品化跨工具记忆；Buddy/CodePal 抢注表层养成。护城河修正为"拥有会话 × 真实成长数据 × 跨 runtime 连续性"三者交集。
- **增强层质询**（主人提出）：Trellis 式寄生层能实现功能性 80%；tsukumo 增量价值收束为四件寄生层够不着的事（事务性捕获 / 拥有会话 / per-person / 抗宿主吸收）。V0 demo 对照组升级为"自律的 Trellis 用户"，三硬标准：零自觉捕获、热交接、审批闭环。
- **决议落账**：DESIGN.md 修订（§1.2 市场更新 / §1.4 痛点四 / §2.7 新增 / §5.6 布局对齐 / §16 V0 验收纪律 / §18 优先级重申 / §19 开放问题 / §20 竞品增补）；cross-runtime-evidence implement.md 追加 step 0 Codex 侦察 spike；隐患 4/5/6（分发 / schema 承诺 / 依恋 n=1）降级为 post-V0 非紧迫开放项。
- **优先级重申**：求职技术展示优先（Loomstead→Tsukumo 接力叙事），产品化/市场竞争非紧迫。

### Git Commits

| Hash | Message |
|------|---------|
| (this commit) | `docs: record V0 go-no-go review` |

### Testing

- N/A（纯文档修订，无产品代码变更）

### Status

[OK] **Completed**

### Next Steps

- 启动 `07-10-c1-cross-runtime-evidence`，从 step 0 Codex 侦察 spike 开工。
- ~~时间盒：2026-07-26 前三个活跃任务全关 + 热交接 demo 可录屏。~~ 本轮原决议，已由 Session 9 的 2026-07-23 效用门取代。
- live smoke 每周至少手动跑一次并记录 CLI 版本。


## Session 9: Trellis 强基线与 Loomstead 效用门复核

**Date**: 2026-07-13
**Task**: `07-10-c1-cross-runtime-evidence` 产品 / 研究协议纠偏
**Branch**: `main`

### Summary

在主人提出“重型 Tsukumo 是否仍有护城河”与“如何证明 trace 有用”后，复核 Trellis 实际能力及 Loomstead 研究档案。结论收窄为：**验证性 GO 至 2026-07-23，当前无已证明护城河；P0 保留可信交接，并以续接 / 恢复成本而非链路完整性作结果变量。**

### Main Changes

- **Trellis 强基线**：承认 context injection、channel runtime、共享任务制品和主人调度已经覆盖跨工具协调的大部分价值；“事务捕获 / 拥有会话 / per-person / 聚合层”均从结构性壁垒降为待测能力。
- **Loomstead 负面先验**：复核 `human_rating_pilot_gate.md`、`process_fidelity_eval_spec.md` 与 research transition audit；明确“证据完整 → 行为敏感 → 任务效用 → 恢复效用 → 净期望价值”证据阶梯。
- **根本差异**：Loomstead 研究受控 runtime 内的动作来源；Tsukumo 验证用户状态跨独立 Claude / Codex runtime 后是否降低真实续接、纠错和恢复成本。系统边界与因变量构成差异，Rust / TUI 不构成研究差异。
- **三条件协议**：C0 Trellis-only、C1 自动迁移但隐藏追溯、C2 自动迁移 + provenance / selective revoke；加入 stale、wrong-scope、contradiction 故障与自然发生率分离报告。
- **决策门**：7/22 冻结，7/23 GO / PIVOT / NO-GO；迁移门约为首次正确动作改善 30% 或人工干预减少 50%，恢复门约为定位 / 恢复改善 50%，正常开销预算约 5–10%。这些是 n=1 产品门，不是统计结论。
- **产品形态解冻**：Trellis 作为声明 / 协作层，Tsukumo 作为 evidence sidecar，TUI 作为可选控制与作品集展示面；权限闭环、陪伴和养成扩展降为 P2。

### Git Commits

| Hash | Message |
|------|---------|
| (this commit) | `docs: define trusted-handoff utility gate` |

### Testing

- [OK] 交叉核对 Loomstead 三份原始研究材料与 Tsukumo 当前任务制品。
- [OK] Trellis task context validation 与 `git diff --check`（提交前执行）。
- [OK] 纯文档 / 协议变更；无产品代码与测试契约变更。

### Status

[OK] **Completed — validation protocol frozen through 2026-07-23**

### Next Steps

- 7/14 前冻结 C0/C1/C2 观察模板，并种下一个 48–72 小时延迟恢复任务。
- 完成 Codex event-surface recon，再实现 decoder；不得先按 Claude 事件形状猜接口。
- 7/17–7/21 记录 12–20 个 handoff episode；不足时如实报告，不用 token 总量替代样本。
- 7/22 形成正向与否证两套作品集叙事，7/23 按数据作最终决策。
