# Vibe Break — 产品需求文档 (PRD)

> 你的 AI 编程守护者——在你忘记照顾自己的时候，温柔地提醒你。

## 1. 问题定义

### 现象

AI 编程正在成为主流开发方式。无论是用 Claude Code、Cursor、Copilot 还是 Windsurf，开发者与 AI 的交互频率极高，传统编程中的天然休息节点——编译等待、查文档、手动敲代码——被逐渐消除。

很多开发者开始注意到一种新的感受：不是"想不出来"的卡顿，而是"停不下来"的空虚。一抬头，几个小时已经过去了。

### 数据支撑

| 来源 | 发现 |
|---|---|
| HBR/BCG 2026 (n=1488) | 14% AI 用户报告"AI brain fry"，决策疲劳增加 33%，严重错误率增加 39% |
| K. Anders Ericsson | 世界顶级专家每天高强度认知工作上限：3-5 小时 |
| METR 2025 | 开发者自感效率提升 20%，实际完成速度慢了 19% |
| Kleitman 超日节律 | 大脑以 ~90 分钟为周期波动 |

### 为什么现有工具不够

市面上有大量休息提醒工具（Stretchly、pomo、CodeFit 等），但它们：

1. **不理解 AI 编程的独特节奏** — 只是通用计时器
2. **不感知交互密度** — 不知道你是在高强度编程还是偶尔问一句
3. **缺乏温度** — 冷冰冰的倒计时，没有人会在意一个闹钟在说什么

---

## 2. 设计理念

### 认知科学基础

我们的设计建立在对 AI 编程疲劳的深入理解之上。这种疲劳是多种神经机制共同作用的结果——多巴胺透支、变比率强化、超时长心流、谷氨酸积累、默认模式网络压制、停止信号缺失。这些机制是我们的设计理念基石，不需要一对一映射到产品功能中。详细的科学依据参见 [CONTEXT.md](CONTEXT.md)。

### 产品哲学

**Vibe Break 想做的，是那个在你沉浸工作时轻轻拍拍你肩膀的朋友。**

核心原则：

- **温柔** — 提醒是邀请，不是命令。你随时可以忽略它。
- **有温度** — 每一条提醒都带着善意，像朋友的关心，不像系统的警告。
- **不打扰** — 在合适的时机出现，出现一次就够了，不会反复唠叨。
- **有边界** — 尊重用户的自主权。我们提供觉察，不做决定。
- **懂你** — 基于你的实际使用模式提醒，而不是机械地倒计时。

### 视觉风格

- 有机 blob 形状角色，三种状态表情（平静/活力/疲惫）
- 实心填充、大胆有机形状、极简面部（参考 Emojis Mood Tracker 风格）
- 呼吸感 UI：Outfit 字体、毛玻璃表面、大量 `rgba()` 透明色、柔和绿色调
- 遵循 taste-skill.md 的 Anti-AI-Slop 设计原则
- 无 emoji，无纯黑色，无过饱和色

---

## 3. 目标用户

使用 AI 编程工具进行日常开发的程序员——无论工具、无论经验水平。只要你曾经在 AI 编程后感到那种说不清的疲惫，Vibe Break 就是为你准备的。

---

## 4. 产品定义

### 一句话定位

> **Vibe Break：你的 AI 编程守护者——温柔地补回被 AI 吃掉的停止信号。**

### 产品形态

**macOS Menubar App**（Tauri 框架），通过 Claude Code hooks 感知 AI 编程活动。

- 菜单栏常驻小图标，点击弹出状态窗口
- 不出现在 Dock 中，不干扰工作
- 开机自启动
- 未来可扩展至其他 AI 编程工具

### 架构

```
Claude Code (PostToolUse hook)
    │  hook.sh → curl POST
    ▼
Vibe Break Menubar App (Tauri, localhost:17422)
    ├── Rust 后端
    │   ├── HTTP Server (Axum) — 接收 hook 事件
    │   ├── State Manager — session 状态 + 历史记录 → ~/.vibe-break/state.json
    │   ├── Rules Engine — 干预规则（强度可调，线性插值）
    │   ├── Notification — macOS 系统通知 (osascript)
    │   └── Background Timer — 检测 session 超时，主动发送回顾
    └── Web 前端 (HTML/CSS/JS in WebView)
        ├── Blob 角色 SVG（三状态动态切换）
        ├── Session 实时统计
        ├── 强度连续滑块 (1-100)
        └── 历史 session 记录
```

### 核心功能

#### F1：智能 Session 感知

不只是计时，而是理解你的工作节奏：
- session 持续时长
- 交互轮次（每次工具调用 = 一次交互）
- 交互密度（滑动窗口内的频率）
- Session 自动管理（30 分钟无交互 = session 结束）

#### F2：懂你的提醒时机

基于你的实际状态，不是固定番茄钟：

| 场景 | 触发条件 |
|---|---|
| 持续专注 | session 持续 ~90 分钟未休息（随强度调整：45-120 分钟）|
| 高密度工作 | 短时间内大量交互（随强度调整：10-30 次/10分钟）|
| 太晚了 | 22:00 后仍在编程（每 session 一次）|

全局冷却：任意两次提醒之间至少间隔 20 分钟（随强度调整：10-30 分钟）。

