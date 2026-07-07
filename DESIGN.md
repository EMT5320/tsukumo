# Tsukumo — 设计稿（合并终版）

> 状态：设计收束，待开工。本文是 tsukumo 项目的唯一事实来源。
> 日期：2026-07-07
> 合并来源：`archive/DESIGN-A.md`（付丧神叙事主线）+ `archive/DESIGN-B.md`（事件驱动内核路线）+ 当晚讨论新增共识
> 命名：Tsukumo（付丧神）— 日本神话中"器物经年吸收精华化为精灵"的概念，是工具拟人化的文化根基。

---

## 1. 项目定位

一个**越用越有灵魂的终端伙伴**——把真实工程能力与拟人化人格结合，把整个工具生态注入灵魂。

### 1.1 核心命题

**把一个 agent 工具变得有意思。**

不一定要有多强的性能，但要足够有趣。定位交叉点（市场空白观察）：
- 工具向 agent（Cursor/Cline/Claude Code）：能力强，但冷冰冰、无人格
- 陪伴向（Character.ai/Talkie）：有人格，但无真实工程能力
- **Tsukumo 占位：真实工程能力 × 二次元陪伴人格 × 像素游戏演出**

### 1.2 "有意思"的三个时间层次

| 层次 | 机制 | 验证方式 |
|---|---|---|
| **第一眼：看它干活像看戏**（spectacle） | 像素舞台演出、台词、结算画面 | 自己愿意开着剧场模式干活 |
| **第一个月：看它长大**（progression） | 技能沉淀、熟练度、羁绊、引用共同历史 | 能感到它比第一天更懂自己 |
| **长期：它成了唯一的那一个**（attachment） | 积累的记忆/技能/理解无法被新实例替代 | 会舍不得删掉它的数据目录 |

MVP 用第一层惊艳，架构为第二三层留骨架。

### 1.3 反 Clippy 原则

- **可靠地做事，有趣地表达**——随机性只许进台词和演出，禁止进任务执行
- **主动性尊重心流**——想说话可以，别打断主人手上的活
- **有用是有趣的地基**——agent 连日常小任务都办不利索时，"有趣"会塌缩成"尬"

### 1.4 付丧神叙事

每个工具（git/shell/filesystem/...）都是一个"成精的器物"，有持久人格、世界观适配外观、台词库、招式集、与主人的羁绊值。叙事在最深处与产品机制同构：付丧神既是皮肤，也是"越用越有灵魂"这个核心机制的神话表达。

---

## 2. 核心概念

### 2.1 付丧神（Tsukumo）

工具拟人化的叙事根基。每个工具都是一个"成精的器物"，有：
- 持久人格（soul）
- 世界观适配外观/称谓（guise）
- 台词库、招式集
- 与主人的羁绊值

### 2.2 任务即场景演绎

不是"调用工具打印日志"，而是工具角色在像素舞台上**演出**执行过程：
- 角色出场/退场有仪式感（走位 + 标志性台词）
- 角色对操作结果有情绪反应（成功得意/失败慌张/冲突吐槽）
- 多角色协作时有站位、互动、对话

### 2.3 演绎引擎 + 内容包分离

Tsukumo 本体是**舞台引擎**，世界观/角色卡/招式都是可加载的内容包：
- 世界观包（world theme）：奇幻 RPG / 都市二次元 / 东方风 ... 可切换
- 角色卡（spirit card）：绑定工具 + 人格 + 台词 + 世界观适配
- 招式（skill）：兼容 agentskills.io 标准，角色可装备

### 2.4 双模式并存（架构原则）

```
任务模式（前台）：命令式 —— 可靠、直接、主人说了算
                  ↕ 同一批角色、同一套记忆
闲时模式（后台）：动机驱动 —— 需求驱动、行为涌现、生活感
```

- **MVP 只做任务模式**，闲时生态 schema 预留 `motivation_state` 字段，post-MVP 接入
- 闲时生态借鉴 Loomstead 的 NPC 动机引擎设计资产（见 §14）
- "可靠地做事"用命令式，"有趣地活着"用动机式——反 Clippy 原则的架构落地

---

## 3. 差异化亮点

