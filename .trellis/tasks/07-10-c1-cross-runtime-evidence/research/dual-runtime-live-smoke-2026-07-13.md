# Claude/Codex 双运行时 Live Smoke（2026-07-13）

## 目的

验证同一个 receipt-committed checkpoint 可以通过生产 Host 端口分别交给
Claude CLI 与 Codex CLI，并由各自 adapter 解码为成功的 durable outcome。
这是一项外部运行时可执行性门禁，不计入 C0/C1/C2 utility episode。

## 运行边界

- 时间：2026-07-13T23:24:30+08:00
- 平台：Windows，`stable-x86_64-pc-windows-gnu`
- Claude：`2.1.205 (Claude Code)`
- Codex：`codex-cli 0.135.0`
- 测试：`crates/tsukumo-host/tests/cross_runtime_live.rs`
- 两个运行时各使用一个空临时目录。
- 两次执行共享同一个 Spirit、checkpoint 和 reviewed projection digest。
- 固定目标仅要求返回 `TSUKUMO_CROSS_RUNTIME_LIVE_OK`，并明确禁止工具调用。
- Claude profile 设置 `--max-budget-usd 0.05`；Codex profile 使用隔离配置、
  `approval_policy="never"` 和只读 sandbox。
- 未记录认证文件、凭据、原始运行时输出或个人路径。

## 命令与结果

```powershell
$env:TSUKUMO_RUN_LIVE_SMOKE='1'
cargo +stable-x86_64-pc-windows-gnu test -p tsukumo-host --test cross_runtime_live --offline -- --ignored --nocapture
```

结果：`1 passed; 0 failed`，测试执行耗时 `17.83s`。两个版本探针先通过，
随后 Claude 与 Codex owned process 均经 Host receipt preflight、stdin projection、
adapter decode、Chronicle append 和 Theater fan-out 完成，最终状态均为
`OutcomeStatus::Succeeded`。

## 成本记录

- 外部模型调用：Claude 1 次，Codex 1 次。
- Claude 实际费用：不可得；测试仅能证明上限为 0.05 美元。
- Codex 实际 token/费用：不可得，未估算。
- `17.83s` 是完整测试耗时，不能拆分为单个运行时延迟。

## 声明边界

本次结果证明本机认证状态下的双 CLI 生产路径可执行，并满足显式启用、
版本记录、隔离目录和固定 payload 的工程门禁。它没有测量真实任务的
first-correct-action time、owner intervention、task quality、故障恢复时间或
自然故障发生率，因此不改变 C1/C0 与 C2/C1 的 utility gate 状态。
