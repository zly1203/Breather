use rand::seq::SliceRandom;

use crate::rules::{FatigueReason, Severity};

/// Pick a reminder message based on severity and the dominant fatigue reason.
/// Severity controls tone (gentle → firm → urgent).
/// Reason controls content (what specifically to say).
pub fn get_reminder(severity: &Severity, reason: &FatigueReason) -> &'static str {
    let pool = match (severity, reason) {
        // ── Long session · Moderate ─────────────────────────────────
        (Severity::Moderate, FatigueReason::LongSession) => &[
            "Good stretch of focus. Maybe a quick look away from the screen?",
            "Time flies when you're in the zone. The code will be here when you get back.",
            "Your brain's been at it for a while. A glass of water might be nice.",
            "Nice run. Your eyes could probably use a break from the screen.",
            "You've been deep in this for a while. A minute of looking at something far away does wonders.",
            "Solid focus session. How about stretching those shoulders?",
            "You've been in the zone for a bit. A quick walk to the kitchen, maybe?",
            "Been a while since you moved. A few deep breaths, maybe a stretch?",
            "Good momentum. A short pause now keeps it going longer.",
            "Your brain's been running at full speed. Coast for a minute.",
        ][..],

        // ── Long session · High ─────────────────────────────────────
        (Severity::High, FatigueReason::LongSession) => &[
            "That's a long session. Your brain does some of its best work when you step away.",
            "You've been deep in this for a while. A few minutes of nothing might be exactly what you need.",
            "Still going? Sometimes the best thing you can do for a problem is forget about it for ten minutes.",
            "You've been at this for a while now. A real break, not just switching tabs, would help.",
            "Long session. Go look out the window for a bit. Seriously, it helps.",
            "You've been heads-down for quite a while. Your neck probably agrees it's time for a break.",
            "At this point a break would actually make you faster. Counterintuitive but true.",
            "Long stretch. Maybe step outside for a minute? Fresh air does something screens can't.",
        ][..],

        // ── Long session · Critical ─────────────────────────────────
        (Severity::Critical, FatigueReason::LongSession) => &[
            "It's been a long one. You've done enough. The rest can wait.",
            "You've earned a real break. Not five minutes. A real one.",
            "This is a very long session. Whatever you're chasing, it'll be clearer after rest.",
            "You've been at this for ages. The code isn't going anywhere. You should.",
            "Marathon session. Please step away for a while.",
            "You've pushed through impressively. Now push your chair back and stand up.",
            "You've been going for so long. Take a proper break. You'll come back sharper.",
            "Really long session. Go do something with your hands that isn't typing.",
        ][..],

        // ── Long day · Moderate ─────────────────────────────────────
        (Severity::Moderate, FatigueReason::LongDay) => &[
            "You've put in solid hours today. A short pause goes a long way.",
            "Productive day. Your brain's been working hard. Give it a moment.",
            "Full day of work. A pause now keeps the rest of it sharp.",
            "Today's been a good one. Keep it that way with a quick breather.",
            "You've been productive today. A few minutes off won't change that.",
            "Nice day of work so far. A quick break would feel good right about now.",
            "Hours well spent today. A short break makes the next hour count too.",
            "Good day so far. Maybe grab a snack or a coffee before the next stretch.",
        ][..],

        // ── Long day · High ────────────────────────────────────────
        (Severity::High, FatigueReason::LongDay) => &[
            "Full day of deep work. You've done a lot. Maybe ease off for a bit?",
            "You've been at this for hours today. The problems will look different after a break.",
            "Long day. Take a break and eat something. Seriously, when did you last eat?",
            "Hours and hours today. You're still sharp, but you won't be for long without a pause.",
            "You've given a lot today. A real break now, not later, would do you good.",
            "Big day. The next bug you find might just be tiredness talking. Take a break first.",
            "Long one today. Step away for a bit. Everything will still be here when you're back.",
            "You've been going all day. Your brain could use some downtime before the next push.",
        ][..],

        // ── Long day · Critical ────────────────────────────────────
        (Severity::Critical, FatigueReason::LongDay) => &[
            "You've given today everything. Time to stop.",
            "Long day. Your future self would really appreciate you stopping here.",
            "That's enough for one day. Seriously. You've done more than enough.",
            "You've been at this all day. Closing the laptop is the right move right now.",
            "Full day. You've done so much already. Let it be enough.",
            "Call it. Today was productive. You don't need to squeeze out more.",
            "All-day session. Please wrap up. You've earned the rest.",
            "You've been going since this morning. It's time. Really.",
        ][..],

        // ── Rapid prompting · Moderate ──────────────────────────────
        (Severity::Moderate, FatigueReason::RapidPrompting) => &[
            "You've been going back and forth pretty quickly. Might be worth pausing to think before the next prompt.",
            "Lots of rapid-fire prompts. Sometimes stepping back helps more than another attempt.",
            "Quick pace. Before the next prompt, maybe take a breath and think about what you actually want.",
            "You're prompting fast. A minute of thinking often saves ten minutes of trying.",
            "Noticing a lot of back-and-forth. A quick pause to gather your thoughts?",
            "Fast exchanges. Maybe slow down and think about the direction before the next one.",
            "Lots of prompts close together. Are you getting closer or just going faster?",
            "Lots of prompts in a short time. Sometimes the best prompt is the one you write after a break.",
        ][..],

        // ── Rapid prompting · High ──────────────────────────────────
        (Severity::High, FatigueReason::RapidPrompting) => &[
            "You're prompting faster and faster. If something's not clicking, a short walk might unstick it.",
            "Intense back-and-forth. When the same approach isn't landing, a fresh perspective helps more than another try.",
            "You've been firing prompts rapidly. Usually that means it's time to step back, not push harder.",
            "The pace is picking up. If you're stuck, more prompts probably won't help. A break might.",
            "Rapid-fire mode. Your subconscious is probably already working on this. Go let it do its thing.",
            "You're in a loop. A change of scenery might break it better than the next prompt.",
            "Noticing acceleration. When things aren't clicking, a 5-minute break often resets everything.",
            "Getting faster and faster. That usually means a walk around the block would help more than another attempt.",
        ][..],

        // ── Rapid prompting · Critical ──────────────────────────────
        (Severity::Critical, FatigueReason::RapidPrompting) => &[
            "You've been at this hard. Step away. The answer often comes when you stop looking for it.",
            "A lot of attempts in a short time. Seriously, take a break. This will still be here.",
            "You're deep in a frustration loop. Every developer's been there. A break is the way out.",
            "So many attempts. The thing that's blocking you right now? It'll be obvious after rest.",
            "You've been hammering at this for a while. Please take a real break before trying again.",
            "A lot of back-and-forth with no progress. That's okay. Walk away for a bit and come back fresh.",
            "You've tried so many times. A break isn't giving up. It's the fastest path forward right now.",
            "Please step away for a bit. Whatever's not working will still be here, and you'll see it differently after a break.",
        ][..],

        // ── No breaks · Moderate ────────────────────────────────────
        (Severity::Moderate, FatigueReason::NoBreaks) => &[
            "You haven't taken a real break in a while. Even a few minutes helps.",
            "A short pause between sessions makes each one better. Maybe now?",
            "Session after session. How about a few minutes away from the screen?",
            "You've been going continuously. Even a 3-minute break changes how the next hour feels.",
            "Nonstop work. A quick break between sessions keeps things fresh.",
            "When did you last stand up? Just asking.",
            "No real pauses for a while. A short one now would feel good.",
            "Back-to-back sessions. A little gap between them goes a long way.",
        ][..],

        // ── No breaks · High ────────────────────────────────────────
        (Severity::High, FatigueReason::NoBreaks) => &[
            "Session after session with no real break. Your brain needs some downtime to process what you've done.",
            "You've been grinding without a pause. A 10-minute break now saves an hour of fog later.",
            "No breaks for hours. You might not feel it yet, but it's catching up with you.",
            "Straight through, no stops. A real pause would help a lot right now. Walk around, look at something green.",
            "You haven't paused in a long time. Even just ten minutes would make a real difference.",
            "Nonstop. At this point you're not saving time by skipping breaks. You're losing it.",
            "You've been going without a pause for a while. Take ten minutes. You'll be glad you did.",
            "No real breaks all this time. Go make some tea, sit somewhere else for a bit.",
        ][..],

        // ── No breaks · Critical ────────────────────────────────────
        (Severity::Critical, FatigueReason::NoBreaks) => &[
            "You've been going nonstop. Please step away for real.",
            "No real breaks today. You're running on fumes. Please take care of yourself.",
            "Hours without a real pause. Be kind to yourself. Take a proper break.",
            "You haven't really stopped all day. Please go do something that isn't screens.",
            "Nonstop for way too long. A proper break. Please. Not a tab switch, a real one.",
            "No breaks, long day, still going. You'd tell a friend to stop. Listen to that voice.",
            "You've been going without rest for so long. A fifteen-minute break. Please.",
            "All this time without a real pause. Your body and mind both need you to stop for a bit.",
        ][..],

        // ── Late night · Moderate ───────────────────────────────────
        (Severity::Moderate, FatigueReason::LateNight) => &[
            "It's getting late. A good night's sleep is the best debugging tool there is.",
            "Late-night coding feels productive, but your brain is slower than you think right now.",
            "Getting late. The thing you're working on will make more sense in the morning.",
            "It's late. One more prompt, or one good night's sleep? The sleep wins every time.",
            "Evening coding. Wrap up what you can and save the hard stuff for tomorrow.",
            "Late hours. Whatever you're building, it deserves your morning brain, not your tired brain.",
            "Getting dark out. Maybe a good time to start wrapping up.",
            "It's late. How about finishing this thought and calling it a night?",
        ][..],

        // ── Late night · High ───────────────────────────────────────
        (Severity::High, FatigueReason::LateNight) => &[
            "It's late and you're still here. Whatever this is, it'll be easier to solve tomorrow morning.",
            "Night owl mode. But the bugs you write now are the ones you'll spend tomorrow fixing.",
            "Still coding this late? The gap between your day-brain and night-brain is bigger than you think.",
            "Late night session. You're writing code that morning-you will have opinions about.",
            "It's late. The late-night code that seemed brilliant? It almost never is. Save it for tomorrow.",
            "Night's getting deep. Your judgment is fuzzier than it feels right now.",
            "Late and still at it. This will be so much easier in the morning. Seriously.",
            "It's pretty late. Do future-you a favor and stop here.",
        ][..],

        // ── Late night · Critical ───────────────────────────────────
        (Severity::Critical, FatigueReason::LateNight) => &[
            "It's really late. Please go to bed. The code will be here tomorrow.",
            "Your bed misses you more than this codebase does. Call it a night.",
            "It's way too late for this. Save your work, close the laptop, go to sleep.",
            "Still here? At this hour? Nothing you write now is worth the sleep you're losing.",
            "It's the middle of the night. Whatever feels urgent right now won't feel urgent at 9am.",
            "Please go to sleep. You've done enough for today.",
            "It's so late. Save, commit, close. Tomorrow is a new day.",
            "Go to bed. Whatever this is, it can wait. You can't run on no sleep.",
        ][..],
    };

    let mut rng = rand::thread_rng();
    pool.choose(&mut rng).unwrap_or(&pool[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_severities() -> [Severity; 3] {
        [Severity::Moderate, Severity::High, Severity::Critical]
    }

    fn all_reasons() -> [FatigueReason; 5] {
        [
            FatigueReason::LongSession,
            FatigueReason::LongDay,
            FatigueReason::RapidPrompting,
            FatigueReason::NoBreaks,
            FatigueReason::LateNight,
        ]
    }

    #[test]
    fn every_combination_has_messages() {
        for sev in all_severities() {
            for reason in all_reasons() {
                let msg = get_reminder(&sev, &reason);
                assert!(!msg.is_empty(),
                    "empty message for {:?} / {:?}", sev, reason);
                assert!(msg.len() > 10,
                    "suspiciously short message for {:?} / {:?}: {}", sev, reason, msg);
            }
        }
    }

    #[test]
    fn no_em_dash_in_user_messages() {
        // Style rule: no em dashes in user-facing notification text.
        for sev in all_severities() {
            for reason in all_reasons() {
                for _ in 0..20 {
                    let msg = get_reminder(&sev, &reason);
                    assert!(!msg.contains('\u{2014}'),
                        "em dash found in {:?} / {:?}: {}", sev, reason, msg);
                }
            }
        }
    }

    #[test]
    fn messages_vary_within_pool() {
        // Each pool has multiple messages; over many draws we should see variety.
        let mut seen = std::collections::HashSet::new();
        for _ in 0..50 {
            seen.insert(get_reminder(&Severity::Moderate, &FatigueReason::LongSession));
        }
        assert!(seen.len() >= 3, "message pool looks degenerate: only {} unique in 50 draws", seen.len());
    }
}

