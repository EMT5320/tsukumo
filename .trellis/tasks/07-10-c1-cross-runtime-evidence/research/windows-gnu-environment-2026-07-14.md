# Windows GNU 工具链环境记录（2026-07-14）

## 结论

仓库代码没有新增 GNU 编译错误。原阻塞来自本机 `C:\\MinGW\\bin\\gcc.exe`
仅支持 32 位，无法处理 `-m64`。在不安装工具、不修改全局 PATH 的前提下，使用
单次命令环境组合后，以下门禁已真实通过：

```text
cargo +stable-x86_64-pc-windows-gnu check --workspace --all-targets --offline
```

结果：0 退出，约 2 分 01 秒。

## 单次命令组合

- VS2019 Build Tools x64 `cl.exe` 编译 C（供 `libsqlite3-sys` 使用）；
- `C:\\MinGW\\bin\\ar.exe` 仅负责归档；
- Rust GNU 1.97 工具链自带 `rust-lld.exe` 负责链接 check 产物；
- `RUSTFLAGS=-C linker-flavor=ld.lld`；
- target-dir 放在用户临时目录；
- 没有修改仓库、系统 PATH 或用户全局配置。

当前会话可复用脚本：

```bash
'/mnt/c/Windows/System32/cmd.exe' /d /c '\\\\wsl.localhost\\Ubuntu\\tmp\\tsukumo-gnu-recovered-check.cmd'
```

该 `/tmp` 脚本是本机临时辅助，不是仓库制品。

## 诚实边界

GNU `cargo test` 的最终可执行链接仍需要真正的 x64 MinGW binutils/CRT。尝试混合
MSVC 静态 CRT 与 GNU CRT 会出现 `__security_cookie`、`__GSHandlerCheck`、
`_fltused` 和 CFGuard 重复符号，ABI 风险不可接受，因此已停止。

当前发布证据可表述为：

- Windows MSVC：workspace fmt/check/clippy/test 全量门禁已通过；
- Windows GNU：workspace/all-targets 离线 check 已通过；
- Windows GNU 可执行链接与 tests：本机缺少 x64 MinGW CRT/binutils，尚未通过；
- Linux/CI 可复现性仍属于后续 release-packaging 任务，不能从本机门禁推断。

若未来需要 GNU 可执行物，最小安全环境修复是安装 user-local/portable
`x86_64-w64-mingw32` 工具链，并仅在单次命令 PATH 前置。
