use crate::state::StateManager;
use chrono::{Local, Timelike};
use rand::Rng;

/// The fatigue-score model replaces the old 3-rule system.
/// A single score is computed from 5 dimensions, then compared against
/// a threshold derived from the user's intensity slider.

// ── Fatigue Score Computation ────────────────────────────────────────

/// Detailed breakdown of fatigue score by dimension.
#[derive(Debug, Clone)]
pub struct FatigueBreakdown {
    pub session: f64,
    pub cumulative: f64,
    pub density: f64,
    pub break_debt: f64,
    pub late_night: bool,
    pub total: f64,
}

impl FatigueBreakdown {
    /// Return the dominant reason for fatigue (highest contributor).
    pub fn dominant_reason(&self) -> FatigueReason {
        // Late night is special — if multiplier is boosting, it's the most actionable signal.
        if self.late_night && self.total > 0.0 {
            return FatigueReason::LateNight;
        }

        let scores = [
            (FatigueReason::LongSession, self.session),
            (FatigueReason::LongDay, self.cumulative),
            (FatigueReason::RapidPrompting, self.density),
            (FatigueReason::NoBreaks, self.break_debt),
        ];

        scores.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(reason, _)| reason.clone())
            .unwrap_or(FatigueReason::LongSession)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FatigueReason {
    LongSession,
    LongDay,
    RapidPrompting,
    NoBreaks,
    LateNight,
}

/// Compute the current fatigue score with a per-dimension breakdown.
pub fn compute_fatigue(state_mgr: &StateManager) -> FatigueBreakdown {
    let state = state_mgr.state.lock().unwrap();
    let session = match state.current_session.as_ref() {
        Some(s) => s,
        None => return FatigueBreakdown {
            session: 0.0, cumulative: 0.0, density: 0.0,
            break_debt: 0.0, late_night: false, total: 0.0,
        },
    };
    let now = Local::now().timestamp_millis();

    // ① Session duration contribution (exponential curve aligned to ultradian rhythm).
    let minutes = ((now - session.start_time) as f64) / 60_000.0;
    let session_score = (minutes / 90.0).powf(1.5) * 30.0;

    // ② Cumulative daily work contribution.
    let today_min = state.today.total_minutes as f64 + minutes;
    let cumulative_score = if today_min < 210.0 {
        0.0
    } else if today_min < 360.0 {
        (today_min - 210.0) / 150.0 * 20.0
    } else if today_min < 480.0 {
        20.0 + (today_min - 360.0) / 120.0 * 15.0
    } else {
        (35.0 + (today_min - 480.0) / 120.0 * 15.0).min(50.0)
    };

    // ③ Density acceleration contribution.
    let density_score = compute_density_acceleration(&session.recent_interactions, now);

    // ④ Time-of-day multiplier.
    let hour = Local::now().hour();
    let late_night = hour >= 22 || hour <= 4;
    let time_multiplier = match hour {
        0..=4 => 1.5,   // Very late / early morning
        5..=8 => 1.0,   // Early morning, normal
        9..=11 => 0.8,  // Morning peak, lenient
        12..=19 => 1.0, // Normal work hours
        20..=21 => 1.2, // Evening, slightly sensitive
        _ => 1.5,       // 22+, late night, should stop
    };

    // ⑤ Break debt contribution.
    let break_debt_score = compute_break_debt(&state.last_session_summary, session.start_time, now);

    let raw = session_score + cumulative_score + density_score + break_debt_score;
    let total = raw * time_multiplier;

    FatigueBreakdown {
        session: session_score,
        cumulative: cumulative_score,
        density: density_score,
        break_debt: break_debt_score,
        late_night,
        total,
    }
}

/// Density acceleration: compare prompt rate in the last 10 minutes
/// vs the 10 minutes before that. Rising rate = possible anxiety loop.
/// Now tracks prompts (Stop events), not tool calls — so counts are much lower.
/// Wider window (10 min) smooths out noise from low prompt counts.
fn compute_density_acceleration(interactions: &[i64], now: i64) -> f64 {
    let window_ms: i64 = 10 * 60 * 1000; // 10 minutes

    let recent = interactions.iter()
        .filter(|&&t| now - t < window_ms)
        .count() as f64;

    let previous = interactions.iter()
        .filter(|&&t| now - t >= window_ms && now - t < window_ms * 2)
        .count() as f64;

    if previous < 2.0 {
        // Not enough history — only penalize if prompting extremely fast.
        // >12 prompts in 10 min = more than one per 50 seconds, genuinely frantic.
        if recent > 12.0 { 10.0 } else { 0.0 }
    } else {
        let ratio = recent / previous;
        if ratio > 2.5 {
            15.0 // Strong acceleration — likely frustration loop
        } else if ratio > 1.8 {
            8.0  // Moderate acceleration
        } else if ratio < 0.5 {
            -5.0 // Slowing down — good sign
        } else {
            0.0  // Stable
        }
    }
}

/// Break debt: based on time since last real break and break quality.
fn compute_break_debt(
    last_summary: &Option<crate::state::SessionSummary>,
    session_start: i64,
    now: i64,
) -> f64 {
    let break_duration_min = match last_summary {
        Some(summary) => {
            // Gap between last session end and current session start.
            let gap_ms = session_start - summary.end_time;
            (gap_ms as f64 / 60_000.0).max(0.0)
        }
        None => 999.0, // No previous session — no debt.
    };

    let time_since_break_min = (now - session_start) as f64 / 60_000.0;

    if break_duration_min >= 15.0 && time_since_break_min < 60.0 {
        -15.0 // Good recent break, big relief
    } else if break_duration_min >= 5.0 && time_since_break_min < 120.0 {
        -5.0  // Decent break, some relief
    } else if break_duration_min < 5.0 && time_since_break_min > 180.0 {
        20.0  // No real break and long continuous work
    } else {
        0.0
    }
}

// ── Threshold from Intensity Slider ──────────────────────────────────

/// Map slider value (1–100) to fatigue threshold.
/// Lower threshold = more sensitive = more reminders.
fn threshold_from_intensity(intensity: u8) -> f64 {
    let t = (intensity.clamp(1, 100) as f64 - 1.0) / 99.0; // 0.0 .. 1.0
    // Less(1) = threshold 80, More(100) = threshold 35
    lerp(80.0, 35.0, t)
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

// ── Natural Pause Detection ──────────────────────────────────────────

/// Check if the user is currently in a natural pause (prompt gap > median × 1.5).
fn is_natural_pause(interactions: &[i64], now: i64) -> bool {
    if interactions.len() < 3 {
        return true; // Not enough data, assume it's fine.
    }

    // Compute median gap between recent interactions.
    let mut gaps: Vec<i64> = interactions.windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    if gaps.is_empty() {
        return true;
    }
    gaps.sort();
    let median_gap = gaps[gaps.len() / 2];

    // Current gap = time since last interaction.
    let last = *interactions.last().unwrap();
    let current_gap = now - last;

    // A "natural pause" is when the current gap is notably longer than usual.
    current_gap > median_gap * 3 / 2
}

// ── Anti-habituation: Jittered Cooldown ──────────────────────────────

/// Minimum cooldown between reminders, with ±15% jitter.
/// Base cooldown scales with intensity: Less = 30 min, More = 15 min.
fn jittered_cooldown_ms(intensity: u8) -> i64 {
    let t = (intensity.clamp(1, 100) as f64 - 1.0) / 99.0;
    let base_min = lerp(30.0, 15.0, t);
    let jitter = rand::thread_rng().gen_range(-0.15..0.15);
    let cooldown_min = base_min * (1.0 + jitter);
    (cooldown_min * 60_000.0) as i64
}

// ── Public Interface ─────────────────────────────────────────────────

/// Check whether a reminder should fire. Returns the fatigue trigger if so.
/// Two paths can trigger a reminder:
///   1. Baseline check-in: guaranteed first reminder at ~45 min if none sent yet.
///   2. Fatigue-driven: score exceeds threshold (severity scales with ratio).
/// Both share the same cooldown — they never stack.
pub fn check_fatigue(state_mgr: &StateManager) -> Option<FatigueTrigger> {
    let state = state_mgr.state.lock().unwrap();
    let session = match state.current_session.as_ref() {
        Some(s) => s,
        None => return None,
    };
    let now = Local::now().timestamp_millis();
    let intensity = state.intensity;
    let session_minutes = (now - session.start_time) as f64 / 60_000.0;

    // Cooldown: don't remind too frequently.
    if let Some(last) = session.last_reminder_time {
        if now - last < jittered_cooldown_ms(intensity) {
            return None;
        }
    }

    // ── Path 1: Baseline check-in ───────────────────────────────────
    // If no reminder has been sent this session and we've passed 45 min,
    // send a gentle check-in so the user knows the app is paying attention.
    // After this, enforce a 30-min cooldown before any fatigue-driven reminder.
    if session.last_reminder_time.is_none() && session_minutes >= 45.0 {
        return Some(FatigueTrigger {
            severity: Severity::Moderate,
            reason: FatigueReason::LongSession,
        });
    }

    // If last reminder was the baseline, enforce 30-min cooldown regardless of slider.
    if let Some(last) = session.last_reminder_time {
        let since_last = now - last;
        let baseline_cooldown_ms: i64 = 30 * 60 * 1000;
        if since_last < baseline_cooldown_ms && session_minutes < 80.0 {
            // Still within baseline cooldown window (session < ~80 min).
            // After 80 min the normal cooldown logic above already governs.
            return None;
        }
    }

    drop(state); // Release lock before computing fatigue (which also locks).

    // ── Path 2: Fatigue-driven ──────────────────────────────────────
    let breakdown = compute_fatigue(state_mgr);
    let threshold = threshold_from_intensity(intensity);

    if breakdown.total < threshold {
        return None;
    }

    // Score exceeded threshold — wait for natural pause.
    let state = state_mgr.state.lock().unwrap();
    let session = state.current_session.as_ref()?;

    let in_pause = is_natural_pause(&session.recent_interactions, now);

    // If not in a natural pause, only fire if score is significantly over threshold
    // (meaning we've been waiting a while and the user hasn't paused).
    if !in_pause && breakdown.total < threshold * 1.4 {
        return None; // Wait for a pause.
    }

    // Determine severity for message selection.
    let ratio = breakdown.total / threshold;
    let severity = if ratio >= 2.0 {
        Severity::Critical
    } else if ratio >= 1.5 {
        Severity::High
    } else {
        Severity::Moderate
    };

    let reason = breakdown.dominant_reason();

    Some(FatigueTrigger { severity, reason })
}

/// Decide which character to show, based on the same fatigue model
/// that drives reminders. Keeps the slider meaningful: more sensitive
/// slider = tired character appears sooner.
///
/// Returns: "idle" (no session), "fresh" (fatigue below threshold),
/// or "tired" (fatigue at or above threshold).
pub fn character_state(state_mgr: &StateManager) -> &'static str {
    let intensity = {
        let s = state_mgr.state.lock().unwrap();
        if s.current_session.is_none() {
            return "idle";
        }
        s.intensity
    };
    let breakdown = compute_fatigue(state_mgr);
    let threshold = threshold_from_intensity(intensity);
    if breakdown.total >= threshold { "tired" } else { "fresh" }
}

/// Mark that a reminder was sent, so cooldown starts.
pub fn mark_reminded(state_mgr: &StateManager) {
    let mut state = state_mgr.state.lock().unwrap();
    let now = Local::now().timestamp_millis();
    if let Some(ref mut session) = state.current_session {
        session.last_reminder_time = Some(now);
    }
}

#[derive(Debug, Clone)]
pub struct FatigueTrigger {
    pub severity: Severity,
    pub reason: FatigueReason,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Moderate,  // Just crossed threshold
    High,      // 1.5× threshold
    Critical,  // 2×+ threshold (very long session, late night, etc.)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_maps_inversely_to_intensity() {
        // Less (intensity 1) should have the highest threshold (least sensitive).
        // More (intensity 100) should have the lowest threshold (most sensitive).
        let low = threshold_from_intensity(1);
        let mid = threshold_from_intensity(50);
        let high = threshold_from_intensity(100);
        assert!(low > mid, "intensity 1 should have higher threshold than 50");
        assert!(mid > high, "intensity 50 should have higher threshold than 100");
        assert!((low - 80.0).abs() < 0.5, "intensity 1 should be ~80");
        assert!((high - 35.0).abs() < 0.5, "intensity 100 should be ~35");
    }

