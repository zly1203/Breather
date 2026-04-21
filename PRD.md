# Breather: Product Requirements Document

> A quiet reminder that you're still human.

## 1. The problem

### What's happening

AI-assisted coding is becoming the default way to build software.
Whether the tool is Claude Code, Cursor, Copilot, or Windsurf,
the interaction cadence is high, and the natural breaks that used
to punctuate programming (compile waits, reading docs, typing code
by hand) are largely gone.

Developers are starting to notice a new sensation. It isn't the
familiar "stuck on a problem" friction. It's an "I can't stop"
emptiness. You look up and hours have passed.

### Supporting data

| Source | Finding |
|---|---|
| HBR / BCG 2026 (n=1488) | 14% of AI users report "AI brain fry." Decision fatigue +33%, serious errors +39%. |
| K. Anders Ericsson | Even world-class experts cap high-effort cognitive work at 3 to 5 hours per day. |
| METR 2025 | Developers feel 20% faster with AI, but are measurably 19% slower. |
| Kleitman's ultradian rhythm | The brain runs on roughly 90-minute cycles. |

### Why existing tools fall short

There are plenty of break-reminder tools (Stretchly, pomo, CodeFit,
and so on), but they have three gaps:

1. **They don't understand the rhythm of AI coding.** They're
   general-purpose timers.
2. **They don't perceive interaction density.** They can't tell
   whether you're in a flow of rapid-fire prompts or just asking
   a question now and then.
3. **They feel cold.** A ticking countdown has no warmth. Nobody
   listens to an alarm that way.

---

## 2. Design philosophy

### Cognitive-science foundation

Our design is built on a specific understanding of AI-coding
fatigue. It's the product of several neural mechanisms acting at
once: dopamine depletion, variable-ratio reinforcement, extended
flow, glutamate accumulation, DMN suppression, and missing stop
signals. These mechanisms are the foundation of our product
thinking; they don't map one-to-one to features. The full reasoning
is in [CONTEXT.md](CONTEXT.md).

### What Breather is

**Breather wants to be the friend who taps your shoulder when
you're deep in the work.**

Core principles:

- **Gentle.** A reminder is an invitation, not an order. Ignore it
  any time.
- **Warm.** Every message should feel like a friend's concern, not
  a system warning.
- **Unintrusive.** Show up at the right moment, say it once, don't
  nag.
- **Respectful.** The user is an adult. We offer awareness, not
  decisions.
- **Attuned.** Remind based on actual patterns, not a mechanical
  countdown.

### Visual language

- Three illustrated characters (a tea cup, a desk lamp, a sleepy
  cat) as state indicators. Organic, hand-drawn feel.
- Breathing UI: Outfit typeface, frosted-glass surfaces, generous
  `rgba()` transparency, a soft green-leaning palette.
- No emoji, no pure black, no oversaturated color.
- No fixed-width timers or countdown digits.

---

## 3. Audience

Developers using AI coding tools in their daily work, regardless of
tool or experience level. If you've ever finished a long AI-coding
session feeling that specific kind of hollow tiredness, Breather
is for you.

---

## 4. Product definition

### One-line positioning

> **Breather: a gentle companion that restores the stop signals
> AI removed.**

### Form factor

**macOS menubar app** (Tauri), listening to AI coding activity via
Claude Code hooks.

- A small menubar icon; click to open the status window.
- No Dock icon. Stays out of the way.
- Optional launch-at-login.
- Future expansion to other AI coding tools.

### Architecture

```
Claude Code (Stop hook)
    │  hook.sh → HTTP POST
    ▼
Breather menubar app (Tauri, localhost:17422)
    ├── Rust backend
    │   ├── HTTP server (Axum): receives hook events
    │   ├── State manager: session + history in ~/.breather/state.json
    │   ├── Fatigue engine: 5-dimension score, threshold from slider
    │   ├── Notifications: native macOS via mac-notification-sys
    │   └── Background timer: detects session timeout, sends summary
    └── Web frontend (HTML / CSS / JS in WebView)
        ├── Character illustrations (idle / fresh / tired)
        ├── Session stats (Today, Now, Interactions)
        └── Intensity slider (1 to 100, continuous)
```

### Core features

#### F1. Smart session tracking

Not just a clock. Tracks:

- Session duration.
- Interaction count (one Claude Code Stop event = one interaction).
- Interaction density over a sliding window.
- Automatic session lifecycle (30 minutes without an interaction
  closes the session and writes a summary).

#### F2. Context-aware reminders

Reminders are driven by a five-dimension fatigue score, not a fixed
Pomodoro timer. The dimensions are:

1. **Session duration**: exponential curve around the ~90-minute
   ultradian rhythm.
2. **Cumulative daily work**: today's total minutes.
3. **Prompting density**: recent interaction rate vs. the
   preceding window (detects frustration loops).
4. **Break debt**: time since the last real break, factoring in
   break quality.
5. **Time of day**: multiplier that makes late-night work feel
   fatigued sooner.

When the combined score crosses the user's threshold, Breather
waits for a natural pause in their work (based on median gap
between recent interactions) and then picks a message. Reminders
have a jittered cooldown to avoid habituation.

#### F3. Continuous intensity control

A single slider (1 to 100) controls sensitivity:

