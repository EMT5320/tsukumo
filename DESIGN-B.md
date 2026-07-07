# Tsukumo — 设计稿 B（事件驱动内核路线）

> 状态：讨论沉淀，未开工。本文仅记录 2026-07-07 一轮讨论的结论，与 `DESIGN.md`（另一轮独立讨论）互不影响，供后续综合抉择。
> 核心命题：**把一个 agent 工具变得有意思**。

---

## 1. 定位与判断

### 1.1 市场空白
- 工具向 agent（coding agent / workflow 自动化）已饱和红海
- 陪伴向产品（Character.AI 类）有人格但无真实 agent 能力
- 空白点：**既有人格魅力、又能真正做事**的结合体；游戏机制（好感度/养成/随机事件）把"用助手"变成"玩助手"

### 1.2 形态结论：CLI/TUI 起步
选择理由：
- 开发最快，能把精力集中在人格/记忆/agent 核心逻辑上
- 终端同样有表现力：彩色对话、ASCII 立绘按情绪切换、打字机逐字输出、常驻状态栏（心情/好感度/疲劳度）
- 内核保持 headless，未来桌宠/GUI 只是接入同一事件流的另一个前端（见 4.4）

### 1.3 陪伴感要点（早期头脑风暴保留项）
- **情绪状态机**：心情/好感度/疲劳是真实状态变量，事件驱动更新，注入 system prompt 影响回复风格（好感低→傲娇，疲劳→犯困）
- **主动性**：驻留模式下定时触发事件（久坐提醒/整点报时/随机闲聊），不做纯被动应答
- **记忆回溯**：每次启动加载"关于主人的档案"，跨会话连续性
- TTS 语音（GPT-SoVITS 等）为后期彩蛋，不进 MVP

---

## 2. 世界观隐喻：冒险者公会

**任务即冒险，工具拟人为队员，agent 执行过程本身变成可观赏的游戏场景。**

### 2.1 概念映射

| Agent 概念 | RPG 映射 |
|---|---|
| 用户 | 公会会长 / 委托人 |
| 主 Agent (orchestrator) | 队长（理解委托、指挥队伍） |
| 一次任务 | 一次「委托 / Quest」 |
| 工具调用 | 队员执行动作（侦察、翻阅、锻造…） |
| 工具返回结果 | 动作战报 |
| 错误 / 重试 | 队员受挫、换策略再上 |
| 任务完成 | 结算画面（经验、掉落、评级 S/A/B） |

### 2.2 工具拟人示例（占位命名，角色设定待细化）

- **斥候**（web 搜索/抓取）："我去打探消息！" → 返回情报卷轴
- **书记官**（文件读写）：翻阅典籍、抄录文书
- **铁匠/战士**（shell 执行）："交给我来砸！" → 失败时"武器崩了一个口"
- **贤者**（记忆/知识库）：从档案馆检索往事
- **吟游诗人**（总结/汇报）：任务结束后吟唱战报

### 2.3 TUI 场景草图

```
╔══ 委托 #42：调查「本周 AI 新闻」════════ 好感 Lv.3 ══╗
║  冒险中...                                          ║
║  [斥候·琳] 掠出阴影："情报到手了喵！" (search 完成)    ║
║  [贤者·梅] 翻开古籍... (检索记忆中)                   ║
║  [队长] "很好，接下来整理成报告！"                    ║
╠════════════════════════════════════════════════════╣
║  > 会长的指示: _                                     ║
╚════════════════════════════════════════════════════╝
```

---

## 3. 核心架构论点：三层解耦

pi 证明事件驱动的极简内核可以支撑任意复杂外层；Hermes 证明记忆→技能闭环让 agent 越用越强；RPG 化是第三块拼图：**学习闭环本身就是养成系统，事件流本身就是剧本**。

