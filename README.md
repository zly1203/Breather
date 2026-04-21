# Breather

> A quiet reminder that you're still human.

Coding with AI is fast. Maybe too fast.

The old breaks are gone. The compile waits, the pauses between
typing, the moments you'd get up to grab water while the tests ran.
When Claude Code finishes in ninety seconds, you don't step away.
You just prompt again. And again. And again.

**Breather** sits quietly in your menubar. It watches your Claude
Code sessions, notices when you've been at it too long, and nudges
you gently to pause. No timers to set. No lock screens. No guilt.
Just a small companion that knows when to speak and when to stay
out of your way.

---

## Requirements

macOS 10.15 (Catalina) or later. Claude Code must be installed
for the hook to do anything; if it isn't, Breather will still run,
it'll just have nothing to listen to.

## Install

1. Download the latest `Breather_*.dmg` from [Releases](https://github.com/zly1203/Breather/releases).
2. Open it and drag **Breather** to `/Applications`.
3. Paste this once in Terminal (it clears the "unidentified developer" quarantine flag that macOS adds to downloaded apps):

   ```
   xattr -cr /Applications/Breather.app
   ```

4. Launch Breather from Applications. On first launch it registers
   a hook into Claude Code automatically and fires a confirmation
   notification.

### If you don't see the confirmation notification

macOS sometimes skips the initial permission prompt for unsigned
apps. Open **System Settings → Notifications**, scroll to
**Breather**, and switch "Allow notifications" on.

## Uninstall

1. Drag `Breather.app` from Applications to the Trash.
2. Remove the hook Breather installed:

   ```
   rm -rf ~/.breather
   ```

3. Edit `~/.claude/settings.json` and remove any entry under
   `hooks.Stop` whose `command` is `~/.breather/hook.sh`. Or, if
   you kept the CLI, run `npx breather uninstall` instead of steps
   2 and 3; it cleans up both.

---

## Three moods

| 🍵 Tea cup | 💡 Desk lamp | 🐱 Sleepy cat |
|:---:|:---:|:---:|
| Not working | Focused | Time to rest |

The slider tunes how sensitive Breather is. **Less** means it stays
quiet longer; **More** means it'll notice fatigue sooner.

Under the hood, Breather tracks five dimensions of fatigue: session
length, today's total, prompting pace, break debt, and time of day.
When the combined score crosses your threshold, it picks a gentle
message and delivers it at the next natural pause in your work.

---

## Questions or issues

Found a bug, want to suggest a different reminder, or just have a
question? [Open an issue](https://github.com/zly1203/Breather/issues).

## Background

- [Origin story](./CONTEXT.md): why this exists
- [Product requirements](./PRD.md): the thinking behind the design

## License

[PolyForm Noncommercial 1.0.0](./LICENSE). Free for personal,
educational, and research use. For commercial use, open an issue
on GitHub to get in touch.
