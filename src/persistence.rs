//! Local JSON persistence for cross-session progress tracking.

use chrono::TimeZone;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::{Level, RoundLength, TrainingMode};
use crate::finger::{Finger, TypingEngine};

/// One completed practice session, frozen at the moment it ended.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Unix timestamp (seconds) of when the session ended.
    pub timestamp: u64,
    /// Overall words-per-minute for the session (0.0 if not computed).
    pub wpm: f64,
    /// Finger key (see `Finger::as_key`) -> accuracy percentage (0-100) for
    /// that session.
    pub finger_accuracy: HashMap<String, f64>,

    /// Session duration in seconds.
    #[serde(default)]
    pub session_duration_secs: Option<f64>,
    /// Difficulty level.
    #[serde(default)]
    pub level: Option<String>,
    /// Training mode.
    #[serde(default)]
    pub training_mode: Option<String>,
    /// Round length setting.
    #[serde(default)]
    pub round_length: Option<String>,
    /// Overall accuracy percentage (0-100).
    #[serde(default)]
    pub overall_accuracy: Option<f64>,
    /// Total keystrokes typed in the session.
    #[serde(default)]
    pub total_keystrokes: Option<u64>,
    /// Correct keystrokes typed in the session.
    #[serde(default)]
    pub correct_keystrokes: Option<u64>,
    /// Error count in the session.
    #[serde(default)]
    pub error_count: Option<u64>,

    /// Finger key -> total keystrokes on that finger for this session.
    #[serde(default)]
    pub finger_keystrokes: HashMap<String, u64>,
    /// Finger key -> error count on that finger for this session.
    #[serde(default)]
    pub finger_errors: HashMap<String, u64>,

    /// Top mistyped character pairs formatted as "expected->actual:count;..."
    #[serde(default)]
    pub top_mistakes: Option<String>,
}

impl SessionSnapshot {
    /// Builds a snapshot from the engine's current in-memory stats, stamped
    /// with the current wall-clock time.
    pub fn from_engine(engine: &TypingEngine, wpm: f64) -> Self {
        SessionSnapshot {
            timestamp: current_unix_timestamp(),
            wpm,
            finger_accuracy: engine.accuracy_snapshot(),
            session_duration_secs: None,
            level: None,
            training_mode: None,
            round_length: None,
            overall_accuracy: None,
            total_keystrokes: None,
            correct_keystrokes: None,
            error_count: None,
            finger_keystrokes: HashMap::new(),
            finger_errors: HashMap::new(),
            top_mistakes: None,
        }
    }
}

pub fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// The full persisted history: every session the user has ever completed,
/// in chronological order (oldest first).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserHistory {
    pub sessions: Vec<SessionSnapshot>,
}

impl UserHistory {
    /// Appends a session and returns `self` for convenient chaining.
    pub fn push(&mut self, snapshot: SessionSnapshot) {
        self.sessions.push(snapshot);
    }

    /// Extracts the accuracy time series for one finger across all
    /// recorded sessions, as `(session_index, accuracy_pct)` pairs suitable
    /// for feeding straight into a ratatui `Dataset`.
    pub fn accuracy_series(&self, finger_key: &str) -> Vec<(f64, f64)> {
        self.sessions
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                s.finger_accuracy
                    .get(finger_key)
                    .map(|acc| (i as f64, *acc))
            })
            .collect()
    }

    /// WPM across every recorded session, as `(session_index, wpm)` pairs
    /// for the dashboard's "WPM trend" tab.
    pub fn wpm_series(&self) -> Vec<(f64, f64)> {
        self.sessions
            .iter()
            .enumerate()
            .map(|(i, s)| (i as f64, s.wpm))
            .collect()
    }

    /// Highest WPM ever recorded, or `0.0` if there's no history yet.
    pub fn best_wpm(&self) -> f64 {
        self.sessions.iter().map(|s| s.wpm).fold(0.0_f64, f64::max)
    }

    /// Counts consecutive most-recent sessions (walking backward from the
    /// latest) whose average accuracy across all fingers is at least
    /// `threshold_pct`. Stops at the first session that falls short.
    pub fn current_streak(&self, threshold_pct: f64) -> usize {
        let mut streak = 0;
        for session in self.sessions.iter().rev() {
            if session.finger_accuracy.is_empty() {
                break;
            }
            let avg = session.finger_accuracy.values().sum::<f64>()
                / session.finger_accuracy.len() as f64;
            if avg >= threshold_pct {
                streak += 1;
            } else {
                break;
            }
        }
        streak
    }
}

/// Cross-session app preferences: the Setup screen's chosen `Level` /
/// `TrainingMode` / `RoundLength`, plus whether the user has opted in to
/// progress tracking. Deliberately kept in its own file rather than folded
/// into `UserHistory`, so wiping history doesn't also reset preferences (or
/// vice versa).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub level: Level,
    pub training_mode: TrainingMode,
    pub round_length: RoundLength,
    /// `None` until the user has answered the one-time tracking-consent
    /// prompt; see `app::Mode::Consent`.
    pub track_progress: Option<bool>,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            level: Level::Beginner,
            training_mode: TrainingMode::FocusWeakest,
            round_length: RoundLength::Medium,
            track_progress: None,
        }
    }
}

