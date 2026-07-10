# Implement: 实现路径 B 与关系层切片

- **Task**: `.trellis/tasks/07-09-impl-path-harness-landscape`
- **Date**: 2026-07-09
- **Depends on**: `prd.md`, `design.md`, `notes-executor-role-model.md`, `research/*`
- **Constraint**: 根目录 `DESIGN.md` 只读北极星，**禁止修改**。执行层 ontology 未拍板（`design.md` §6），实现勿写死物种分表。

> 本文是执行 checklist，不是把范围扩成完整产品。本任务规划阶段以「可开工的切片」为交付；真正写代码须先 `task.py start` 且主人确认。

---

## 0. Guardrails（全程）

- [x] 不修改根目录 `DESIGN.md`
- [x] 不落地「付丧神 vs 雇佣兵」两套成长表；身份与 `BackendKind` 分离预留即可
- [x] 不进 GEPA / 完整 §8 / 完整 ACP fs·terminal / 领悟技能 UI
- [x] 关系探针遵守 token 纪律（`design.md` §2.4）：禁止每 tool call 提炼
- [ ] A1 驱动级通过不以「JSONL 观察级好看」冒充

---

## 1. Ordered Checklist

### Phase P0 — 工程骨架（可与调研结论并行收尾）

- [x] **P0.1** 确认仓库仍无应用 crate 时：起草最小 Cargo workspace 草案（可调，不锁死切分）  
      已落地：`crates/tsukumo-kernel` + `crates/tsukumo-theater`；`adapters` / bin 延后至 A1/K0。
- [x] **P0.2** 定义 `KernelEvent` 最小可演集合（tool_start / tool_end / waiting_permission 或等价 / turn_or_quest_end / error）  
      vendor 细节进 adapter 私有，不进 theater。
- [x] **P0.3** 定义 `StageEvent` 最小集合（actor_pose / bubble / log_line / attention_tier）  
      Director 签名保持纯函数可测：`direct(event, ctx) -> Vec<StageEvent>`。
- [x] **P0.4** 夹具：一份录制/手写 JSONL 或内存事件流，供舞台与 Director 单测。  
      `crates/tsukumo-theater/fixtures/minimal_quest.jsonl` + `tests/fixture_replay.rs`。

**验证：** `cargo test`（Director 纯函数）+ 夹具回放无 panic。  
**本机注记（2026-07-09）：** 默认 `x86_64-pc-windows-msvc` 缺 `link.exe`；用 `cargo +stable-x86_64-pc-windows-gnu test` 通过（13 tests）。

**回滚点：** 仅类型与测试，无外部进程依赖。

---

### Phase S — 舞台瘦身线（F0，不阻塞 A1 通道选型）

- [x] **S0** HalfBlock（或既定降级）静态工房 + 1 sprite 占位  
      验收参考根设计 S0 清单精神：Win Terminal 可显示、中文宽字符不炸、可折叠布局可后做。  
      落地：`render.rs`（ratatui Canvas `Marker::HalfBlock`）+ `examples/workshop_demo` print 模式；CJK 经 `buffer_to_string` 宽字符跳格。
- [x] **S1** Idle/Walk/Work 状态机 + 气泡 + 分屏日志消费同一 `StageEvent` 流  
      演出不阻塞：事件可合并/丢帧（有损采样）。  
      落地：`world.rs`（`Motion` + 有损 log cap + `tick` 行走）消费 `StageEvent`；Director 仍纯。
- [x] **S1b** 用 P0.4 夹具驱动舞台，证明「假事件也能演戏」。  
      落地：`drive.rs` + `drive::tests::fixture_drives_stage_world_acting`；demo 同路径。

**验证：** 人工：`cargo +stable-x86_64-pc-windows-gnu run -p tsukumo-theater --example workshop_demo`；自动化：StageEvent → 渲染状态快照 / fixture→world 测试。  
**本机注记：** 继续用 `cargo +stable-x86_64-pc-windows-gnu`（MSVC 缺 `link.exe`）。

