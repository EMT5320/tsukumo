# C1 Contracts and Chronicle

## Parent

`.trellis/tasks/07-10-c1-handoff-continuity`

## Goal

建立 C1 后续所有层共用的身份、事件、Chronicle 与长期状态契约，使真实经历能够被稳定引用、持久化、重放和安全地转化为 StateRecord。

## User Value

主人能够知道 Tsukumo 记住了什么、证据来自哪里，并在状态错误或过期时撤销/替代它；后续 runtime 使用的每条硬约束都有可追溯来源。

## Confirmed Evidence

- 当前 `KernelEvent` 是无持久 envelope 的 payload enum，fixtures、adapter 与 theater 直接消费它。
- 当前 `SoulStore` 使用 `facts` + FTS，并把 Markdown 注释为事实源；C1 已冻结 SQLite 为唯一持久事实源。
- 当前 `TraceLog` 是独立 JSONL 且调用方会忽略 append 错误；C1 证据写入不得静默失败。

## Requirements

- 实现 `SpiritId`、execution/session/quest/event/state 等 typed IDs，以及独立 `RuntimeBinding`。
- 将 `KernelEvent` 重构为版本化 envelope + vendor-neutral payload。
- adapter 解码 payload，host/Chronicle writer 统一分配持久事件身份、时间、顺序与 correlation。
- 使用单一 SQLite 持久事实源，建立 append-only Chronicle 读写与 replay；StateRecord/evidence 与事件写入共享事务。
- JSONL、Markdown 与 FTS 仅作为可重建导出/索引。
- 采用 additive、幂等 schema migration；旧 `facts` 行若导入，只能通过 `legacy_imported` Chronicle 事件形成低强度状态，不得自动成为 `Explicit` 或硬 `Constraint`。
- 实现 stable `StateKey`、kind、Subject/Applicability、strength、status、evidence refs、TTL。
- 实现 deterministic StateWriter 的 create/supersede/revoke 与安全写入规则。
- 定义 provider-neutral `StateExtractor`/`StateDraft`；规则与结构化 LLM extractor 均无直接写库权限。
- 回归测试使用确定性/录制 extractor，live LLM 仅作可选 smoke。
- 为现有 fixture、adapter、Director 和 Soul probe 提供明确迁移路径。

## Acceptance Criteria

- [x] 已知事件可序列化、持久化、重开并以相同顺序重放。
- [x] tool/permission/projection 相关事件具有可追踪 execution/correlation。
- [x] theater 和 soul 不读取 vendor 原始 payload。
- [x] 显式 GNU toolchain 用户事件可形成带 scope/evidence 的 Constraint。
- [x] 单次 inferred draft 无法形成硬 Constraint。
- [x] malformed/timeout LLM 提取不会阻塞主任务，也不会写入半成品状态。
- [x] 同 key/scope 冲突不静默覆盖，revoke/supersede 保留历史。
- [x] Chronicle/evidence 写入失败可见，测试不允许静默丢失。
- [x] 删除导出文件后可从 SQLite 重建，且修改导出文件不会改变 canonical state。
- [x] 旧 `facts` migration 重跑幂等，且导入状态不会获得伪造的 explicit strength。

## Out of Scope

- Checkpoint/projector、live runtime host、第二 runtime、产品 UI。
