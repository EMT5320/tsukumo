# Tsukumo — 设计稿（初版）

> 状态：想法雏形，未开工。本文记录收束后的初版设计意图，保留开放项与演进空间。
> 日期：2026-07-07
> 命名：Tsukumo（付丧神）— 日本神话中"器物经年吸收精华化为精灵"的概念，是工具拟人化的文化根基。

---

## 1. 项目定位

一个偏二次元/游戏风的**陪伴向 agent 助手**，把真实工程能力与拟人化人格结合。

核心立意：不是给 agent 套一层二次元皮肤，而是把**整个工具生态注入灵魂**——每个工具都是一个有记忆、有成长、有羁绊的付丧神角色，每次任务都是在 TUI 舞台上演出的一场戏。

定位交叉点（市场空白观察）：
- 工具向 agent（Cursor/Cline/Claude Code）：能力强，但冷冰冰、无人格
- 陪伴向（Character.ai/Talkie）：有人格，但无真实工程能力
- **Tsukumo 占位：真实工程能力 × 二次元陪伴人格 × 场景演绎**

---

## 2. 核心概念

### 2.1 付丧神（Tsukumo）
工具拟人化的叙事根基。每个工具（git/shell/filesystem/...）都是一个"成精的器物"，有：
- 持久人格（soul）
- 世界观适配外观/称谓（guise）
- 台词库、招式集
- 与主人的羁绊值

### 2.2 任务即场景演绎
不是"调用工具打印日志"，而是工具角色在 TUI 舞台上**演出**执行过程：
- 角色出场/退场有仪式感（立绘滑入 + 标志性台词）
- 角色对操作结果有情绪反应（成功得意/失败慌张/冲突吐槽）
- 多角色协作时有站位、互动、对话

这是核心差异化：把枯燥的工具调用变成可观看的剧情。

### 2.3 演绎引擎 + 内容包分离
Tsukumo 本体是**舞台引擎**，世界观/角色卡/招式都是可加载的内容包：
- 世界观包（world theme）：奇幻 RPG / 都市二次元 / 东方风 ... 可切换
- 角色卡（spirit card）：绑定工具 + 人格 + 台词 + 世界观适配
- 招式（skill）：兼容 agentskills.io 标准，角色可装备

用户可自定义新角色接入新工具，扩展性强。

---

## 3. 差异化亮点

1. **任务即场景演绎** — 工具调用变成可观看的剧情，不是套皮日志
2. **角色即工具的持久人格** — 有记忆、成长、羁绊，越用越有温度（区别于一次性拟人化吐槽）
3. **自沉淀记忆/skill** — 从使用中沉淀经验，越用越懂主人、越懂怎么演好场景
4. **演绎引擎 + 内容包** — 引擎与内容解耦，多世界观可切换，社区可贡献角色卡

---

## 4. 系统架构（Rust crate 分层）

借鉴 pi（`earendil-works/pi`）的分层纪律：

```
tsukumo-llm       统一多 provider LLM 抽象（借鉴 pi-ai）
                  不锁 provider，trait 抽象，后期可切 Claude/OpenAI/本地

tsukumo-runtime   agent runtime + tool dispatch + state（借鉴 pi-agent-core）
                  agent loop、工具调用、状态管理

tsukumo-spirit    付丧神角色系统 ★独创
                  ├─ 角色卡加载（soul + guise + 台词库 + 招式集）
                  ├─ 羁绊/关系状态
                  └─ 出场/退场仪式

tsukumo-memory    自沉淀记忆 + skill 系统 ★借鉴 hermes
                  ├─ episodic（场景存档）
                  ├─ procedural = 招式/skill
                  ├─ user model（主人画像）
                  ├─ bond（羁绊值）
                  └─ 沉淀引擎（nudge + auto-skill + self-evolution 接口）

tsukumo-stage     TUI 舞台演绎层 ★独创（借鉴 pi-tui 差分渲染）
                  ├─ 场景渲染 / 角色站位 / 演出动画
                  └─ Galgame 风对话框

tsukumo-cli       入口
```

