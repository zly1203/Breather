# Breather: Background and Research Notes

> A record of the project's origin and thinking, kept as context for
> future development.

## Origin

A simple observation: time spent using AI tools is rising sharply,
and people need a small tool to remind them to stop and re-engage
with the physical world. The first sketch was a Claude Code plugin
that would nudge the user to rest after a session crossed some time
threshold.

## Landscape research

### Existing products, by layer

**Desktop break reminders (most mature)**

- **Stretchly**: open source, 13k stars, the most popular option, but
  completely unaware of what the user is actually doing.
- **LookAway** (Mac, $15): smart enough to defer reminders during
  meetings and screen recording.
- **DeskBreak**: the closest to a developer-aware tool; suggests breaks
  after git commits.

**VS Code extensions (many, but simple)**

- A dozen or so Pomodoro-style timers, all basically stopwatches.
- **CodeFit**: the most complete (exercise prompts, gamification, team
  leaderboards), but has no awareness of AI usage.

**CLI tools**

- **pomo** (Go, 1.5k stars): terminal Pomodoro, scriptable.
- **Flomo**: computes break length from actual work duration
  (flowmodoro), rather than a fixed 25 minutes.

**Claude Code ecosystem**

- **Health Buddy Skill**: triggers support when the user types
  something like "I'm going crazy"; not a proactive timer.
- No existing session-timer or break-reminder plugin.

### Key conclusion

If the core feature is "remind me to rest every so often," pomo,
Flomo, and Stretchly already cover it. Differentiation has to come
from "AI awareness": knowing interaction rounds, interaction density,
and code-change volume, rather than tracking a clock.

## Core insight: six neural mechanisms

Deeper research confirms that fatigue from vibe coding is the result
of six neural mechanisms acting at once. These mechanisms are the
theoretical foundation of the product and its source of
differentiation.

### 1. Dopamine depletion

- **Researchers**: Anna Lembke (Stanford, *Dopamine Nation*);
  Wolfram Schultz (Cambridge).
- **Mechanism**: the pleasure-pain seesaw. Rapid, repeated rewards
  cause neural adaptation; the pleasure hit weakens and the trough
  that follows deepens.
- **In vibe coding**: a feedback loop every 30 to 120 seconds.
  Sessions start exciting and end empty.

### 2. Extended flow states

- **Researchers**: Arne Dietrich (transient hypofrontality hypothesis,
  2003); Limb and Braun (fMRI, 2008).
- **Mechanism**: during flow, the prefrontal cortex (self-monitoring,
  time perception) quiets down.
- **In vibe coding**: AI eliminates the natural interruption points
  that used to break flow, and the very brain region responsible for
  saying "time to stop" is the one that gets turned off.

### 3. Glutamate accumulation

- **Researchers**: Wiehler et al. (2022, *Current Biology*, Paris
  Brain Institute).
- **Mechanism**: sustained high-effort cognitive work raises
  glutamate concentrations in the lateral prefrontal cortex, making
  further cognitive control metabolically expensive.
- **In vibe coding**: continuously evaluating AI output is pure
  System 2 work. The prefrontal cortex becomes overloaded at a
  physical, chemical level.

### 4. Variable-ratio reinforcement (the slot-machine effect)

- **Researchers**: B.F. Skinner; Natasha Dow Schull (NYU,
  *Addiction by Design*).
- **Mechanism**: unpredictable reward timing produces the highest
  response rates. "Near-misses" activate nearly the same brain
  regions as actual wins.
- **In vibe coding**: AI output is good sometimes, bad sometimes.
  "Let me try one more prompt" is functionally "one more spin."

### 5. Default Mode Network (DMN) starvation

- **Researcher**: Marcus Raichle (Washington University, 2001).
- **Mechanism**: the DMN activates when the mind wanders and
  supports creative association and self-reflection. Sustained focus
  suppresses it.
- **In vibe coding**: hours of uninterrupted focus fully suppress
  the DMN, and with it the developer's ability to notice their own
  fatigue.

### 6. Missing stop signals

- **Researcher**: Adam Alter (NYU, *Irresistible*).
- **Mechanism**: addictive technology removes natural stopping
  points (television has an end-of-episode; newspapers have a last
  page; infinite scroll has neither).
- **In vibe coding**: the prompt loop has no natural endpoint.

### How they compound

The six mechanisms reinforce each other. Dopamine depletion drives
the urge to keep going; variable-ratio reinforcement resists
stopping; DMN suppression removes self-monitoring; missing stop
signals remove external triggers; extended flow silences the
prefrontal brakes; and glutamate accumulation lowers cognitive
capacity at the physiological level.

## Key data points

| Source | Finding |
|---|---|
| HBR / BCG 2026 (n=1488) | 14% of AI users report "AI brain fry." Decision fatigue +33%, serious errors +39%, intent to leave +39%. |
| K. Anders Ericsson | Even world-class experts cap high-effort cognitive work at 3 to 5 hours per day, ~1 hour per sitting. |
| Kleitman's ultradian rhythm | The brain runs on ~90-minute cycles. Optimal: 90 minutes of work, 15 to 20 minutes of rest. |
| METR 2025 | Developers feel 20% faster with AI but are measurably 19% slower. Perception-reality gap. |
| Wiehler 2022 | A full day of high-cognition work produces a significant glutamate rise in the prefrontal cortex. |
| Pomodoro research | The scientific support for a fixed 25-minute interval is thin. It may not match individual rhythms. |

## Product direction

- **Positioning**: not another Pomodoro timer. A tool that restores
  the stop signals AI erased.
- **First platform**: Claude Code. Its hooks system provides a clean
  lifecycle to observe.
- **Smart reminders**: driven by interaction density and patterns
  (like detecting slot-machine-style rapid prompting), not a fixed
  timer.
- **Reminder content**: informed by neuroscience. Tells the user
  what's actually happening rather than repeating "time to rest."
- **Local-only**: state lives on disk, no server, no pricing, open
  source.

## Core references

- Lembke, A. (2021). *Dopamine Nation*. Dutton.
- Wiehler, A. et al. (2022). *Current Biology*, 32(16), 3564-3575.
- Dietrich, A. (2003). *Consciousness and Cognition*, 12(2), 231-256.
- Ericsson, A. and Pool, R. (2016). *Peak*. Houghton Mifflin.
- Alter, A. (2017). *Irresistible*. Penguin.
- Bedard, J. et al. (2026). When Using AI Leads to "Brain Fry." *HBR*.
- Csikszentmihalyi, M. (1990). *Flow*. Harper and Row.
- Schultz, W. (2016). *Dialogues in Clinical Neuroscience*, 18(1), 23-32.
- Schull, N.D. (2012). *Addiction by Design*. Princeton University Press.
