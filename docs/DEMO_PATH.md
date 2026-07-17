# Tsukumo V0：5 分钟 Demo Path 与 90 秒讲解脚本

> 目标：让招聘方在不登录 Claude/Codex、不写入本机长期状态的前提下，看见实际 TUI、可信交接架构和 stale-state re-entry。
>
> 本路径默认离线；只有显式标记为 live 的命令才允许启动 vendor CLI。

## 1. 前置条件

- 从仓库根目录执行；
- 使用 `rust-toolchain.toml` 固定的工具链；
- 首次构建需已有 Cargo 依赖缓存或允许 Cargo 下载依赖；
- 以下默认命令不会运行 ignored live tests。

## 2. 5 分钟离线路径

### Step A：验证可安装产物

```bash
install_root="$(mktemp -d)"
cargo install --path crates/tsukumo-host --locked --root "$install_root"
"$install_root/bin/tsukumo-host" --version
rm -rf "$install_root"
```

期望版本：`tsukumo-host 0.1.0`。

发布 tag 可用后，招聘方也可以执行：

```bash
cargo install --git https://github.com/EMT5320/tsukumo \
  --tag v0.1.0 tsukumo-host --locked
```

### Step B：看实际 renderer

```bash
cargo run -p tsukumo-host --offline --example workshop_frame -- full --ansi
cargo run -p tsukumo-host --offline --example workshop_frame -- states --ansi
cargo run -p tsukumo-host --offline --example workshop_frame -- projection --ansi
cargo run -p tsukumo-host --offline --example workshop_frame -- permission --ansi
```

这四个模式分别展示：

1. workshop 与 Shiori；
2. Canonical State inspector；
3. projection receipt / provenance inspector；
4. fail-closed permission modal。

它们运行真实 renderer 与确定性夹具，不启动 vendor CLI。

如需直接打开交互式 TUI：

```bash
TSUKUMO_DATA_DIR="$(mktemp -d)" cargo run -p tsukumo-host --offline
```

键位：`W` workshop、`S` state、`P` projection、`R` refresh、`X` revoke、`Q` quit。

### Step C：复现 stale-state re-entry

仓库内 `.run/episode-review-2026-07-14/` 是永久禁止 seed/resume 的真实 stale-state 样本，只允许 inspect：

```bash
cargo run -p tsukumo-host --offline -- episode inspect \
  --spec .run/episode-review-2026-07-14/c2.review.json \
  --runtime-executable claude \
  --working-dir .
```

期望关键字段：

```json
{
  "overall_status": "drifted",
  "semantic_review_required": true,
  "mutation_performed": false
}
```

实际 runtime 版本会随本机环境变化；找不到 runtime 时应得到 blocked/unknown 类 finding，而非启动或改写 episode。

### Step D：跑最小合同门

```bash
cargo test -p tsukumo-host --test cli_parse_contract --offline
cargo test -p tsukumo-host --test episode_inspect_contract --offline
```

完整质量门：

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --offline
```

## 3. 90 秒 renderer walkthrough 与讲解脚本

已生成的 H.264 作品集视频：[`assets/tsukumo-v0-walkthrough.mp4`](assets/tsukumo-v0-walkthrough.mp4)。容器经 AVFoundation 反读为 `90.0s`、`1280×720`；视频使用下列真实 deterministic renderer 画面、架构页和证据页，不包含桌面录制、vendor live session、用户路径或通知。

| 时间 | 画面 | 讲解 |
|---|---|---|
| 0–12 秒 | `docs/assets/tsukumo-v0-demo.gif` / Full workshop | “Tsukumo 把 Claude、Codex 等外部 Agent 的工作状态变成用户持有、可检查、可撤销的交接记录；九十九工房是它的可观察产品面。” |
| 12–28 秒 | README Mermaid 架构图 | “五个 crate 单向依赖。Adapter 只翻译 vendor 事件，Kernel 分配身份，Soul 持有三本账，Theater 只消费事件，Host 负责进程和安全。” |
| 28–45 秒 | State Inspector | “这里看到的是 Canonical State：状态有来源、作用域和版本，不把自由文本直接当成可执行真相。” |
| 45–60 秒 | Projection Inspector | “投影时先提交不可变 receipt，再启动目标 runtime。即使启动失败，也能知道当时准备交给谁、选择了哪些状态。” |
| 60–73 秒 | Permission modal | “权限默认 fail-closed；未批准的执行不能靠 TUI 动画绕过安全边界。” |
| 73–86 秒 | `episode inspect` JSON | “隔天回来先只读对账。旧 Git、artifact 或 runtime 发生漂移时，系统显式报告 drifted 并要求人工复核，不擅自继续。” |
| 86–90 秒 | Evidence 页 | “当前证据覆盖离线合同、Linux/Windows CI 和 opt-in live seam；还没有声称效率或用户价值提升。” |

## 4. 媒体标注纪律

推荐 caption：

> Actual deterministic renderer capture using a fixed `ProductView`; no live Claude/Codex session or user data is included.

不要使用：

- “production session”；
- “real-time Claude/Codex run”；
- “效率提升”；
- “跨任意 runtime”；
- “完整 Agent memory platform”。

概念图只标注为 concept / character reference；实际截图、GIF 与 90 秒 H.264 walkthrough 位于 `docs/assets/`。

## 5. Live 路径（不属于默认 Demo）

只有在本机 CLI 已登录、额度和网络已确认、并显式授权后运行：

```bash
TSUKUMO_RUN_LIVE_SMOKE=1 \
  cargo test -p tsukumo-host --test claude_live -- --ignored

TSUKUMO_RUN_LIVE_SMOKE=1 \
  cargo test -p tsukumo-host --test cross_runtime_live -- --ignored
```

live smoke 只验证当次 CLI seam，不等价于用户效用评测。