#### F3：连续强度控制

用户通过一个滑块（1-100）调节提醒敏感度，无需理解内部参数：
- 1 (Gentle)：30 分钟冷却，120 分钟周期，密度阈值 30
- 50 (Balanced)：20 分钟冷却，83 分钟周期，密度阈值 20
- 100 (Attentive)：10 分钟冷却，45 分钟周期，密度阈值 10
- 中间值全部线性插值，平滑过渡

#### F4：有温度的提醒

- macOS 系统通知，不侵入终端
- 每种规则有多条随机文案，朋友式关心语气
- 静默通知（无声音），不打断心流

#### F5：Session 回顾

- Session 结束时主动发送回顾通知（后台每 2 分钟检测 30 分钟无交互）
- 新 session 开始时显示上次 session 摘要
- UI 中展示最近 5 条历史记录（最多保存 50 条）

#### F6：安全网

- App 未运行时，hook 会通过 Claude Code 的 additionalContext 提醒用户
- 开机自启动（macOS LaunchAgent）

#### F7：Blob 角色

UI 顶部的 SVG 有机角色，三种状态自动切换：
- **Calm**（绿色，闭眼微笑）— 无 session / 休息中
- **Fresh**（黄色，亮眼开心）— session < 60 分钟
- **Tired**（粉色，耷拉眉毛）— session >= 60 分钟

带 6 秒周期形状变形呼吸动画，状态切换有淡入淡出。

---

## 5. 非目标

- 不做游戏化 / 成就系统
- 不做团队功能
- 不做运动指导
- 不做强制锁定 / 阻断（你随时可以忽略提醒）
- 不收费，开源

---

## 6. 技术栈

| 层 | 技术 | 说明 |
|---|---|---|
| 桌面框架 | Tauri v2 | Rust 后端 + 系统 WebView，~5MB |
| 后端 | Rust + Axum | HTTP server, 状态管理, 规则引擎 |
| 前端 | HTML/CSS/JS | Outfit 字体, 毛玻璃 UI, SVG blob |
| Hook | Bash (curl) | 一行 HTTP POST，静默失败 |
| 通知 | osascript | macOS 原生通知 |
| 持久化 | JSON 文件 | ~/.vibe-break/state.json，纯本地 |
| 自启动 | tauri-plugin-autostart | macOS LaunchAgent |

### 安装

```bash
npx vibe-break init    # 注册 Claude Code hook
open "Vibe Break.app"  # 启动 menubar app
```

### 数据存储

`~/.vibe-break/state.json`（纯本地，不上传任何数据）：
```json
{
  "current_session": {
    "start_time": 1679234567890,
    "interaction_count": 42,
    "recent_interactions": [...]
  },
  "today": {
    "date": "2026-03-20",
    "total_minutes": 180,
    "sessions": 3
  },
  "history": [...],
  "intensity": 50
}
```

---

## 7. 成功指标

- **留存** — 用户装了之后没有关掉（7 日留存 > 40%）
- **不打扰** — 提醒不被视为干扰（满意度 > 4/5）
- **有用** — 用户认为提醒时机合理（> 60%）
- **行为改变** — 安装后平均 session 时长自然下降

---

## 8. 范围

### MVP（当前实现）

- Tauri macOS Menubar App
- Claude Code PostToolUse hook → HTTP → app
- Session 追踪（时长、交互次数、密度）
- 三条干预规则（时长、高密度、深夜）+ 全局冷却
- 连续强度滑块（1-100，线性插值）
- 温暖的系统通知 + 多条随机文案
- SVG Blob 角色（三状态动态切换 + 呼吸动画）
- Session 自动关闭 + 回顾通知
- 历史记录（最近 50 条 session）
- App 未运行提示
- 开机自启动
- 呼吸感 UI（Outfit 字体、毛玻璃、透明色）

### 未来迭代方向

- App icon / menubar icon 设计优化（当前使用程序生成的 blob，需要专业设计）
- 更智能的交互模式识别（重复 prompt 检测等）
- 扩展到其他 AI 编程工具（VS Code 扩展、Cursor 插件等）
- 可选的匿名使用统计
- 用户自定义提醒文案

---

## 9. 开放问题

1. **Icon 设计** — 目前用 Python PIL 生成的 blob icon 质量不够，需要专业设计或更好的生成方案
2. **提醒频率的平衡** — 需要真实用户反馈
3. **跨平台** — 成功后如何扩展到 Cursor / VS Code？hook 机制不同
4. **隐私** — 所有数据纯本地，是否需要可选的匿名聚合统计？

---

## 附录：核心参考文献

- Lembke, A. (2021). *Dopamine Nation*. Dutton.
- Wiehler, A. et al. (2022). *Current Biology*, 32(16), 3564-3575.
- Dietrich, A. (2003). *Consciousness and Cognition*, 12(2), 231-256.
- Ericsson, A. & Pool, R. (2016). *Peak*. Houghton Mifflin.
- Alter, A. (2017). *Irresistible*. Penguin.
- Bedard, J. et al. (2026). When Using AI Leads to "Brain Fry." *HBR*.
- Csikszentmihalyi, M. (1990). *Flow*. Harper & Row.
- Schultz, W. (2016). *Dialogues in Clinical Neuroscience*, 18(1), 23-32.