1. **TUI 像素游戏级演出** — 终端里有一座像素风工房，付丧神小人在里面走动、干活、冒气泡，干活即演出
2. **角色即工具的持久人格** — 有记忆、成长、羁绊，越用越有温度
3. **自沉淀记忆/skill** — 从使用中沉淀经验，越用越懂主人、越懂怎么演好场景
4. **演绎引擎 + 内容包** — 引擎与内容解耦，多世界观可切换，社区可贡献角色卡
5. **冒险回放** — 事件流 append-only 日志 = 免费的"重温冒险"功能

---

## 4. 系统架构

### 4.1 四层解耦

借鉴 pi 的事件驱动极简内核 + hermes 的成长闭环 + 像素游戏演出，三块拼图：

```
┌────────────────────────────────────────────────┐
│ L4 剧场层 theater/   像素舞台演出（纯事件消费者）  │
│ L3 世界层 world/     任务系统·好感度·结算·随机事件  │
│ L2 成长层 growth/    记忆·技能·Curator（Hermes 魂）│
│ L1 内核层 kernel/    agent loop·工具·事件流（pi 魂）│
│ L0 基础层            LLM Provider 抽象·TUI 框架    │
└────────────────────────────────────────────────┘
```

### 4.2 双事件流与导演器

内核事件与游戏事件分层，避免内核被游戏逻辑污染：

```
KernelEvent (pi 式)          WorldEvent (游戏)
tool_execution_end ──┐      affinity_changed
agent_end ───────────┼──→ [世界规则引擎 reducer] ──→ quest_completed / rank_awarded
error ───────────────┘      random_event / level_up
                                   │
                    [导演器 Director] ←── 角色台词库 × 当前情绪
                                   │
                              StageEvent → 像素舞台渲染 + 日志区
```

- **世界状态 = reducer(事件日志)**：好感度、等级全部事件溯源，可重放可审计
- **导演器为纯函数**：`direct(event, worldState, lineBook) → StageEvent`，可单测、可回放
- **台词三级策略**：模板库（角色×事件×情绪索引，零成本）为主 → 关键节点 LLM 润色 → 结算总结用完整 LLM
- **角色包 = pi extension 的拟人化重构**：`card.yaml（人设+数值） + tools/（Tool 实现） + lines/（台词库） + art/（像素 sprite）`
- **单 loop 多皮起步**：初版单 agent loop，角色是工具的"皮"；架构上队长也是角色模块，未来 sub-agent = 真正独立行动的队员

### 4.3 消息双轨制（演出不污染上下文）

`AgentMessage = LlmMessage | UiMessage`，经 `convertToLlm()` 过滤后 LLM 只见标准消息。**结算卡、随机事件、角色寒暄进 session 日志但不进 LLM 上下文**——不烧 token、不干扰任务。

### 4.4 Hook 即演出节点

`beforeToolCall / afterToolCall` 两个钩子是游戏化拦截点：
- 危险命令 → `beforeToolCall` block → 角色犹豫台词 + 确认门——**安全确认门变成角色演出**
- 失败重试 → `afterToolCall` → 受挫台词 + 换策略宣言

### 4.5 headless 内核 + 多运行模式

内核不感知前端，像素舞台只是默认消费者。未来升级 Live2D 桌宠 = 事件流的另一个消费者，零重写。

### 4.6 Cargo workspace 布局（草案）

```
tsukumo/
├── crates/
│   ├── tsukumo-kernel/    # L1: loop, events, tools, session, provider
│   ├── tsukumo-growth/    # L2: memory, skills, curator
│   ├── tsukumo-world/     # L3: reducer, quest, progression, random events
│   └── tsukumo-theater/   # L4: director, stage rendering, sprites, animation
├── characters/            # 角色包: <name>/{card.yaml, tools.rs, lines.yaml, art/}
├── assets/                # 像素美术资产 (PNG source + ANSI processed)
├── worlds/                # 世界观包
└── src/bin/tsukumo.rs     # 入口: interactive / print / replay 三种模式
```

边界纪律：kernel 禁止 import 上层、theater 只消费事件。

---

## 5. 像素舞台（tsukumo-theater）★核心差异化

