use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// How long before a session is considered stale (30 min).
const STALE_MS: i64 = 30 * 60 * 1000;
/// Max recent interactions to keep for density calculations.
/// Need ~6 min of history for acceleration detection at high interaction rates.
const MAX_RECENT: usize = 60;
/// How much to backdate a new session's start_time to compensate for the time
/// the user spent composing & awaiting the first Stop event.
/// Stop hook fires when Claude finishes responding, but the user began the turn
/// a minute or two earlier — this offset accounts for that gap.
const SESSION_START_OFFSET_MS: i64 = 90 * 1000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub start_time: i64,
    pub interaction_count: u32,
    pub recent_interactions: Vec<i64>,
    pub last_reminder_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayState {
    pub date: String,
    pub total_minutes: u32,
    pub sessions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub start_time: i64,
    pub end_time: i64,
    pub duration_minutes: u32,
    pub interaction_count: u32,
    pub date: String,
}

/// Max history entries to keep.
const MAX_HISTORY: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_session: Option<SessionState>,
    pub today: DayState,
    pub last_session_summary: Option<SessionSummary>,
    pub history: Vec<SessionSummary>,
    pub intensity: u8, // 1 (gentle) — 100 (attentive), default 50
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_session: None,
            today: DayState {
                date: today_str(),
                total_minutes: 0,
                sessions: 0,
            },
            last_session_summary: None,
            history: Vec::new(),
            intensity: 50,
        }
    }
}

pub struct StateManager {
    pub state: Mutex<AppState>,
    data_dir: PathBuf,
}