/// Default on-disk location: `<config_dir>/fingertrack/settings.json`.
/// Mirrors `default_history_path`'s fallback for environments with no
/// resolvable OS config dir.
pub fn default_settings_path() -> PathBuf {
    if let Some(mut dir) = dirs::config_dir() {
        dir.push("fingertrack");
        dir.push("settings.json");
        dir
    } else {
        PathBuf::from("./settings.json")
    }
}

/// Loads settings from `path`. A missing file is treated as "first launch"
/// and returns `AppSettings::default()` (which has `track_progress: None`,
/// so the caller knows to show the consent prompt) rather than an error.
pub fn load_settings(path: &Path) -> io::Result<AppSettings> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let settings: AppSettings = serde_json::from_str(&contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(settings)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(AppSettings::default()),
        Err(e) => Err(e),
    }
}

/// Writes `settings` to `path` as pretty-printed JSON, creating parent
/// directories as needed. Same write-to-temp-then-rename pattern as
/// `save_history`, for the same crash-safety reason.
pub fn save_settings(path: &Path, settings: &AppSettings) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, json)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Default on-disk location for a CSV export: alongside `history.json`, at
/// `<config_dir>/fingertrack/history_export.csv`. Re-exporting overwrites
/// the previous export rather than accumulating timestamped files, since
/// it's always a full dump of current history rather than an incremental
/// log.
pub fn default_csv_export_path() -> PathBuf {
    PathBuf::from("./history_export.csv")
}

/// Writes `history` out as CSV: one header row, then one row per session
/// with its human-readable date/time, timestamp, WPM, and one accuracy column
/// per finger (0-100, blank if that finger had no recorded keystrokes that session).
/// This is a one-way export for sharing/analysis outside the app - there's no
/// matching `import_history_csv`, since `history.json` remains the
/// authoritative source the app itself reads and writes.
pub fn export_history_csv(history: &UserHistory, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut out = String::new();
    let mut header: Vec<String> = vec![
        "date_time".to_string(),
        "timestamp".to_string(),
        "session_duration_secs".to_string(),
        "level".to_string(),
        "training_mode".to_string(),
        "round_length".to_string(),
        "wpm".to_string(),
        "overall_accuracy".to_string(),
        "total_keystrokes".to_string(),
        "correct_keystrokes".to_string(),
        "error_count".to_string(),
    ];

    for finger in Finger::ALL {
        let key = finger.as_key();
        header.push(key.to_string());
        header.push(format!("{key}_count"));
        header.push(format!("{key}_errors"));
    }
    header.push("top_mistakes".to_string());

    out.push_str(&header.join(","));
    out.push('\n');

    for session in &history.sessions {
        let date_time = match chrono::Local.timestamp_opt(session.timestamp as i64, 0) {
            chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            chrono::LocalResult::Ambiguous(dt, _) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            chrono::LocalResult::None => "unknown".to_string(),
        };

        let mut fields: Vec<String> = vec![
            date_time,
            session.timestamp.to_string(),
            session.session_duration_secs.map_or(String::new(), |v| format!("{:.1}", v)),
            session.level.clone().unwrap_or_default(),
            session.training_mode.clone().unwrap_or_default(),
            session.round_length.clone().unwrap_or_default(),
            format!("{:.2}", session.wpm),
            session.overall_accuracy.map_or(String::new(), |v| format!("{:.2}", v)),
            session.total_keystrokes.map_or(String::new(), |v| v.to_string()),
            session.correct_keystrokes.map_or(String::new(), |v| v.to_string()),
            session.error_count.map_or(String::new(), |v| v.to_string()),
        ];

        for finger in Finger::ALL {
            let key = finger.as_key();
            match session.finger_accuracy.get(key) {
                Some(acc) => fields.push(format!("{acc:.2}")),
                None => fields.push(String::new()),
            }
            match session.finger_keystrokes.get(key) {
                Some(cnt) => fields.push(cnt.to_string()),
                None => fields.push(String::new()),
            }
            match session.finger_errors.get(key) {
                Some(errs) => fields.push(errs.to_string()),
                None => fields.push(String::new()),
            }
        }

        fields.push(session.top_mistakes.clone().unwrap_or_default());

        out.push_str(&fields.join(","));
        out.push('\n');
    }

    fs::write(path, out)
}

/// Default on-disk location: `<config_dir>/fingertrack/history.json`, e.g.
/// `~/.config/fingertrack/history.json` on Linux. Falls back to
/// `./history.json` in the current directory if the OS config dir can't be
/// determined (e.g. some minimal containers).
pub fn default_history_path() -> PathBuf {
    if let Some(mut dir) = dirs::config_dir() {
        dir.push("fingertrack");
        dir.push("history.json");
        dir
    } else {
        PathBuf::from("./history.json")
    }
}

/// Loads history from `path`. A missing file is treated as "no history
/// yet" and returns an empty [`UserHistory`] rather than an error, since
/// that's the expected state on first run.
pub fn load_history(path: &Path) -> io::Result<UserHistory> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let history: UserHistory = serde_json::from_str(&contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(history)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(UserHistory::default()),
        Err(e) => Err(e),
    }
}

/// Writes `history` to `path` as pretty-printed JSON, creating parent
/// directories as needed. Uses a write-to-temp-then-rename pattern so a
/// crash mid-write can't corrupt existing history.
pub fn save_history(path: &Path, history: &UserHistory) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(history)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, json)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}