### 5.1 分屏混合布局

```
╔═════════════ 像素舞台 · 可折叠 ═══════════════╗
║                                              ║
║   ▄▄██▄▄     ▄██▄        ░░ 工房·像素场景 ░░  ║
║   (Gina)    (Term)   ← 小人走动/干活/冒气泡    ║
║  档案架前    磨剑中                            ║
╠══════════════════════════════════════════════╣
║ [Gina] 哼，commit 好了！写点像样的 message 啊  ║ ← 台词落日志，可回滚
║ [系统] 委托 #42 完成 · 评级 A · 羁绊 +2        ║
║ ──────────────────────────────────────────── ║
║ > 主人的指示: _                                ║
╚══════════════════════════════════════════════╝
```

- **上半部**：像素游戏舞台（Canvas + HalfBlock 渲染，20~60fps 动画）
- **下半部**：对话/日志滚动区（Paragraph + List widget，低频更新）
- **舞台可折叠**：展开=剧场 / 折叠=精简 / 关闭=静默，演出密度三档自然涌现
- **舞台和日志区消费同一条 StageEvent 流**：气泡在舞台上飘 3 秒消失，同一句台词永久落进日志区

### 5.2 终端像素渲染技法

| 技法 | 效果 | 用途 |
|---|---|---|
| HalfBlock `▀▄` | 1 字符 = 2 竖直像素，前景/背景双色 | **像素画主力** |
| Braille `⣿` | 1 字符 = 2×4 点阵，单色 | 粒子/轨迹/线条 |
| 四分块 `▖▚▛` | 1 字符 = 2×2 亚像素 | 高分辨率细节 |
| 24-bit 真彩 | 全色 | 现代终端普遍支持 |
| DEC 2026 同步输出 | 防撕裂 | SSH/tmux 下必需 |

降级策略：Kitty 图形协议（WezTerm/Kitty）→ Sixel（Windows Terminal 1.22+）→ 纯 HalfBlock（通用）。

### 5.3 游戏循环

```rust
// tokio::select! 分频架构
tokio::select! {
    _ = logic_tick.tick() => { game_state.update(TICK_DELTA); }    // 20Hz 逻辑
    _ = render_tick.tick() => { terminal.draw(|f| render(f))?; }   // 60Hz 渲染（实测定档）
    Some(ev) = event_stream.next() => { handle_input(ev); }        // 异步输入
}
```

- stdout + BufWriter + DEC 2026 同步输出
- 差异渲染：多数时间画面近乎静态（小人 idle 呼吸），全 cell 变化的最坏情况在本场景不存在

### 5.4 Actor 状态机（不引入 ECS）

tsukumo 舞台本质是"演出播放器"，不是真游戏——没有玩家实时操控、没有物理/碰撞/战斗。所需全部行为：

```
Actor 状态: Idle → Walking → Working → Talking → ...
+ 补间移动 + 帧动画 + 气泡
```

不引入 bevy_ratatui（案例少、相机转文本性能未知），纯 Ratatui Canvas 起步。

### 5.5 资产管线

```
Aseprite 像素艺术
    ↓ 导出 PNG 序列
chafa 批量转换
    ├─ 主路线：HalfBlock ANSI（--symbols=block --colors=24bit）
    └─ 彩蛋路线：Kitty/Sixel（检测终端能力自动升级）
    ↓
运行时加载 .ansi → SpriteFrame → 动画播放
```

复用 Loomstead 美术经验（见 §14.2）：风格关键词、色板、Prompt 模板、命名/manifest 组织。终端适配调整：强调高对比色块、粗轮廓、无渐变无 dithering、blocky shapes。

---

## 6. 角色系统（spirit card）

### 6.1 角色卡 schema 雏形

