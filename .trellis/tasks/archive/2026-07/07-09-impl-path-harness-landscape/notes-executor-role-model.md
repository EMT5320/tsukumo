# Notes: 执行层角色模型（初步讨论，未拍板）

- **Date**: 2026-07-09
- **Status**: 开放 / 需后续专项深入；本轮不收敛 schema
- **Related**: `DESIGN.md` §2.4, §3.1, §9；`research/tsukumo-differentiation-implications.md`；`research/competitor-matrix.md`

## 主人初步论点（2026-07-09）

1. **工具付丧神与雇佣兵存在概念重合**  
   - 付丧神：初版设计（自有内核上的工具拟人）  
   - 雇佣兵：2026-07-08 愿景收束（外部 agent 作为可签约 runtime）  
   - 本质同一角色粒度：**具体执行层**；差别只是后端是自有伴生内核，还是兼容的第三方 agent 工具。

2. **不应拆成两套成长账本**  
   - 若「付丧神自成长」与「雇佣层自成长」分轨，会出现归属混乱，也难回答：用户为什么不直接用 Claude Code。  
   - 自有付丧神的能力不可能系统性强过 Claude Code 等重型 agent（符合反 Clippy / 不卷能力军备）。

3. **可移植性约束（设计压力测试）**  
   > 自有付丧神上的所有设定，都应该能在其他第三方工具上生效。  
   - 即：角色/灵魂设定是 **runtime 无关的执行者身份**，不是「绑死在自有 loop 上的皮」。  
   - 与已拍板的「关系自进化 + §8 化妆在出口 + canonical IR」同向，但 **角色 ontology 尚未定稿**。

4. **主搭档是否独立一层**  
   - 本轮未展开；三分法（付丧神 / 雇佣兵 / 主搭档）被指臃肿草率，整体搁置。  
   - 后续讨论需单独处理「对话脸 / 调度主体」是否等于执行层，还是关系层的另一投影。

## 与已拍板决策的兼容边界（可扩展，不锁死）

| 已拍板 | 对本题的含义 |
|---|---|
| 关系自进化主战场 | 成长主账本应是关系/记忆，不宜再按「自有工具 vs 外包 agent」拆两套能力成长 |
| 化妆在出口（§8.1） | 执行后端不知自己「扮演谁」；拟人在事件出口 —— 支持「同一执行者身份，多种后端」 |
| A1 双通道务实 | 先验证事件→舞台+简报；**不依赖**先定 Gina=git 还是 Gina=某 runtime |
| 记忆探针 / skill 留接口 | 探针挂在「谁」身上可后定；存储先按 canonical 灵魂 IR，避免过早按角色类型分表 |

**预留扩展点（实现时勿写死）：**

- `Character` / `Spirit` 与 `RuntimeBackend`（builtin loop | acp | stream-json | watcher）**分离**：身份 ≠ 进程  
- 成长事件打在 **spirit_id**（或统一 executor_id），不打在 vendor 名上  
- 自有工具（git/shell/fs）可视为 builtin backend 的能力面，而非另一类「物种」  
- 主搭档：暂用可选 `companion` 标记或独立 soul 槽，**允许后续合并进执行者或升为关系层门面**

## 调研侧相关信号（供后续深挖）

来自本任务 `research/`（非结论，仅线索）：

- **Hermes 替代风险**：用户若只要「一个越来越懂我的 agent」，会直接用 Hermes；Tsukumo 必须讲清多执行后端 + 关系资产不跟 vendor 走（`tsukumo-differentiation-implications.md` §5.1）  
- **Pixel / Cockpit**：spectacle 与 ops 已占位；他们的「角色」多半是会话/进程皮肤，不是跨 runtime 同一身份（`pixel-viz-companions.md`, `competitor-matrix.md`）  
- **静态 SOUL 包商品化**（AgentSoul 等）：若执行者设定不能随经历增长且不能挂到第三方工具，会沦为又一份可下载人设文件  
- **Process Fidelity 空位仍在**：无论 ontology 怎么切，数值来自真实经历、跨后端可追溯，仍是相对空的层

## 后续专项应回答的问题（勿本轮拍板）

1. 执行者身份的主键是什么？（spirit card / 契约 / 主人侧伙伴 ID）  
2. git 事件与「Claude Code 会话里的 git」是否映射到同一执行者，还是「工具招式」从属于当前出场的执行者？  
3. 主搭档：独立门面，还是某个默认执行者，还是无实体的 UI 叙述者？  
4. 观察级 runtime（只读、灵魂不长）在统一执行层模型里如何降级，而不重新引入「两种生物」？  
5. 如何一句话对外解释「为何不直接 Claude Code」——且不依赖「我们的付丧神更强」？

## Out of scope this round

- 不修改 `DESIGN.md`（项目北极星只读；结论写入任务笔记/子任务设计）
- 不定角色卡 schema、不定羁绊记账规则
- 不派实现子代理
