# Vibe Break — 项目背景与调研记录

> 本文档记录了项目起源的完整思考过程，作为后续开发的上下文参考。

## 起源

用户观察到：AI 使用时间骤增，人们需要小工具提醒自己停下来、接触现实世界。最初的想法是在 Claude Code 中做一个插件，在一定 session 时间后提醒休息。

## 竞品调研

### 已有产品分层

**桌面端通用休息提醒（最成熟）**
- **Stretchly** — 开源，13k stars，最流行，但完全不感知用户在做什么
- **LookAway** (Mac, $15) — 能检测开会/录屏时自动延迟，较智能
- **DeskBreak** — 最接近开发者场景，能在 git commit 后建议休息

**VS Code 扩展（多但简单）**
- 番茄钟类十几个，都是简单计时器
- **CodeFit** 最全面——运动指导、游戏化、团队排行榜，但不感知 AI 使用

**CLI 工具**
- **pomo** (Go, 1.5k stars) — 终端番茄钟，可脚本化
- **Flomo** — 根据实际工作时长动态计算休息（flowmodoro），非固定 25 分钟

**Claude Code 生态**
- **Health Buddy Skill** — 检测到用户说"我要疯了"时触发关怀，非主动计时
- 没有现成的 session 计时休息提醒插件

### 关键结论

如果核心功能只是"每隔一段时间提醒休息"，pomo/flomo/Stretchly 已经能做。差异化只在"AI 感知"这一层才成立——需要知道交互轮次、交互密度、代码变更量，而非单纯计时。

## 核心洞察：六重神经机制

经过深度调研，确认 vibe coding 疲劳是六种神经机制同时作用的结果。这是产品的理论根基和差异化来源。

### 1. 多巴胺透支
- **研究者**：Anna Lembke（斯坦福，《多巴胺国度》）、Wolfram Schultz（剑桥）
- **机制**：快感-痛苦跷跷板。反复快速刺激导致神经适应，快感变弱，痛苦加强
- **在 vibe coding 中**：每 30-120 秒一次反馈循环，session 开始兴奋、结束空虚

### 2. 超时长心流
- **研究者**：Arne Dietrich（短暂前额叶抑制假说，2003）、Limb & Braun（fMRI，2008）
- **机制**：心流时前额叶皮层（自我监控、时间感知）被关闭
- **在 vibe coding 中**：AI 消除所有天然心流打断点，负责说"该停了"的脑区恰好被关掉

### 3. 谷氨酸积累
- **研究者**：Wiehler et al.（2022，*Current Biology*，巴黎脑研究所）
- **机制**：高强度认知工作导致侧前额叶谷氨酸浓度升高，进一步认知控制成本增加
- **在 vibe coding 中**：持续评估 AI 输出是纯 System 2 工作，前额叶在物理化学层面过载

### 4. 变比率强化（老虎机效应）
- **研究者**：B.F. Skinner、Natasha Dow Schull（NYU，《Addiction by Design》）
- **机制**：不可预测的奖励时机产生最高响应率，near-miss 激活的脑区≈真正获奖
- **在 vibe coding 中**：AI 输出时好时坏，"再试一个 prompt"≈"再来一把"

### 5. 默认模式网络（DMN）饥饿
- **研究者**：Marcus Raichle（华盛顿大学，2001）
- **机制**：DMN 在走神/发呆时激活，负责创造性联想和自我反思；持续专注会压制 DMN
- **在 vibe coding 中**：数小时持续专注完全压制 DMN，开发者失去察觉自身疲劳的能力

### 6. 停止信号缺失
- **研究者**：Adam Alter（NYU，《欲罢不能》）
- **机制**：上瘾技术移除停止信号（电视有集末，报纸有最后一页，无限滚动没有）
- **在 vibe coding 中**：prompt 循环没有天然终点

### 协同效应

六个机制相互强化：多巴胺透支驱动继续→变比率强化抵抗停止→DMN 压制消除自我监控→停止信号缺失移除外部触发→超时长心流关闭前额叶刹车→谷氨酸积累在生理层面降低认知能力。

## 关键数据点

| 来源 | 发现 |
|---|---|
| HBR/BCG 2026 (n=1488) | 14% AI 用户报告"AI brain fry"，决策疲劳 +33%，严重错误 +39%，离职意愿 +39% |
| K. Anders Ericsson | 世界顶级专家高强度认知工作上限：3-5 小时/天，单次 ~1 小时 |
| Kleitman 超日节律 | 大脑 ~90 分钟一个周期，最佳模式：90 分钟工作 + 15-20 分钟休息 |
| METR 2025 | 开发者自感快 20%，实际慢 19%——感知与现实鸿沟 |
| Wiehler 2022 | 一整天高认知工作后前额叶谷氨酸显著升高 |
| 番茄钟研究 | 固定 25 分钟间隔的科学支持有限，可能不匹配个体节律 |

## 产品方向决策

- **核心定位**：不是又一个番茄钟，而是"补回被 AI 吃掉的停止信号"
- **首发平台**：Claude Code（hooks 系统提供完整的生命周期事件）
- **智能提醒**：基于交互密度和模式（如老虎机模式检测），而非固定时间间隔
- **提醒内容**：基于神经科学，告诉用户正在发生什么，而非空洞的"该休息了"
- **纯本地**：所有数据存储在本地，不收费，开源

## 核心参考文献

- Lembke, A. (2021). *Dopamine Nation*. Dutton.
- Wiehler, A. et al. (2022). *Current Biology*, 32(16), 3564-3575.
- Dietrich, A. (2003). *Consciousness and Cognition*, 12(2), 231-256.
- Ericsson, A. & Pool, R. (2016). *Peak*. Houghton Mifflin.
- Alter, A. (2017). *Irresistible*. Penguin.
- Bedard, J. et al. (2026). When Using AI Leads to "Brain Fry." *HBR*.
- Csikszentmihalyi, M. (1990). *Flow*. Harper & Row.
- Schultz, W. (2016). *Dialogues in Clinical Neuroscience*, 18(1), 23-32.
- Schull, N.D. (2012). *Addiction by Design*. Princeton University Press.