- **Less (1)**: threshold ~80. Infrequent reminders.
- **Balanced (50)**: threshold ~58.
- **More (100)**: threshold ~35. More attentive.

Intermediate values interpolate smoothly. The slider also governs
when the character changes to "tired."

#### F4. Warm notification text

- Native macOS notifications, with Breather's own app icon.
- Each reason / severity combination has a pool of handwritten
  messages. The tone stays friend-like at every severity.
- Silent by default (no sound) so reminders don't interrupt flow.

#### F5. Session summary

- When a session goes stale (30 minutes of inactivity), the
  background timer closes it and optionally sends a summary
  notification for sessions of meaningful length.
- History is kept in state.json (last 50 sessions).

#### F6. Safety net

- If the app isn't running when a hook fires, the hook returns an
  `additionalContext` message nudging the user to open Breather.
- The app auto-installs its Claude Code hook on first launch
  (idempotent; preserves other hooks the user has configured).

#### F7. The three characters

| State | Character | When it shows |
|---|---|---|
| Idle | 🍵 Tea cup | No active session |
| Fresh | 💡 Desk lamp | Session active, fatigue below threshold |
| Tired | 🐱 Sleepy cat | Session active, fatigue at or above threshold |

Fade transitions between states. No countdown clock, no numeric
progress bar.

---

## 5. Non-goals

- No gamification or achievement systems.
- No team features.
- No exercise coaching.
- No forced lockouts or blocking (ignore any reminder at will).
- No paid tier in the initial release; the source is open.

---

## 6. Tech stack

| Layer | Tool | Notes |
|---|---|---|
| Desktop framework | Tauri v2 | Rust backend + system WebView, ~10MB |
| Backend | Rust + Axum | HTTP server, state, fatigue engine |
| Frontend | HTML / CSS / JS | Outfit (bundled locally), frosted-glass UI |
| Hook | Bash + curl | Silent on failure, one POST per Stop event |
| Notifications | mac-notification-sys | Native macOS, bound to app bundle ID |
| Persistence | JSON | `~/.breather/state.json`, atomic writes |
| Autostart | tauri-plugin-autostart | macOS LaunchAgent |

### Install

Drop `Breather.app` into `/Applications`. On first launch the app:

1. Writes `~/.breather/hook.sh`.
2. Registers a Stop hook in `~/.claude/settings.json`, preserving
   any other hooks that were there.
3. Fires a confirmation notification.

No CLI step required. A CLI (`bin/breather.js`) is kept as a
fallback for `init` / `status` / `uninstall`.

### State

`~/.breather/state.json` is local-only and looks roughly like:

```json
{
  "current_session": {
    "start_time": 1679234567890,
    "interaction_count": 42,
    "recent_interactions": [],
    "last_reminder_time": null
  },
  "today": {
    "date": "2026-04-20",
    "total_minutes": 180,
    "sessions": 3
  },
  "last_session_summary": null,
  "history": [],
  "intensity": 50
}
```

---

## 7. Success signals

What "good" would look like if measured. These are aspirational
targets for a future evaluation, not current numbers.

- **Retention.** Installed users keep the app running past the
  first week.
- **Low-friction.** Reminders are not reported as intrusive in
  qualitative feedback.
- **Useful timing.** Users feel the reminders arrive at sensible
  moments most of the time.
- **Behavior change.** Average session length drops after
  installation.

---

## 8. Scope

### Current MVP

- Tauri macOS menubar app (universal binary).
- Claude Code Stop hook → HTTP → app.
- Session tracking (duration, interaction count, density).
- Five-dimension fatigue score + intensity-driven threshold.
- Continuous intensity slider (1 to 100, linear interpolation).
- Warm notifications with randomized message pools.
- Three-character state indicator with cross-fade transitions.
- Automatic session close and optional summary notification.
- Rolling history (last 50 sessions).
- "App not running" hint through Claude Code's `additionalContext`.
- Launch-at-login.
- Breathing-room UI (Outfit font bundled locally, frosted glass).

### Future directions

- More refined pattern detection (e.g., explicit frustration-loop
  heuristics, repeated near-identical prompts).
- Expansion to other AI coding tools (Cursor, VS Code extensions).
- Optional, opt-in anonymous usage statistics.
- User-editable reminder text.
- Code signing and notarization for friction-free installation.

---

## 9. Open questions

1. **Reminder frequency balance.** Needs real-user feedback.
2. **Cross-platform.** What's the cleanest expansion path to
   Cursor and VS Code? The hook mechanism is different.
3. **Privacy vs. insight.** All data is local. Is there ever a
   case for optional anonymous aggregation?

---

## Appendix: core references

- Lembke, A. (2021). *Dopamine Nation*. Dutton.
- Wiehler, A. et al. (2022). *Current Biology*, 32(16), 3564-3575.
- Dietrich, A. (2003). *Consciousness and Cognition*, 12(2), 231-256.
- Ericsson, A. and Pool, R. (2016). *Peak*. Houghton Mifflin.
- Alter, A. (2017). *Irresistible*. Penguin.
- Bedard, J. et al. (2026). When Using AI Leads to "Brain Fry." *HBR*.
- Csikszentmihalyi, M. (1990). *Flow*. Harper and Row.
- Schultz, W. (2016). *Dialogues in Clinical Neuroscience*, 18(1), 23-32.