    #[test]
    fn threshold_is_monotonic() {
        let mut prev = f64::INFINITY;
        for i in 1..=100u8 {
            let t = threshold_from_intensity(i);
            assert!(t <= prev + 0.001,
                "threshold should not increase as intensity rises: {} at {}", t, i);
            prev = t;
        }
    }

    #[test]
    fn dominant_reason_favors_late_night() {
        let b = FatigueBreakdown {
            session: 100.0, cumulative: 100.0, density: 100.0, break_debt: 100.0,
            late_night: true, total: 50.0,
        };
        assert_eq!(b.dominant_reason(), FatigueReason::LateNight);
    }

    #[test]
    fn dominant_reason_picks_highest_contributor() {
        let b = FatigueBreakdown {
            session: 10.0, cumulative: 5.0, density: 50.0, break_debt: 2.0,
            late_night: false, total: 67.0,
        };
        assert_eq!(b.dominant_reason(), FatigueReason::RapidPrompting);

        let b = FatigueBreakdown {
            session: 40.0, cumulative: 10.0, density: 5.0, break_debt: 2.0,
            late_night: false, total: 57.0,
        };
        assert_eq!(b.dominant_reason(), FatigueReason::LongSession);
    }

    #[test]
    fn natural_pause_requires_minimum_history() {
        // Too few samples: defaults to "in pause".
        assert!(is_natural_pause(&[], 1000));
        assert!(is_natural_pause(&[100], 1000));
        assert!(is_natural_pause(&[100, 200], 1000));
    }