```yaml
name: Gina            # 角色名
tool: git             # 绑定的工具
soul:                 # 持久人格（跨世界观）
  archetype: 傲娇大小姐
  core_prompt: |
    你是 Gina，git 拟人化的大小姐...
  voice: 傲娇、嘴硬心软
guise:                # 世界观适配外观（随 world theme 切换）
  fantasy_rpg:
    title: 版本管理之魔女
    appearance: 紫袍法师少女
    call_sign: 本小姐
lines:                # 台词库（按场景/情绪分类）
  on_commit_success: ["哼，又是本小姐替你收尾！"]
  on_merge_conflict: ["呜...这种程度的冲突，本小姐才不会慌呢！"]
  on_failure: ["才、才不是本小姐的错！是你写的问题！"]
skills: [git_commit, git_branch]  # 装备的招式
bond: 0               # 羁绊初值
motivation_state: idle # MVP 恒为 idle，post-MVP 接入动机引擎
```

### 6.2 soul / guise 两层设计（倾向方案，待定）

- `soul`：持久人格，跨世界观不变，承载羁绊和长期记忆
- `guise`：随世界观切换的外观/称谓，承载沉浸感
- 叙事包装：同一灵魂在不同世界"转生"，皮相随世界变

> ⚠️ 待拍板。备选：每世界观独立角色（简单但陪伴断层）。

### 6.3 MVP 内置角色

任务域定为**编码向**，角色阵容对应编码工具链：

| 角色 | 工具 | 人格原型 | 台词风格 |
|---|---|---|---|
| Gina | git | 傲娇大小姐 | 嘴硬心软 |
| Term | shell | 沉默寡言剑士 | 简短可靠 |
| Fio | filesystem | 温柔管家 | 碎碎念关心 |
| 贤者 | 记忆检索 | （待定） | 翻阅编年史 |
| 主搭档 | 调度+陪伴 | （待定） | 日常陪伴主体 |

> 主搭档人设待定，是 90% 对话面向的角色，需专门讨论。

---

## 7. 记忆系统（自沉淀闭环）

借鉴 hermes-agent 的闭环学习思想：

```
┌─ episodic 场景存档 ─┐  每次任务=一场戏，存：出场角色/动作/结果/台词
│                     │
│  periodic nudge     │──→ 提炼 ──→ procedural 招式（"这类任务该这么演"）
│  （角色内省时刻）    │──→ 提炼 ──→ user model（"主人偏好这样"）
│                     │──→ 更新 ──→ bond 羁绊值（这次合作愉快 +xx）
└─────────────────────┘
        │
        ▼
  self-evolution 角色修行（GEPA，MVP 留接口）
```

### 7.1 记忆类型

| 类型 | 内容 | 对应 hermes |
|---|---|---|
| episodic | 场景存档（每场戏的完整记录） | session history |
| procedural | 招式/skill（程序性记忆） | skills |
| user model | 主人画像（偏好/习惯） | Honcho user modeling |
| bond | 羁绊值（每角色独立） | ★独创 |

### 7.2 Hermes 机制吸收表

| Hermes 机制 | 直接照搬的细节 | RPG 化呈现 |
|---|---|---|
| `MEMORY.md`/`USER.md` 冻结快照注入 | 容量表头、substring 式 replace/remove、保 prompt cache | 「公会档案」+「会长画像」 |
| 会话记忆 SQLite FTS 跨会话检索 | `session_search` 工具 | 贤者翻阅「编年史」 |
| Skills 渐进披露三层 | L0 目录(~3k tok) → L1 全文 → L2 附件 | 技能书：书脊 → 翻开 → 附录 |
| `SKILL.md` 格式 | `When to Use / Procedure / Pitfalls / Verification` 四段 | 「战术手册」，Pitfalls = 前辈的血泪 |
| agent 自建技能 `skill_create` | 任务后主动询问"要沉淀吗" | 结算画面：「新技能领悟！」 |
| Curator 定期整理 | `interval_messages / interval_minutes` 触发 | 随机事件：「今天是档案整理日」 |

### 7.3 养成数值真实性原则

Hermes 学习闭环（经验→记忆→技能）与 RPG 养成（冒险→经验→技能解锁）**同构**：
- 技能熟练度 Lv = 真实的 skill 使用次数 / 成功率
- 羁绊等级 = 真实的 USER.md 丰富程度
- 角色升级 = 该角色（工具）的实际使用统计

**养成数值全部来自真实的自改进数据，不是假进度条**。但需用前快后慢的映射曲线（对数/里程碑解锁）重塑节奏感，避免线性慢增长导致主人感知不到成长。