**回滚点：** 舞台可独立砍分辨率/帧率；不影响 adapter。

---

### Phase K — 瘦宿主（非完整 coding agent）

- [ ] **K0** 最小 loop 或「会话宿主壳」：能组装 prompt、能挂 adapter、能写 session JSONL  
      **不做**与 Claude Code 比拼的工具完备性。
- [ ] **K1** 自有或夹具路径产出完整最小 `KernelEvent` 流，经 Director → 舞台（M1 雏形）。

**验证：** print/replay 模式跑通一次假任务；JSONL 可回放。

**回滚点：** 可整段用「只跑 adapter、builtin 更瘦」替代，保持事件契约。

---

### Phase A1 — 双通道务实（F1 ★ / F2）

- [x] **A1.0** Windows 通道 spike（当天结论写入 `research/` 或本任务 `notes-a1-channel.md`）  
      - 试 ACP bridge（若可得）  
      - 试自有进程 stream-json（或等价）  
      - 记录：tool/wait/end 信号质量、权限事件是否可得  
      **结论（2026-07-09）：** `claude` 2.1.205 可用，`stream-json` 实测通；ACP 包更名且 `npx` 易挂，**默认 drive = stream-json**；live `waiting_permission` 未在短 plan 跑中捕获，CI 用 synthetic。详见 `notes-a1-channel.md`。
- [x] **A1.1** 选定主通道实现第一个 drive adapter → `KernelEvent`  
      失败则切另一通道，不扩 scope 到 fs/terminal 代理。  
      落地：`crates/tsukumo-adapters`（Claude-like stream-json 子集解析 + synthetic 生产者）。
- [x] **A1.2** 映射等待/审批到 StageEvent（UI 可 stub：日志 + 小人举手即可）  
      Director 已有 `WaitingPermission` → `AttentionTier::Urgent` + `ActorPose::Wait`；adapter 产出同契约事件即可。
- [x] **A1.3** 对接 S1：真实（或半真实）外部事件驱动舞台 —— **F1 门禁**  
      集成测 `a1_stream_json_waiting_permission_raises_urgent` + example `a1_stream_demo`；`NullBriefing` / `assemble_prompt` 为 R 相组装点。

**验证命令（示例，按实际 bin 名调整）：**

```text
# 通道探测结论：notes-a1-channel.md

cargo +stable-x86_64-pc-windows-gnu test
cargo +stable-x86_64-pc-windows-gnu test -p tsukumo-adapters -- a1_
cargo +stable-x86_64-pc-windows-gnu run -p tsukumo-adapters --example a1_stream_demo -- --stop-at-wait
# 人工：等待/审批时舞台 attention=Urgent；结束有结算或 log
```

**A1 通过定义（抄 design）：** 事件保真 + 等待/审批可演 + 简报组装点存在（见 R 相）。  
**本轮诚实边界：** F1 以 synthetic/recorded stream-json 证伪；live Claude ACP 握手与 live permission 捕获仍后置（见 notes）。

**回滚点：** 弃 ACP 保 stream-json；或 A1 仅达「事件→日志」而舞台仍用夹具（须在笔记标明 F1 未过）。

---

### Phase R — 中偏早关系探针（F3 工程前置）

- [x] **R1** Soul store 最小：canonical 快照文件（MEMORY/USER 或等价）+ 可选 sqlite FTS  
      落地：`crates/tsukumo-soul` — `MEMORY.md`/`USER.md` + `soul.db`（rusqlite bundled + FTS5，LIKE 回退）。
- [x] **R2** `BriefCompiler`：容量封顶 + top-k（上限数字实现时标定，原则已定）  
      默认：`DEFAULT_BRIEF_CHAR_CAP = 800`，`DEFAULT_TOP_K = 5`。