```
┌────────────────────────────────────────────────┐
│ L4 剧场层 theater/   TUI 演出（纯事件消费者）      │
│ L3 世界层 world/     任务系统·好感度·结算·随机事件  │
│ L2 成长层 growth/    记忆·技能·Curator（Hermes 魂）│
│ L1 内核层 kernel/    agent loop·工具·事件流（pi 魂）│
│ L0 基础层            LLM Provider 抽象·TUI 框架    │
└────────────────────────────────────────────────┘
```

---

## 4. 从 pi 吸收的四个关键设计

### 4.1 事件流即剧本（最重要）
loop 在每个环节发射类型化事件（`agent_start / turn_start / message_update / tool_execution_start / tool_execution_end / turn_end / agent_end / error`）。事件流天然就是舞台指令：
- `tool_execution_start(search)` → 「斥候跃出阴影」
- `message_update` → 队长说话的打字机流
- `tool_execution_end(isError)` → 「铁匠的锤子脱手了！」

推论：**演出层不侵入内核一行代码**，只订阅事件。session 为 append-only JSONL → 重放事件日志 = 免费获得「冒险回放」功能。

### 4.2 消息双轨制（演出不污染上下文）
`AgentMessage = LlmMessage | UiMessage`，经 `convertToLlm()` 过滤后 LLM 只见标准消息。**结算卡、随机事件、角色寒暄进 session 日志但不进 LLM 上下文**——不烧 token、不干扰任务。

### 4.3 Hook 即演出节点
`beforeToolCall / afterToolCall` 两个钩子是游戏化拦截点：
- 危险命令 → `beforeToolCall` block → 「铁匠犹豫："会长，这一锤下去可收不回来！"（y/n）」——**安全确认门变成角色演出**
- 失败重试 → `afterToolCall` → 受挫台词 + 换策略宣言

### 4.4 headless 内核 + 多运行模式
参考 pi 的 interactive / print / RPC / SDK 四模式共用同一 core：Tsukumo 内核不感知前端，TUI 只是默认消费者。未来升级 Live2D 桌宠 = RPC 事件流的另一个消费者，零重写。

---

## 5. 从 Hermes 吸收的成长闭环（游戏化同构）

| Hermes 机制 | 直接照搬的细节 | RPG 化呈现 |
|---|---|---|
| `MEMORY.md`/`USER.md` 冻结快照注入 | 容量表头 `[67% — 1474/2200]` 让 agent 自我管理；substring 式 replace/remove；改动落盘但当次快照不变（保 prompt cache） | 「公会档案」+「会长画像」 |
| 会话记忆 SQLite FTS 跨会话检索 | `session_search` 工具 | 贤者翻阅「编年史」 |
| Skills 渐进披露三层 | L0 目录(~3k tok) → L1 全文 → L2 附件 | 技能书：书脊 → 翻开 → 附录 |
| `SKILL.md` 格式 | `When to Use / Procedure / Pitfalls / Verification` 四段；agentskills.io 兼容 | 「战术手册」，Pitfalls = 前辈的血泪 |
| agent 自建技能 `skill_create` | 任务后主动询问"要沉淀吗" | 结算画面：「新技能领悟！」 |
| Curator 定期整理 | `interval_messages / interval_minutes` 触发 | 随机事件：「今天是档案整理日」 |

### 5.1 同构洞察与养成数值真实性原则（本设计核心）
Hermes 学习闭环（经验→记忆→技能）与 RPG 养成（冒险→经验→技能解锁）**同构**：
- 技能熟练度 Lv = 真实的 skill 使用次数 / 成功率
- 羁绊等级 = 真实的 USER.md 丰富程度
- 角色升级 = 该角色（工具）的实际使用统计

**养成数值全部来自真实的自改进数据，不是假进度条**——游戏化不是贴皮，是把已验证的机制可视化。

---

## 6. 双事件流与导演器

内核事件与游戏事件分层，避免内核被游戏逻辑污染：