### 7.4 沉淀触发

- 任务结束时自动提炼
- 定时 nudge（角色"内省时刻"）
- 主人手动 `/reminisce`

### 7.5 回忆检索

sqlite + FTS5，支持"主人上次遇到类似问题是 Term 帮忙的"这类跨会话召回。

### 7.6 self-evolution（角色修行）

MVP 只留 trait 接口，后期接入 GEPA：
- 读执行轨迹 → 针对性优化角色台词/prompt
- guardrails：测试通过 / size 限制 / 语义不漂移 / 人工 review

---

## 8. 招式 / skill 系统

- 载体：**SKILL.md 式**（markdown，人可读可改，兼容 agentskills.io 标准）
- 调用：`/招式名` 或角色自主选用
- 角色"装备"招式，不同角色可有不同招式集
- 招式可被 self-evolution 优化

---

## 9. 世界观包（world theme）

多世界观可切换。每个世界观包定义：
- 场景渲染风格（像素工房布局 / tilemap / 调色）
- 角色 guise 适配表
- 任务包装规则（bug fix = 讨伐恶魔 / 委托 ...）
- 称谓/术语表

**MVP 写死冒险者公会**（奇幻 RPG 的具体化），guise 字段留在 schema 里但只填一份，不为多世界观提前抽象引擎。

---

## 10. 技术选型（已定）

| 维度 | 选型 | 理由 |
|---|---|---|
| 语言 | **Rust** | 像素游戏级演出需要 <1ms 重渲染，Rust 零分配 |
| TUI | **Ratatui + crossterm** | 即时模式渲染无闪烁，Canvas HalfBlock 像素画主力，tachyonfx 特效 |
| 游戏引擎 | **不引入 bevy_ratatui**，纯 Canvas + Actor 状态机 | 舞台是演出播放器不是真游戏，ECS 过度设计 |
| LLM Provider | **抽象层，多 provider 可切换**（rig/genai） | 不锁 provider，MVP 先接 Claude |
| 存储 | **sqlite + FTS5** | 跨会话检索 |
| session | **append-only JSONL**（id/parentId 预留分支） | 事件溯源，可重放 |
| skill 载体 | **SKILL.md**（兼容 agentskills.io） | 人可读可改 |
| 分发 | cargo install / 预编译二进制 | 开发者友好 |

> 选型依据：两轮专项调研（Ink 闪烁/OOM 实锤 + TS 侧 60fps 无余量出局 + movy 先例验证终端像素游戏可行性）。详见调研记录。

---

## 11. 内核核心类型草案

```rust
// kernel/events.ts —— 事件即协议，一切外层围绕它构建
enum KernelEvent {
    QuestStart { quest_id: String, goal: String },
    TurnStart { turn: u32 },
    TextDelta { delta: String },                          // 流式输出
    ToolStart { call_id: String, tool: String, args: serde_json::Value },
    ToolEnd { call_id: String, result: ToolResult, is_error: bool },
    TurnEnd { message: AssistantMessage },
    QuestEnd { stats: QuestStats },                       // 供结算
    Error { error: AppError, recoverable: bool },
}

// kernel/tool.rs
trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn params(&self) -> &serde_json::Value;  // JSON Schema
    async fn execute(&self, args: serde_json::Value, ctx: &ToolContext) -> ToolResult;
}

// characters/types.rs —— pi extension 的拟人化重构
struct CharacterPack {
    card: Card,                    // 人设 + 数值
    tools: Vec<Box<dyn Tool>>,     // 该角色承载的能力
    lines: LineBook,               // 事件×情绪 → 台词模板（带权重随机）
    art: HashMap<Mood, Sprite>,    // 像素 sprite（ASCII/ANSI 字符画）
}
```

loop 纪律：本体目标 **<300 行**——`while: 调 LLM → 流事件 → 收集 tool calls → (hook) → 执行 → 结果入上下文`，只留 `beforeToolCall / afterToolCall` 两个 hook 位。

---

## 12. 里程碑（双线并行）