- [x] **R3** 在 A1 的 prompt 组装点注入简报（无二次元人格词）  
      soul：`assemble_delegation_prompt` / `PromptAssembler`；adapters 既有 `BriefingSource` + `assemble_prompt` 保持不变，host 接线。
- [x] **R4** 跨一次会话边界的召回演示（同一主机数据目录）—— **F3 最小证伪**  
      `tests/cross_session_recall.rs` + `examples/recall_demo`。
- [x] **R5** skills 目录与 trait 插座占位（空实现 / 不暴露领悟 UI）  
      `skills/` + `SkillSocket` / `SkillStub`（`skill_create` 恒 false）。
- [x] **R6**（可选）注入/召回写一条可追溯日志 stub  
      `TraceLog` → `inject_trace.jsonl`。

**验证：**

```text
cargo +stable-x86_64-pc-windows-gnu test -p tsukumo-soul
cargo +stable-x86_64-pc-windows-gnu run -p tsukumo-soul --example recall_demo
# 会话 1：写入一条可召回事实到 soul store
# 会话 2：简报或 recall 命中该事实（人工或集成测）
```

**回滚点：** 简报可改夹具字符串；召回可降级为「读整个冻结快照」但须仍遵守容量封顶。

---

### Phase G — 规划收尾（本任务可在 start 前完成的文档门）

- [x] **G1** 固化 `implement.jsonl` / `check.jsonl` 真实条目（非 `_example`）
- [x] **G2** 主人审阅 prd + design + implement 后选 A 开工；执行层专项仍开放（见 `notes-executor-role-model.md`）
- [x] **G3** `task.py start` 已执行（2026-07-09）；P0/S/A1/R 切片已落地，check 裁决 MERGEABLE（warn：K0/K1 未做、live ACP/permission 后置）
- [ ] **G4**（可选后续）K0/K1 瘦宿主壳；真 Claude 进程 / ACP smoke；host 把 soul brief 接入 adapters `assemble_prompt`

---

## 2. Validation Matrix

| ID | 命令/方式 | 对应门禁 |
|---|---|---|
| V-unit | `cargo test` | P0 Director / 事件契约 |
| V-fixture | 夹具 → 舞台 | S1b / F0 辅助 |
| V-a1-spike | Windows 上手动 ACP vs stream-json | F2 |
| V-a1-drive | 外部事件 → 舞台 attention | F1 ★ |
| V-brief | 会话 2 命中会话 1 记忆 | F3 最小 |
| V-budget | 代码审查：无 per-tool 提炼 LLM | §2.4 纪律 |
| V-northstar | `git diff -- DESIGN.md` 为空 | 北极星只读 |

---

## 3. Risky Files / Rollback Points

| 区域 | 风险 | 回滚 |
|---|---|---|
| adapter 进程生命周期 | 僵尸进程 / 权限模式 | 杀进程；改 stream-json |
| 简报进 prompt | 污染工具默认人设 | 简报仅事实句；可关编译器 |
| 舞台帧率 | Win Terminal 卡顿 | 降 fps / 折叠舞台 |
| 过早角色 schema | 两套成长 | 删物种字段，只留 id+backend |

---

## 4. Explicit Non-Goals（本 checklist 不做）

- 改根 `DESIGN.md`
- 执行层 ontology 定稿、主搭档人设
- 完整 §8 受管区块 / MCP 记忆服务产品化
- 多 runtime 观察级全覆盖、闲时生态、GEPA
- 发布 / 求职评测件

---

## 5. Follow-up Before `task.py start`

1. 主人确认本 implement 切片可接受（尤其 A1 与 R 探针仍旁路并行）。  
2. 执行层专项：继续讨论或另建任务 —— **不阻塞** P0/S/A1 开工，但阻塞「按角色物种建表」。  
3. `implement.jsonl` / `check.jsonl` 已填真实 research/spec 条目。  
4. 若平台要求 before-dev：开工时再跑 `trellis-before-dev` 拉 frontend/backend 指南。