```
KernelEvent (pi 式)          WorldEvent (游戏)
tool_execution_end ──┐      affinity_changed
agent_end ───────────┼──→ [世界规则引擎 reducer] ──→ quest_completed / rank_awarded
error ───────────────┘      random_event / level_up
                                   │
                    [导演器 Director] ←── 角色台词库 × 当前情绪
                                   │
                              StageEvent → TUI 渲染
```

- **世界状态 = reducer(事件日志)**：好感度、等级全部事件溯源，可重放可审计
- **导演器为纯函数**：`direct(event, worldState, lineBook) → StageEvent`，可单测、可回放
- **台词三级策略**：模板库（角色×事件×情绪索引，零成本）为主 → 关键节点 LLM 润色 → 结算总结用完整 LLM
- **角色包 = pi extension 的拟人化重构**：`card.yaml（人设+数值） + tools/（Tool 实现） + lines/（台词库） + art/（ASCII 表情）`，招募新队员 = 装一个角色包
- **单 loop 多皮起步**：初版单 agent loop，角色是工具的"皮"；架构上队长也是角色模块，未来 sub-agent = 真正独立行动的队员，事件流天然支持多角色同时演出

---

## 7. 技术选型（本轮结论）

内核路线三选一的记录：
- A. 基于 `pi-agent-core`（TS）：内核白拿，但受 pi 抽象约束
- B. Python 自研 + Rich/Textual：生态丰富，但 loop/流式/provider 全自写
- **B'. TS 自研极简内核（选定）**：不依赖 pi 但可随时参考其源码，保留 Ink 生态与 npm 分发

选型清单：
- 语言：**TypeScript**（Node >= 20）
- TUI：**Ink**
- Provider 层：**Vercel AI SDK 仅作 provider 抽象**（每 turn 调一次 `streamText`，loop 主权在自己手里）——自研价值在 loop/事件协议/成长闭环，不在 SSE 解析器，这是唯一建议借力的地方
- 工具参数校验：**zod**（校验 + 自动生成 JSON Schema）
- 存储：session 为 **append-only JSONL**（id/parentId 预留分支）；跨会话检索 **SQLite + FTS5**
- 分发：npm

---

## 8. 仓库形态与模块边界

单包起步（不过早 monorepo），等 RPC/桌宠前端真出现再拆包：

```
src/
├── kernel/        # L1: loop, events, tools, session, provider — 禁止 import 上层
│   ├── loop.ts          # agent loop（状态机）
│   ├── events.ts        # KernelEvent 类型定义
│   ├── tool.ts          # Tool 接口 + registry
│   ├── session.ts       # JSONL append-only 读写/回放
│   └── providers/       # LLM 流式抽象
├── growth/        # L2: memory.ts, skills.ts, curator.ts
├── world/         # L3: reducer.ts(事件溯源), quest.ts, progression.ts, events.ts(随机事件)
├── characters/    # 角色包: <name>/{card.yaml, tools.ts, lines.yaml, art.txt}
├── theater/       # L4: director.ts, stage/(Ink 组件), typewriter.ts
└── cli.tsx        # 入口：interactive / print / replay 三种模式
```

边界纪律：eslint `import/no-restricted-paths` 强制 `kernel` 不认识任何上层、`theater` 只消费事件（学自 pi：`pi-ai` 对 agent 逻辑零依赖）。

---

## 9. 内核核心类型草案

```typescript
// kernel/events.ts —— 事件即协议，一切外层围绕它构建
type KernelEvent =
  | { type: 'quest_start'; questId: string; goal: string }
  | { type: 'turn_start'; turn: number }
  | { type: 'text_delta'; delta: string }                          // 流式输出
  | { type: 'tool_start'; callId: string; tool: string; args: unknown }
  | { type: 'tool_end'; callId: string; result: ToolResult; isError: boolean }
  | { type: 'turn_end'; message: AssistantMessage }
  | { type: 'quest_end'; stats: QuestStats }                       // 供结算
  | { type: 'error'; error: Error; recoverable: boolean };

// kernel/tool.ts
interface Tool<P = unknown> {
  name: string;
  description: string;
  params: z.ZodType<P>;
  execute(args: P, ctx: ToolContext): Promise<ToolResult>;
}

// characters/types.ts —— pi extension 的拟人化重构
interface CharacterPack {
  card: { id: string; name: string; persona: string; role: string };
  tools: Tool[];                        // 该角色承载的能力
  lines: LineBook;                      // 事件×情绪 → 台词模板（带权重随机）
  art: Record<Mood, string>;            // ASCII 表情/立绘
}
```