    #[test]
    fn natural_pause_detects_gap_after_rapid_prompts() {
        // Tight cluster: gaps are 1s each.
        let base = 1_000_000;
        let interactions = vec![base, base + 1000, base + 2000, base + 3000];
        // Now = 5 seconds after last: gap = 2000 > median (1000) * 1.5.
        assert!(is_natural_pause(&interactions, base + 5000));
        // Now = 500ms after last: gap = 500 < median (1000) * 1.5.
        assert!(!is_natural_pause(&interactions, base + 3500));
    }

    #[test]
    fn fatigue_zero_without_session() {
        use std::path::PathBuf;
        let dir: PathBuf = std::env::temp_dir().join(format!("breather-rules-{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let mgr = StateManager::with_data_dir(dir);
        let b = compute_fatigue(&mgr);
        assert_eq!(b.total, 0.0);
        assert!(check_fatigue(&mgr).is_none());
    }

    #[test]
    fn character_idle_without_session() {
        use std::path::PathBuf;
        let dir: PathBuf = std::env::temp_dir().join(format!("breather-char1-{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let mgr = StateManager::with_data_dir(dir);
        assert_eq!(character_state(&mgr), "idle");
    }

    #[test]
    fn character_fresh_on_new_session() {
        use std::path::PathBuf;
        let dir: PathBuf = std::env::temp_dir().join(format!("breather-char2-{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let mgr = StateManager::with_data_dir(dir);
        mgr.record_interaction();
        // A brand-new session has near-zero fatigue; should be fresh.
        assert_eq!(character_state(&mgr), "fresh");
    }

    #[test]
    fn check_fatigue_respects_cooldown() {
        use std::path::PathBuf;
        let dir: PathBuf = std::env::temp_dir().join(format!("breather-cooldown-{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let mgr = StateManager::with_data_dir(dir);
        mgr.record_interaction();
        // Mark as just reminded. Cooldown should suppress the next check.
        mark_reminded(&mgr);
        assert!(check_fatigue(&mgr).is_none(),
            "should not fire while in cooldown");
    }
}