---

## 5. 角色系统（spirit card）

### 5.1 角色卡 schema 雏形

```yaml
# 角色 = 付丧神，绑定一个真实工具
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
  urban_modern:
    title: 代码档案室部长
    appearance: 制服学姐
    call_sign: 学姐
lines:                # 台词库（按场景/情绪分类）
  on_commit_success: ["哼，又是本小姐替你收尾！"]
  on_merge_conflict: ["呜...这种程度的冲突，本小姐才不会慌呢！"]
  on_failure: ["才、才不是本小姐的错！是你写的问题！"]
skills: [git_commit, git_branch]  # 装备的招式
bond: 0               # 羁绊初值
```

### 5.2 soul / guise 两层设计（倾向方案，待定）

- `soul`：持久人格，跨世界观不变，承载羁绊和长期记忆
- `guise`：随世界观切换的外观/称谓，承载沉浸感

设计意图：既保陪伴连续性（换世界观不丢羁绊），又有世界观沉浸感。
叙事包装：同一灵魂在不同世界"转生"，皮相随世界变。

> ⚠️ 此为倾向方案，尚未拍板。备选：每世界观独立角色（简单但陪伴断层）。

### 5.3 MVP 内置角色（3 个 + 主搭档）

| 角色 | 工具 | 人格原型 | 台词风格 |
|---|---|---|---|
| Gina | git | 傲娇大小姐 | 嘴硬心软 |
| Term | shell | 沉默寡言剑士 | 简短可靠 |
| Fio | filesystem | 温柔管家 | 碎碎念关心 |
| 主搭档 | 调度+陪伴 | （待定） | 日常陪伴主体 |

---

## 6. 记忆系统（自沉淀闭环）

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
  读执行轨迹 → 优化角色台词/prompt → 测试 guardrail → 越演越到位