loop 纪律：本体目标 **<300 行**——`while: 调 LLM → 流事件 → 收集 tool calls → (hook) → 执行 → 结果入上下文`，只留 `beforeToolCall / afterToolCall` 两个 hook 位，其他一概不加。

---

## 10. 剧场层组件树

```
<App>
├── <StatusBar/>      # 好感 Lv.3 · 委托 #42 · 队伍状态
├── <Stage/>          # 场景日志：角色动作台词流（打字机）+ ASCII 表情
├── <Settlement/>     # quest_end 触发：评级/掉落/「新技能领悟！」
└── <CommandBar/>     # 会长输入 + /命令
```

`replay` 模式直接喂历史 JSONL 给导演器，免费的"重温冒险"。

---

## 11. 里程碑（每步可运行可玩）

| 里程碑 | 内容 | 预估 |
|---|---|---|
| M0 内核 | loop + 事件流 + 2 个工具（bash/读文件）+ JSONL session，print 模式跑通——"无聊但正确"的 mini agent | ~3 天 |
| M1 剧场 | Ink TUI + 导演器 + 3 个角色包 + 委托框架/结算画面——"有意思"在这一步诞生 | ~4 天 |
| M2 成长 | MEMORY.md/USER.md 冻结快照 + 容量表头 + memory 工具；SKILL.md 渐进披露 + skill_create | ~4 天 |
| M3 世界 | 好感度/熟练度 reducer（读真实使用统计）、随机事件、Curator"整理日" | ~3 天 |
| M4 打磨 | 回放模式、npm 发布、README | — |

M0→M1 优先：核心命题是"有意思"，最快验证"看 agent 干活像看戏"这个体验是否成立。

---

## 12. 开放问题

- [ ] **演出节奏 vs 效率**：打字机/动画拖慢感知速度，需要"剧场模式/快进模式"切换
- [ ] **初版委托的任务类型定位**：信息调研类 / 本地文件与开发任务类 / 日常助理类？决定第一批招募哪些队员（本轮未拍板）
- [ ] **角色阵容与命名**：本轮用队长/斥候/书记官/铁匠/贤者占位，人设待细化
- [ ] **单 loop → 多 agent 的演进时机**
- [ ] **主动性/驻留模式**的触发规则与打扰边界
- [ ] 情绪状态机的具体变量与衰减曲线

---

## 13. 参考项目吸收要点（本轮视角）

### pi（badlogic/pi-mono）— 内核骨架
- 分层 monorepo：`pi-ai`（零 agent 依赖）→ `pi-agent-core`（事件驱动 loop、steering、并行工具）→ 应用层
- 类型化事件流贯穿每一层；`AgentMessage` 自定义类型 + `convertToLlm` 过滤
- `beforeToolCall / afterToolCall` 钩子；session JSONL 树结构（id/parentId 分支）
- 同一 AgentSession 支撑 interactive / print / RPC / SDK 四种模式

### Hermes（Nous Research）— 成长闭环
- 三层记忆：会话（SQLite FTS 跨会话检索）/ 持久（MEMORY.md + USER.md 冻结快照注入、容量表头、substring 式 memory 工具）/ 技能（SKILL.md）
- Skills 渐进披露三层加载省 token；frontmatter 支持 `platforms / requires_toolsets / fallback_for_toolsets` 条件激活
- agent 经 `skill_create / skill_edit` 自建技能，任务后主动建议沉淀
- Curator 定期整理：合并冗余、清理矛盾、把重复流程建议为技能