| 线 | 里程碑 | 内容 | 验证什么 |
|---|---|---|---|
| 舞台线 | S0 | 像素工房静态场景渲染（HalfBlock） | 画面观感成立？ |
| 舞台线 | S1 | 小人行走动画 + 气泡 + 分屏布局 + 折叠 | 演出感成立？帧率稳？ |
| 内核线 | K0 | agent loop + 2 工具 + JSONL session（print 模式）| 无聊但正确 |
| 内核线 | K1 | KernelEvent 流完整化 + 导演器纯函数 | 事件协议够用？ |
| 汇合 | M1 | S1 + K1 对接：真实任务驱动小人演出 | **"看 agent 干活像看戏"成立？** |
| 成长 | M2 | MEMORY.md/USER.md 冻结快照 + SKILL.md 渐进披露 + skill_create | 成长闭环 |
| 世界 | M3 | 好感度/熟练度 reducer、随机事件、Curator | 养成可视化 |
| 打磨 | M4 | 回放模式、cargo 发布、README | — |

两线在事件协议处握手——KernelEvent/StageEvent 的 schema 是唯一的共享契约，先定协议再分头开工。

### S0 验收清单

- [ ] Windows Terminal 上实测：工房场景 + 1 个走动小人的真实帧率与 CPU 占用
- [ ] 中文气泡的宽字符对齐正确（unicode-width）
- [ ] 分屏布局：上舞台下日志，舞台可折叠
- [ ] chafa 管线跑通：一张 PNG 像素图 → .ansi → 屏幕上渲染出来
- [ ] tmux / conhost 下的降级不崩溃

---

## 13. MVP 范围与边界

### MVP 做
- 3~4 个付丧神角色 + 主搭档
- 1 个世界观（冒险者公会 / 奇幻 RPG）
- 像素舞台分屏演出（S0+S1）
- 角色卡可加载（yaml）
- 基础场景演绎（走位/台词/情绪反应/气泡）
- episodic 记忆 + 简单沉淀（nudge + auto-skill 雏形）
- 羁绊值系统
- 统一 LLM 抽象（先接 Claude）
- 真实工具执行（git/shell/filesystem）
- 任务模式（命令式可靠执行）

### MVP 不做（留接口）
- 闲时生态（动机驱动）—— schema 预留 motivation_state
- self-evolution（GEPA）—— 只留 trait
- 多世界观切换 —— 架构预留，只实现 1 个
- GUI/Live2D —— 像素 TUI 占位
- 语音/TTS
- 跨平台 messaging gateway

---

## 14. Loomstead 搭桥

### 14.1 搭桥原则

搬**设计资产**（schema/机制/经验教训），不搬代码（Loomstead 是 Python，tsukumo 是 Rust）。两项目任务型 agent loop 结构差异大（生活模拟周期 vs ReAct 流式循环），真正可平移的是闲时生态/记忆存储/美术管线。

### 14.2 可复用资产清单

| Loomstead 资产 | tsukumo 位置 | 价值 |
|---|---|---|
| NPC 闲时行为环（需求累积→规则选行为） | 工房闲时生态（post-MVP） | 已验证纯规则零 LLM 成本产生生活感 |
| 三层工具路由（生理=规则/职业=规则+LLM/社交=LLM受预算） | 台词三级策略同构 + LLM 升级触发条件 | token 预算纪律现成答案 |
| `failure_modes` 带 `emotional_charge` 的工具 schema | 失败演出化的数据基础 | 直接抄 |
| 双轨主观记忆（客观事件流 + 每 NPC 主观视图） | 每个付丧神对同一事件有不同记忆 | 角色深度来源 |
| Interrupt 机制（interruptible + 优先级阈值） | 小人被打断跑向岗位 | 工房"活着"的关键细节 |
| Director 间接干预 | 世界层随机事件与闲时编排 | post-MVP 深度方向 |
| trace 可观测性 | "Gina 为什么说这句台词"可追溯 | 演出系统调试器 |
| 美术管线（AI 生成像素资产 prompt、manifest 组织） | 像素小人资产管线 | 风格关键词/Prompt 模板/命名规范直接复用 |

### 14.3 美术管线适配

