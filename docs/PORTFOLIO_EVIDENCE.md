# Tsukumo V0 作品集证据索引

> 状态：V0.1 发布候选证据；最后核验日期：2026-07-17。
>
> 本页区分“本轮复现”“持久化 CI”“历史 live smoke 记录”和“尚未证明”，避免把契约测试或演示夹具升级成用户价值结论。

## 1. 证据快照

| 层级 | 证据 | 当前结果 | 能证明什么 | 不能证明什么 |
|---|---|---|---|---|
| 本轮复现 | `cargo fmt --all --check` | 通过 | Rust 格式门 | 运行时正确性 |
| 本轮复现 | `cargo clippy --workspace --all-targets --offline -- -D warnings` | 通过 | 当前 workspace 无 clippy warning | 跨平台行为 |
| 本轮复现 | `cargo test --workspace --offline` | `256 passed / 0 failed / 2 ignored` | 离线合同、状态链、TUI 与 CLI 回归 | 真实 vendor CLI 可用性或用户收益 |
| 本轮复现 | `cargo install --path crates/tsukumo-host --locked --root <temp>` | 安装成功，`tsukumo-host 0.1.0` | 当前 checkout 可构建、可安装 | crates.io 已发布或预编译二进制存在 |
| 本轮复现 | `workshop_frame` 16 个确定性模式 | 全部生成成功 | 实际 renderer 的 Full / Inspector / modal / pose / 降级画面 | 真实 Claude/Codex session 录屏 |
| 持久化 CI | GitHub Actions：Linux + Windows GNU | 当前 HEAD 全绿 | 两个平台上的 fmt / clippy / tests | macOS、tmux 或 legacy conhost 全覆盖 |
| 历史记录 | 2026-07-16 opt-in live smoke | Claude 单 runtime、Claude→Codex 双 runtime 记录为通过 | 当日指定 CLI 版本的受控 live seam 可用 | 持续兼容、效用提升或生产可靠性 |

当前公开 CI：<https://github.com/EMT5320/tsukumo/actions>

发布后以 `v0.1.0` tag 对应 revision 和该 revision 的 CI 为最终发布证据。

## 2. 实际作品集媒体

- [`assets/tsukumo-v0-actual.png`](assets/tsukumo-v0-actual.png)：实际 `tsukumo-theater` / `tsukumo-host` renderer 通过确定性 `ProductView` 生成的 Full 模式截图。
- [`assets/tsukumo-v0-demo.gif`](assets/tsukumo-v0-demo.gif)：依次展示 Full、State Inspector、Projection Inspector、Permission modal 与 Shiori 五态。
- [`assets/tsukumo-v0-walkthrough.mp4`](assets/tsukumo-v0-walkthrough.mp4)：90.0 秒、1280×720 的 H.264 walkthrough；按 Demo 脚本组合实际 renderer 画面、五 crate 架构、只读 inspect 与证据边界页。
- 生成入口：

```bash
cargo run -p tsukumo-host --offline --example workshop_frame -- full --ansi
cargo run -p tsukumo-host --offline --example workshop_frame -- states --ansi
cargo run -p tsukumo-host --offline --example workshop_frame -- projection --ansi
cargo run -p tsukumo-host --offline --example workshop_frame -- permission --ansi
```

这些媒体必须标注为 **actual deterministic renderer capture / walkthrough**。它们使用真实渲染代码和固定夹具，不启动 Claude/Codex，也不冒充 live session。视频中的架构与证据页是解释性版面，不是 runtime 画面。

`docs/visual-references/` 下的两张图片只用于概念和角色视觉契约；不得标注为产品运行截图。

## 3. 五 crate 实现边界

| Crate | 已实现边界 | 主要证据入口 |
|---|---|---|
| `tsukumo-kernel` | 身份、`KernelEvent`、session JSONL、脱敏 | crate tests、workspace tests |
| `tsukumo-adapters` | Claude / Codex / synthetic 事件归一化 | adapter contract tests、opt-in live seam |
| `tsukumo-soul` | Chronicle、Canonical State、handoff、projection、receipt、revoke | soul integration / invariant tests |
| `tsukumo-theater` | `StageEvent`、Director、HalfBlock 工房、presentation pack | visual contract、deterministic renderer |
| `tsukumo-host` | 进程生命周期、Safety Plane、TUI、episode inspect/seed/resume | CLI/episode/runtime contract tests |

依赖保持单向：`host → soul/theater/adapters/kernel`；`soul/theater/adapters → kernel`。

## 4. 可信交接与 re-entry 证据

### Receipt-first

`episode resume` 在目标进程启动前提交 projection receipt；启动失败保留失败语义，不把未记录投影伪装为已交接。相关边界由 episode runner、runtime launch、projection receipt 合同覆盖。

### 只读 re-entry

以下命令使用仓库内永久标注为 stale、禁止 seed/resume 的审阅材料：

```bash
cargo run -p tsukumo-host --offline -- episode inspect \
  --spec .run/episode-review-2026-07-14/c2.review.json \
  --runtime-executable claude \
  --working-dir .
```

2026-07-17 本轮复现结果：

- exit code：`0`
- `overall_status`：`drifted`
- target runtime：reviewed `2.1.205`，observed `2.1.211`
- `semantic_review_required`：`true`
- `mutation_performed`：`false`

该输出证明“旧审阅状态可被只读对账并显式标记漂移”；它不自动判断自然语言 open loop 是否仍应继续，也不写 Soul/Chronicle。

## 5. Live seam 证据边界

`claude_live` 与 `cross_runtime_live` 默认 ignored，只有显式设置 `TSUKUMO_RUN_LIVE_SMOKE=1` 才会启动已登录 vendor CLI。2026-07-16 的 DESIGN 记录包括：

- Claude Code `2.1.211` 单 runtime；
- Claude Code `2.1.211` → Codex CLI `0.144.5` 双 runtime；
- 本机 Codex provider 流需显式继承 `HTTPS_PROXY` / `ALL_PROXY`。

本页不把该历史记录写成“本轮复现”。发布前若未再次授权 live 调用，保持这一证据等级。

## 6. 可公开陈述

- 使用 Rust 五 crate workspace 构建本地优先的 Agent 状态、交接与 re-entry 层；
- 将 Claude/Codex vendor 流归一为 vendor-neutral `KernelEvent`，由 Kernel 分配持久身份；
- Chronicle、Canonical State 与 Handoff/Projection 分账，projection receipt 在 runtime 启动前提交；
- `episode inspect` 只读核对 Git、artifact、open loop 与 runtime 漂移；
- 九十九工房 TUI 具备 workshop、state/projection inspector、permission modal、Shiori 五态与降级模式；
- 离线质量门和 Linux / Windows GNU CI 具备可复现证据。

## 7. 红线

不得仅凭现有证据声称：

- 提升开发效率、恢复速度、任务质量或用户留存；
- 已完成 C0/C1/C2 的统计效用结论；
- 支持任意 runtime、完整 ACP、MCP 长期记忆或多 Agent 编排；
- crates.io 已发布、已有预编译二进制或已完成全终端兼容；
- deterministic renderer GIF 是 live Claude/Codex session；
- receipt/provenance 完整本身构成因果证明。

## 8. V0.1 延期项

以下不阻塞 V0.1：

- tmux 与 legacy conhost 实机回执；
- web/panel 第二消费端；
- ACP 主通道与 watcher；
- C0/C1/C2 扩样效用实验；
- 长期 snapshot / memory 产品生命周期；
- 多角色、成长、羁绊与内容包分发；
- crates.io 多 crate 发布和预编译二进制。