impl StateManager {
    pub fn new() -> Self {
        let data_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".breather");
        Self::with_data_dir(data_dir)
    }

    pub fn with_data_dir(data_dir: PathBuf) -> Self {
        fs::create_dir_all(&data_dir).ok();
        let state = Self::load_from_disk(&data_dir);
        Self {
            state: Mutex::new(state),
            data_dir,
        }
    }

    fn state_path(data_dir: &PathBuf) -> PathBuf {
        data_dir.join("state.json")
    }

    fn load_from_disk(data_dir: &PathBuf) -> AppState {
        let path = Self::state_path(data_dir);
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => AppState::default(),
        }
    }

    pub fn save(&self) {
        let state = self.state.lock().unwrap();
        let final_path = Self::state_path(&self.data_dir);
        let tmp_path = final_path.with_extension("json.tmp");
        let Ok(json) = serde_json::to_string_pretty(&*state) else { return };
        // Write to a temp file then atomically rename. This prevents a
        // half-written state.json if the process dies mid-write.
        if fs::write(&tmp_path, json).is_err() { return }
        fs::rename(&tmp_path, &final_path).ok();
    }

    /// Record a new interaction. Returns (is_new_session, previous_summary).
    pub fn record_interaction(&self) -> (bool, Option<SessionSummary>) {
        let mut state = self.state.lock().unwrap();
        let now = now_ms();

        // Reset daily totals if new day.
        if state.today.date != today_str() {
            state.today = DayState {
                date: today_str(),
                total_minutes: 0,
                sessions: 0,
            };
        }

        let mut is_new = false;
        let mut prev_summary = None;

        // Detect stale or missing session.
        if let Some(ref session) = state.current_session {
            let last = session.recent_interactions.last()
                .copied()
                .unwrap_or(session.start_time);
            if now - last > STALE_MS {
                prev_summary = close_session(&mut state);
                is_new = true;
            }
        } else {
            is_new = true;
        }

        if is_new {
            state.current_session = Some(SessionState {
                // Backdate start to approximate when the user actually began this turn.
                start_time: now - SESSION_START_OFFSET_MS,
                interaction_count: 0,
                recent_interactions: Vec::new(),
                last_reminder_time: None,
            });
        }

        // Record the interaction.
        if let Some(ref mut session) = state.current_session {
            session.interaction_count += 1;
            session.recent_interactions.push(now);
            if session.recent_interactions.len() > MAX_RECENT {
                let drain = session.recent_interactions.len() - MAX_RECENT;
                session.recent_interactions.drain(..drain);
            }
        }

        (is_new, prev_summary)
    }

    pub fn get_stats(&self) -> Option<SessionStats> {
        let state = self.state.lock().unwrap();
        let session = state.current_session.as_ref()?;
        let now = now_ms();

        // If the session has gone stale, cap the duration at the last interaction.
        // The background timer will close it properly on its next tick; meanwhile
        // the UI shouldn't show a duration that keeps growing while the user is away.
        let last_interaction = session.recent_interactions.last()
            .copied()
            .unwrap_or(session.start_time);
        let effective_end = if now - last_interaction > STALE_MS {
            last_interaction
        } else {
            now
        };
        let duration_min = ((effective_end - session.start_time) / 60000).max(0) as u32;

        Some(SessionStats {
            duration_minutes: duration_min,
            interaction_count: session.interaction_count,
            today_total_minutes: state.today.total_minutes + duration_min,
            today_sessions: state.today.sessions + 1,
            intensity: state.intensity,
        })
    }

    pub fn get_history(&self) -> Vec<SessionSummary> {
        let state = self.state.lock().unwrap();
        state.history.clone()
    }

    /// Today's total accumulated minutes — closed sessions today plus the
    /// current session's contribution (capped at last interaction if stale).
    pub fn get_today_total(&self) -> u32 {
        let state = self.state.lock().unwrap();
        let today = today_str();

        // If `state.today.date` is from a previous day, treat closed total as 0.
        let closed_total = if state.today.date == today {
            state.today.total_minutes
        } else {
            0
        };

        // Add the current session contribution if any, using the same
        // staleness-aware logic as get_stats.
        let current_minutes = match state.current_session.as_ref() {
            Some(session) => {
                let now = now_ms();
                let last_interaction = session.recent_interactions.last()
                    .copied()
                    .unwrap_or(session.start_time);
                let effective_end = if now - last_interaction > STALE_MS {
                    last_interaction
                } else {
                    now
                };
                ((effective_end - session.start_time) / 60000).max(0) as u32
            }
            None => 0,
        };

        closed_total + current_minutes
    }

    /// Closed sessions whose date matches today, in chronological order.
    pub fn get_today_history(&self) -> Vec<SessionSummary> {
        let state = self.state.lock().unwrap();
        let today = today_str();
        state.history.iter()
            .filter(|s| s.date == today)
            .cloned()
            .collect()
    }

    /// Check if the current session has gone stale. If so, close it and return summary.
    pub fn check_and_close_stale(&self) -> Option<SessionSummary> {
        let mut state = self.state.lock().unwrap();
        let now = now_ms();

        if let Some(ref session) = state.current_session {
            let last = session.recent_interactions.last()
                .copied()
                .unwrap_or(session.start_time);
            if now - last > STALE_MS {
                return close_session(&mut state);
            }
        }
        None
    }

    /// Gracefully close the current session (e.g. on app quit).
    pub fn close_current_session(&self) {
        let mut state = self.state.lock().unwrap();
        if state.current_session.is_some() {
            close_session(&mut state);
        }
    }

    pub fn set_intensity(&self, level: u8) {
        let mut state = self.state.lock().unwrap();
        state.intensity = level.clamp(1, 100);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub duration_minutes: u32,
    pub interaction_count: u32,
    pub today_total_minutes: u32,
    pub today_sessions: u32,
    pub intensity: u8,
}

fn close_session(state: &mut AppState) -> Option<SessionSummary> {
    let session = state.current_session.take()?;
    let last = session.recent_interactions.last()
        .copied()
        .unwrap_or(session.start_time);
    let duration_min = ((last - session.start_time) / 60000) as u32;

    let summary = SessionSummary {
        start_time: session.start_time,
        end_time: last,
        duration_minutes: duration_min,
        interaction_count: session.interaction_count,
        date: today_str(),
    };

    state.today.total_minutes += duration_min;
    state.today.sessions += 1;
    state.last_session_summary = Some(summary.clone());

    // Persist to history.
    state.history.push(summary.clone());
    if state.history.len() > MAX_HISTORY {
        state.history.remove(0);
    }

    Some(summary)
}

fn now_ms() -> i64 {
    Local::now().timestamp_millis()
}