```

### 6.1 记忆类型

| 类型 | 内容 | 对应 hermes |
|---|---|---|
| episodic | 场景存档（每场戏的完整记录） | session history |
| procedural | 招式/skill（程序性记忆） | skills |
| user model | 主人画像（偏好/习惯） | Honcho user modeling |
| bond | 羁绊值（每角色独立） | ★独创 |

### 6.2 沉淀触发

- 任务结束时自动提炼
- 定时 nudge（角色"内省时刻"）
- 主人手动 `/reminisce`

### 6.3 回忆检索

sqlite + FTS5（借鉴 hermes），支持"主人上次遇到类似问题是 Term 帮忙的"这类跨会话召回。

### 6.4 self-evolution（角色修行）

MVP 只留 trait 接口，后期接入 GEPA（借鉴 `hermes-agent-self-evolution`）：
- 读执行轨迹 → 针对性优化角色台词/prompt
- guardrails：测试通过 / size 限制 / 语义不漂移 / 人工 review

---

## 7. 招式 / skill 系统

- 载体：**SKILL.md 式**（markdown，人可读可改，兼容 agentskills.io 标准）
- 调用：`/招式名` 或角色自主选用（借鉴 hermes `/<skill>`）
- 角色"装备"招式，不同角色可有不同招式集
- 招式可被 self-evolution 优化

---

## 8. 世界观包（world theme）

多世界观可切换。每个世界观包定义：
- 场景渲染风格（奇幻战场 / 都市咖啡店 / ...）
- 角色 guise 适配表
- 任务包装规则（bug fix = 讨伐恶魔 / 委托 ...）
- 称谓/术语表

MVP 先做 1 个世界观（奇幻 RPG），架构上预留切换能力。

---

## 9. TUI 舞台演绎（tsukumo-stage）

借鉴 pi-tui 差分渲染。MVP 草图：

```
╔══════════════════════════════════════════════════╗
║  [Gina 立绘]   ┌──────────────────────────────┐  ║
║   ／￣￣＼     │ Gina: 哼！本小姐帮你 commit   │  ║
║  |  ╮  ╭|     │      好了，记得写点像样的      │  ║
║  |  >  < |    │      commit message 啊！(傲娇) │  ║
║   ＼＿＿＿／   └──────────────────────────────┘  ║
║  羁绊: ███░░░ 62%   [git] [shell] [file]        ║
╠══════════════════════════════════════════════════╣
║ > _                                               ║
╚══════════════════════════════════════════════════╝
```

MVP 用 ASCII/Unicode 立绘占位，后期可接 Live2D/GUI。

---

## 10. 技术选型（已定）

- 语言：**Rust**
- TUI：**Ratatui**
- LLM：**抽象层，多 provider 可切换**（MVP 先接 Claude，充分利用主人订阅额度）
- 存储：sqlite + FTS5
- skill 载体：SKILL.md（兼容 agentskills.io）

---

## 11. MVP 范围与边界

### MVP 做
- 3 个付丧神角色（Gina/Term/Fio）+ 主搭档
- 1 个世界观（奇幻 RPG）
- 角色卡可加载（yaml）
- 基础场景演绎（出场/台词/情绪反应）
- episodic 记忆 + 简单沉淀（nudge + auto-skill 雏形）
- 羁绊值系统
- 统一 LLM 抽象（先接 Claude）
- 真实工具执行（git/shell/filesystem）

### MVP 不做（留接口）
- self-evolution（GEPA）—— 只留 trait
- 多世界观切换 —— 架构预留，只实现 1 个
- GUI/Live2D —— TUI ASCII 占位
- 语音/TTS
- 跨平台 messaging gateway

---

## 12. 待定 fork / 后续方向

- [ ] soul/guise 两层设计是否最终采用（倾向是，待拍板）
- [ ] 主搭档人格设定
- [ ] 演绎规则细节（多角色协作怎么站位、对话怎么编排）
- [ ] 记忆沉淀的具体触发阈值与频率
- [ ] 角色卡 schema 细化（字段完备性）
- [ ] 世界观包 schema
- [ ] 项目目录结构 / Cargo workspace 布局
- [ ] 是否 git init + 仓库托管
- [ ] 测试策略

---

## 13. 参考项目与吸收要点

### pi（`earendil-works/pi`）— 架构骨架
- 教科书式分层：`pi-ai` / `pi-agent-core` / `pi-coding-agent` / `pi-tui`
- 统一多 provider LLM 抽象不锁 provider
- 严格类型纪律、自扩展配置目录（`.pi`）
- TUI 差分渲染

### hermes-agent（`NousResearch/hermes-agent`）— 灵魂系统
- 闭环学习：agent 自策划记忆 + periodic nudge + auto-skill + skill 自我改进
- skills = procedural memory，兼容 agentskills.io，`/<skill>` 调用
- 跨会话回忆：FTS5 + LLM 摘要
- 用户建模：Honcho 辩证式
- personality 切换 / subagent 委派 / ContextCompressor

### hermes-agent-self-evolution — 角色修行
- DSPy + GEPA 读执行轨迹 → 针对性优化 skill/prompt/tool
- guardrails：测试 / size / 语义不漂移 / PR review

### 抽象 → 叙事 转译表

| hermes 抽象 | tsukumo 拟人化转译 | 叙事含义 |
|---|---|---|
| skill（程序性记忆） | 角色招式/技能 | 角色学会的动作套路，可成长 |
| memory curation | 角色回忆整理 | 付丧神"沉淀修为" |
| user modeling | 羁绊系统 | 角色越来越懂主人 |
| personality | 付丧神人格卡 | 每个工具一个角色 |
| self-evolution (GEPA) | 角色修行 | 台词/演出自动优化，越演越好 |
| periodic nudge | 角色内省时刻 | 定期"打坐回忆"，提炼经验 |