Loomstead 经验直接复用：风格定位（二次元轻幻想）、色板（3-5 主色）、头身比（2.0-2.5）、Prompt 模板结构、命名/manifest 组织。

终端字符像素适配调整：
- 轮廓线加粗（1-3 px），强调高对比色块
- 减少细节到 2-3 个最关键识别点
- 无渐变、无 dithering、无细微纹理
- Prompt 新增：`high contrast, distinct color blocks, terminal-friendly, blocky shapes, geometric silhouette, no gradients, no dithering`
- 验证流程：PNG 64x64 → chafa 转字符画 → 不同终端宽度下验证可读性

---

## 15. 求职叙事占位（待 MVP 验证）

tsukumo 在求职画像中的潜在定位：**方法论的产品闭环**——补上"把评测结果转化为上线取舍"这半句。

现有项目证明"我能用评测定位四个设计变量的能力边界"：
- 模型 → AlgoCoach
- 上下文协议 → ContextGuard
- Agent harness → Loomstead
- 推理配置 → Inference Lab

tsukumo 潜在价值：证明"然后把能力边界认知做成了真实可用的产品"。

> ⚠️ 此定位待 MVP 出成果后再定。不让求职目标在 MVP 阶段绑架设计决策。

---

## 16. 开放问题

- [ ] 主搭档人格设定（90% 对话面向的角色，需专门讨论）
- [ ] soul/guise 两层设计是否最终采用（倾向是，待拍板）
- [ ] 角色阵容细节与命名
- [ ] 演绎规则细节（多角色协作怎么站位、对话怎么编排）
- [ ] 记忆沉淀的具体触发阈值与频率
- [ ] 角色卡 schema 细化（字段完备性）
- [ ] 世界观包 schema
- [ ] Cargo workspace 详细布局
- [ ] 测试策略
- [ ] "有意思"的可评测定义（待 MVP 验证）
- [ ] 演出节奏 vs 效率的快进模式细节
- [ ] 情绪状态机的具体变量与衰减曲线
- [ ] 主动性/驻留模式的触发规则与打扰边界（post-MVP）
- [ ] 是否 git init + 仓库托管

---

## 17. 参考项目与吸收要点

### pi（badlogic/pi-mono）— 内核骨架
- 分层 monorepo：`pi-ai`（零 agent 依赖）→ `pi-agent-core`（事件驱动 loop、steering、并行工具）→ 应用层
- 类型化事件流贯穿每一层；`AgentMessage` 自定义类型 + `convertToLlm` 过滤
- `beforeToolCall / afterToolCall` 钩子；session JSONL 树结构（id/parentId 分支）
- 同一 AgentSession 支撑 interactive / print / RPC / SDK 四种模式

### hermes-agent（Nous Research）— 成长闭环
- 三层记忆：会话（SQLite FTS 跨会话检索）/ 持久（MEMORY.md + USER.md 冻结快照注入、容量表头）/ 技能（SKILL.md）
- Skills 渐进披露三层加载省 token；frontmatter 条件激活
- agent 经 `skill_create / skill_edit` 自建技能，任务后主动建议沉淀
- Curator 定期整理：合并冗余、清理矛盾、把重复流程建议为技能

### hermes-agent-self-evolution — 角色修行
- DSPy + GEPA 读执行轨迹 → 针对性优化 skill/prompt/tool
- guardrails：测试 / size / 语义不漂移 / PR review

### Loomstead — 设计资产搭桥
- NPC 动机引擎、主观记忆、failure_modes 情绪电荷、美术管线（见 §14）

### 抽象 → 叙事 转译表

| hermes 抽象 | tsukumo 拟人化转译 | 叙事含义 |
|---|---|---|
| skill（程序性记忆） | 角色招式/技能 | 角色学会的动作套路，可成长 |
| memory curation | 角色回忆整理 | 付丧神"沉淀修为" |
| user modeling | 羁绊系统 | 角色越来越懂主人 |
| personality | 付丧神人格卡 | 每个工具一个角色 |
| self-evolution (GEPA) | 角色修行 | 台词/演出自动优化，越演越好 |
| periodic nudge | 角色内省时刻 | 定期"打坐回忆"，提炼经验 |