fn today_str() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("breather-test-{}", nanos));
        fs::create_dir_all(&dir).ok();
        dir
    }

    #[test]
    fn first_interaction_creates_session() {
        let mgr = StateManager::with_data_dir(temp_dir());
        let (is_new, prev) = mgr.record_interaction();
        assert!(is_new);
        assert!(prev.is_none());
        let stats = mgr.get_stats().unwrap();
        assert_eq!(stats.interaction_count, 1);
    }

    #[test]
    fn rapid_interactions_stay_in_same_session() {
        let mgr = StateManager::with_data_dir(temp_dir());
        mgr.record_interaction();
        let (is_new, _) = mgr.record_interaction();
        assert!(!is_new);
        let (is_new, _) = mgr.record_interaction();
        assert!(!is_new);
        assert_eq!(mgr.get_stats().unwrap().interaction_count, 3);
    }

    #[test]
    fn stale_session_closes_and_starts_new() {
        let mgr = StateManager::with_data_dir(temp_dir());
        mgr.record_interaction();

        // Backdate the session so it looks stale.
        {
            let mut s = mgr.state.lock().unwrap();
            if let Some(ref mut session) = s.current_session {
                let old = now_ms() - STALE_MS - 1000;
                session.start_time = old;
                session.recent_interactions = vec![old];
            }
        }

        let (is_new, prev) = mgr.record_interaction();
        assert!(is_new, "should detect stale and start new session");
        assert!(prev.is_some(), "previous session should be summarized");
    }

    #[test]
    fn intensity_is_clamped() {
        let mgr = StateManager::with_data_dir(temp_dir());
        mgr.set_intensity(0);
        assert_eq!(mgr.state.lock().unwrap().intensity, 1);
        mgr.set_intensity(200);
        assert_eq!(mgr.state.lock().unwrap().intensity, 100);
        mgr.set_intensity(50);
        assert_eq!(mgr.state.lock().unwrap().intensity, 50);
    }

    #[test]
    fn today_total_is_zero_without_session() {
        let mgr = StateManager::with_data_dir(temp_dir());
        assert_eq!(mgr.get_today_total(), 0);
    }

    #[test]
    fn close_current_session_finalizes() {
        let mgr = StateManager::with_data_dir(temp_dir());
        mgr.record_interaction();
        assert!(mgr.get_stats().is_some());
        mgr.close_current_session();
        assert!(mgr.get_stats().is_none(), "session should be gone");
        assert_eq!(mgr.get_history().len(), 1, "history should have summary");
    }

    #[test]
    fn save_uses_atomic_rename() {
        let dir = temp_dir();
        let mgr = StateManager::with_data_dir(dir.clone());
        mgr.record_interaction();
        mgr.save();
        let state_file = dir.join("state.json");
        let tmp_file = dir.join("state.json.tmp");
        assert!(state_file.exists(), "state.json should exist");
        assert!(!tmp_file.exists(), "tmp file should be gone after rename");
    }

    #[test]
    fn save_and_reload_roundtrip() {
        let dir = temp_dir();
        {
            let mgr = StateManager::with_data_dir(dir.clone());
            mgr.record_interaction();
            mgr.record_interaction();
            mgr.set_intensity(75);
            mgr.save();
        }
        let mgr2 = StateManager::with_data_dir(dir);
        assert_eq!(mgr2.state.lock().unwrap().intensity, 75);
        assert_eq!(mgr2.get_stats().unwrap().interaction_count, 2);
    }

    #[test]
    fn check_and_close_stale_closes_only_stale() {
        let mgr = StateManager::with_data_dir(temp_dir());
        mgr.record_interaction();

        // Fresh session: should not close.
        assert!(mgr.check_and_close_stale().is_none());
        assert!(mgr.get_stats().is_some());

        // Make it stale.
        {
            let mut s = mgr.state.lock().unwrap();
            if let Some(ref mut session) = s.current_session {
                let old = now_ms() - STALE_MS - 1000;
                session.recent_interactions = vec![old];
            }
        }
        assert!(mgr.check_and_close_stale().is_some());
        assert!(mgr.get_stats().is_none());
    }

    #[test]
    fn get_stats_caps_duration_when_stale() {
        let mgr = StateManager::with_data_dir(temp_dir());
        mgr.record_interaction();
        // Backdate so the computed duration would be large,
        // but the cap should stop it at the last interaction.
        {
            let mut s = mgr.state.lock().unwrap();
            if let Some(ref mut session) = s.current_session {
                let start = now_ms() - 2 * STALE_MS;
                let last = now_ms() - STALE_MS - 1000;
                session.start_time = start;
                session.recent_interactions = vec![last];
            }
        }
        let stats = mgr.get_stats().unwrap();
        // duration should be capped at (last - start), not (now - start).
        let expected_max = (STALE_MS / 60000) as u32 + 1;
        assert!(stats.duration_minutes <= expected_max,
            "duration {} should be capped near {}", stats.duration_minutes, expected_max);
    }
}
