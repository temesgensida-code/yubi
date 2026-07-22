//! Application state.
//!
//! Four modes now:
//! - `Consent`: shown once, on first launch, asking whether to persist
//!   progress to disk at all. Skipped on every later launch once answered.
//! - `Setup`: choose a difficulty level, training mode, and round length
//!   before a round. Choices persist across restarts (see
//!   `persistence::AppSettings`).
//! - `Typing`: the user is actively typing a generated practice string.
//! - `Dashboard`: shown after a session ends, lets the user browse
//!   per-finger historical accuracy/WPM/mistakes via sidebar + tabs (mouse
//!   click or ↑/↓ to select a finger), and export history to CSV.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

use crate::finger::{Finger, TypingEngine};
use crate::generator;
use crate::paragraphs;
use crate::persistence::{self, AppSettings, SessionSnapshot, UserHistory};

/// How many WPM samples to keep for the live sparkline (roughly this many
/// seconds of history, since we sample about once a second).
const WPM_SAMPLE_CAPACITY: usize = 40;
/// How long a wrong keystroke stays "flash" red before settling to a
/// duller, permanent red.
pub const ERROR_FLASH_DURATION: Duration = Duration::from_millis(350);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// One-time prompt (shown only while `track_progress` has never been
    /// answered) asking whether to save session history to disk at all.
    Consent,
    Setup,
    Typing,
    Dashboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Level {
    Beginner,
    Intermediate,
    Advanced,
}

impl Level {
    pub const ALL: [Level; 3] = [Level::Beginner, Level::Intermediate, Level::Advanced];

    pub fn label(&self) -> &'static str {
        match self {
            Level::Beginner => "Beginner — short words",
            Level::Intermediate => "Intermediate — short paragraph",
            Level::Advanced => "Advanced — full paragraph",
        }
    }

    fn index(&self) -> usize {
        Level::ALL.iter().position(|l| l == self).unwrap_or(0)
    }

    pub fn next(&self) -> Level {
        Level::ALL[(self.index() + 1) % Level::ALL.len()]
    }

    pub fn prev(&self) -> Level {
        Level::ALL[(self.index() + Level::ALL.len() - 1) % Level::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainingMode {
    FocusWeakest,
    Random,
}

impl TrainingMode {
    pub fn label(&self) -> &'static str {
        match self {
            TrainingMode::FocusWeakest => "Target weakest finger",
            TrainingMode::Random => "Random",
        }
    }

    pub fn toggle(&self) -> TrainingMode {
        match self {
            TrainingMode::FocusWeakest => TrainingMode::Random,
            TrainingMode::Random => TrainingMode::FocusWeakest,
        }
    }
}

/// How long a round's practice text is, in whole words. Applies uniformly
/// across every `Level`: it's the knob for "how much text", while `Level`
/// stays the knob for "how simple/complex the text is" (Beginner strips
/// punctuation and lowercases; Intermediate/Advanced keep the paragraph
/// bank's natural casing and punctuation). The two settings are stored and
/// changed independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundLength {
    Short,
    Medium,
    Long,
}

impl RoundLength {
    pub const ALL: [RoundLength; 3] = [RoundLength::Short, RoundLength::Medium, RoundLength::Long];

    pub fn label(&self) -> &'static str {
        match self {
            RoundLength::Short => "Short — 8 words",
            RoundLength::Medium => "Medium — 16 words",
            RoundLength::Long => "Long — 28 words",
        }
    }

    /// Target word count fed to the generator (Beginner) or used to cap the
    /// paragraph bank text (Intermediate/Advanced).
    pub fn word_count(&self) -> usize {
        match self {
            RoundLength::Short => 8,
            RoundLength::Medium => 16,
            RoundLength::Long => 28,
        }
    }

    fn index(&self) -> usize {
        RoundLength::ALL.iter().position(|l| l == self).unwrap_or(0)
    }

    pub fn next(&self) -> RoundLength {
        RoundLength::ALL[(self.index() + 1) % RoundLength::ALL.len()]
    }

    pub fn prev(&self) -> RoundLength {
        RoundLength::ALL[(self.index() + RoundLength::ALL.len() - 1) % RoundLength::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupField {
    Level,
    Training,
    Length,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardTab {
    Accuracy,
    Wpm,
    Mistakes,
}

impl DashboardTab {
    pub const ALL: [DashboardTab; 3] = [
        DashboardTab::Accuracy,
        DashboardTab::Wpm,
        DashboardTab::Mistakes,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            DashboardTab::Accuracy => "Accuracy Trend",
            DashboardTab::Wpm => "WPM Trend",
            DashboardTab::Mistakes => "Mistake Matrix",
        }
    }
}

/// One committed character in the typing buffer. Remembers whether it was
/// correct *and* when it was typed, so the UI can show a brief "flash" on
/// a fresh mistake before it settles into a duller, permanent error color.
#[derive(Debug, Clone, Copy)]
pub struct TypedChar {
    pub expected: char,
    pub actual: char,
    pub at: Instant,
}

impl TypedChar {
    pub fn is_correct(&self) -> bool {
        self.expected == self.actual
    }
}

pub struct App {
    pub engine: TypingEngine,
    pub history: UserHistory,
    pub history_path: PathBuf,
    pub settings_path: PathBuf,

    pub mode: Mode,
    pub should_quit: bool,

    pub level: Level,
    pub training_mode: TrainingMode,
    pub round_length: RoundLength,
    pub setup_focus: SetupField,

    /// `None` until the user answers the one-time consent prompt (see
    /// `Mode::Consent`). `Some(true)` means sessions are appended to
    /// `history` and saved to disk; `Some(false)` means rounds still work
    /// (live stats, dashboard) but nothing is written to disk and CSV
    /// export is unavailable.
    pub track_progress: Option<bool>,

    pub target: Vec<char>,
    pub typed: Vec<TypedChar>,
    pub session_start: Instant,
    pub last_session_wpm: f64,
    pub last_session_errors: u64,

    pub finger_list_state: ListState,
    pub dashboard_tab: DashboardTab,
    /// The screen area the finger sidebar's `List` was last drawn into,
    /// recorded by `ui::draw_finger_sidebar` each frame so mouse clicks can
    /// be hit-tested against it in `handle_sidebar_click`.
    pub sidebar_area: Rect,

    /// Rolling WPM samples for the live sparkline during a typing round.
    pub wpm_samples: VecDeque<u64>,
    last_wpm_sample_at: Instant,

    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let history_path = persistence::default_history_path();
        let settings_path = persistence::default_settings_path();
        let settings = persistence::load_settings(&settings_path).unwrap_or_default();
        // History is loaded regardless of the tracking preference so a user
        // who already has data (or flips tracking back on later) can still
        // see it; `track_progress` only gates whether *new* sessions get
        // appended and saved (see `finish_session`).
        let history = persistence::load_history(&history_path).unwrap_or_default();
        let engine = TypingEngine::new();

        let mut finger_list_state = ListState::default();
        finger_list_state.select(Some(0));

        // Only ask once: if `track_progress` was already answered on a
        // previous launch, skip straight past the consent screen.
        let mode = if settings.track_progress.is_none() {
            Mode::Consent
        } else {
            Mode::Setup
        };

        App {
            engine,
            history,
            history_path,
            settings_path,
            mode,
            should_quit: false,
            level: settings.level,
            training_mode: settings.training_mode,
            round_length: settings.round_length,
            setup_focus: SetupField::Level,
            track_progress: settings.track_progress,
            target: Vec::new(),
            typed: Vec::new(),
            session_start: Instant::now(),
            last_session_wpm: 0.0,
            last_session_errors: 0,
            finger_list_state,
            dashboard_tab: DashboardTab::Accuracy,
            sidebar_area: Rect::default(),
            wpm_samples: VecDeque::with_capacity(WPM_SAMPLE_CAPACITY),
            last_wpm_sample_at: Instant::now(),
            status_message: None,
        }
    }

    // ---- Settings persistence ---------------------------------------

    fn save_settings(&self) {
        let settings = AppSettings {
            level: self.level,
            training_mode: self.training_mode,
            round_length: self.round_length,
            track_progress: self.track_progress,
        };
        // Best-effort: a failed settings write shouldn't crash a typing
        // round, so errors are swallowed here (unlike history saves, which
        // do report failures via `status_message` since losing recorded
        // progress is more surprising to a user than losing a UI setting).
        let _ = persistence::save_settings(&self.settings_path, &settings);
    }

    /// Answers the one-time "save my progress?" prompt and moves on to
    /// Setup. Called from `Mode::Consent`.
    pub fn set_tracking_consent(&mut self, enabled: bool) {
        self.track_progress = Some(enabled);
        self.save_settings();
        self.mode = Mode::Setup;
        self.status_message = Some(if enabled {
            "Progress tracking on — sessions will be saved locally.".to_string()
        } else {
            "Progress tracking off — sessions won't be saved. Change this any time from the settings file.".to_string()
        });
    }

    // ---- Setup screen ----------------------------------------------

    pub fn setup_toggle_focus(&mut self) {
        self.setup_focus = match self.setup_focus {
            SetupField::Level => SetupField::Training,
            SetupField::Training => SetupField::Length,
            SetupField::Length => SetupField::Level,
        };
    }

    pub fn setup_cycle_left(&mut self) {
        match self.setup_focus {
            SetupField::Level => self.level = self.level.prev(),
            SetupField::Training => self.training_mode = self.training_mode.toggle(),
            SetupField::Length => self.round_length = self.round_length.prev(),
        }
        // Persist immediately (not just at round-start) so a value chosen
        // and then abandoned via Esc is still remembered next launch.
        self.save_settings();
    }

    pub fn setup_cycle_right(&mut self) {
        match self.setup_focus {
            SetupField::Level => self.level = self.level.next(),
            SetupField::Training => self.training_mode = self.training_mode.toggle(),
            SetupField::Length => self.round_length = self.round_length.next(),
        }
        self.save_settings();
    }

    // ---- Typing round -----------------------------------------------

    /// Builds fresh practice text from the current `level` + `training_mode`
    /// and enters `Mode::Typing`. Called from the Setup screen (Enter) and
    /// from the Dashboard's "practice again with same settings" flow.
    pub fn start_new_round(&mut self) {
        self.target = build_target_text(&self.engine, self.level, self.training_mode, self.round_length);
        self.typed.clear();
        self.session_start = Instant::now();
        self.wpm_samples.clear();
        self.last_wpm_sample_at = Instant::now();
        self.mode = Mode::Typing;
        self.status_message = None;
    }

    /// Handles one printable character typed by the user during a typing
    /// round. Returns `true` if this keystroke completed the round.
    pub fn handle_char_input(&mut self, actual: char) -> bool {
        let idx = self.typed.len();
        let Some(expected) = self.target.get(idx).copied() else {
            return true; // already complete, ignore stray input
        };

        self.engine.record_keystroke(expected, actual);
        self.typed.push(TypedChar {
            expected,
            actual,
            at: Instant::now(),
        });

        if self.typed.len() >= self.target.len() {
            self.finish_session();
            true
        } else {
            false
        }
    }

    /// Removes the last committed character, letting the user correct a
    /// mistake mid-round. This does not retroactively "unrecord" the
    /// keystroke from engine statistics - the mistake genuinely happened
    /// and stays in the analytics; only the visible buffer rewinds.
    pub fn handle_backspace(&mut self) {
        self.typed.pop();
    }

    /// Called once per event-loop iteration regardless of whether a key was
    /// pressed, so timing-based effects (the live WPM sparkline, the error
    /// flash fade) keep animating even when the user pauses.
    pub fn tick(&mut self) {
        if self.mode != Mode::Typing {
            return;
        }
        if self.last_wpm_sample_at.elapsed() >= Duration::from_secs(1) {
            self.last_wpm_sample_at = Instant::now();
            let elapsed_minutes = self.session_start.elapsed().as_secs_f64() / 60.0;
            let wpm = if elapsed_minutes > 0.0 {
                (self.typed.len() as f64 / 5.0) / elapsed_minutes
            } else {
                0.0
            };
            if self.wpm_samples.len() >= WPM_SAMPLE_CAPACITY {
                self.wpm_samples.pop_front();
            }
            self.wpm_samples.push_back(wpm.round().max(0.0) as u64);
        }
    }

    /// Finalizes the current round: computes WPM, snapshots per-finger
    /// accuracy, appends it to history, and persists to disk.
    fn finish_session(&mut self) {
        let elapsed_minutes = self.session_start.elapsed().as_secs_f64() / 60.0;
        let words_typed = self.target.len() as f64 / 5.0; // standard "word" = 5 chars
        let wpm = if elapsed_minutes > 0.0 {
            words_typed / elapsed_minutes
        } else {
            0.0
        };
        let errors = self.typed.iter().filter(|t| !t.is_correct()).count() as u64;

        self.last_session_wpm = wpm;
        self.last_session_errors = errors;

        if self.track_progress == Some(true) {
            let snapshot = SessionSnapshot::from_engine(&self.engine, wpm);
            self.history.push(snapshot);

            let save_status = match persistence::save_history(&self.history_path, &self.history) {
                Ok(()) => format!("Session saved to {}", self.history_path.display()),
                Err(e) => format!("Failed to save history: {}", e),
            };

            let csv_path = persistence::default_csv_export_path();
            match persistence::export_history_csv(&self.history, &csv_path) {
                Ok(()) => {
                    self.status_message = Some(format!("{} & CSV updated at {}", save_status, csv_path.display()));
                }
                Err(e) => {
                    self.status_message = Some(format!("{} (Failed to auto-export CSV: {})", save_status, e));
                }
            }
        } else {
            self.status_message =
                Some("Session complete. Progress tracking is off, so it wasn't saved.".to_string());
        }

        self.mode = Mode::Dashboard;
    }

    /// Exports the persisted history to a CSV file (one row per session:
    /// timestamp, WPM, then one accuracy column per finger). Only available
    /// when the user opted in to progress tracking, since otherwise
    /// `history` reflects a previous opt-in period rather than what the
    /// user asked to be tracked right now.
    pub fn export_history_to_csv(&mut self) {
        if self.track_progress != Some(true) {
            self.status_message = Some(
                "Progress tracking is off, so there's nothing tracked to export.".to_string(),
            );
            return;
        }
        if self.history.sessions.is_empty() {
            self.status_message = Some("No sessions recorded yet — nothing to export.".to_string());
            return;
        }
        let path = persistence::default_csv_export_path();
        match persistence::export_history_csv(&self.history, &path) {
            Ok(()) => {
                self.status_message = Some(format!(
                    "Exported {} session(s) to {}",
                    self.history.sessions.len(),
                    path.display()
                ));
            }
            Err(e) => {
                self.status_message = Some(format!("Failed to export CSV: {e}"));
            }
        }
    }

    // ---- Dashboard ----------------------------------------------------

    /// All ten fingers, sorted worst-accuracy-first. Both the sidebar list
    /// and `selected_finger` walk this same order so "the finger currently
    /// highlighted" always matches "the row currently drawn as selected".
    pub fn finger_display_order(&self) -> Vec<Finger> {
        let mut fingers = Finger::ALL.to_vec();
        fingers.sort_by(|a, b| {
            let acc_a = self
                .engine
                .stats
                .get(a)
                .map(|s| s.accuracy_pct())
                .unwrap_or(100.0);
            let acc_b = self
                .engine
                .stats
                .get(b)
                .map(|s| s.accuracy_pct())
                .unwrap_or(100.0);
            acc_a
                .partial_cmp(&acc_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        fingers
    }

    pub fn selected_finger(&self) -> Finger {
        let order = self.finger_display_order();
        let idx = self
            .finger_list_state
            .selected()
            .unwrap_or(0)
            .min(order.len().saturating_sub(1));
        order[idx]
    }

    pub fn select_next_finger(&mut self) {
        let len = Finger::ALL.len();
        let idx = self.finger_list_state.selected().unwrap_or(0);
        self.finger_list_state.select(Some((idx + 1) % len));
    }

    pub fn select_prev_finger(&mut self) {
        let len = Finger::ALL.len();
        let idx = self.finger_list_state.selected().unwrap_or(0);
        self.finger_list_state.select(Some((idx + len - 1) % len));
    }

    /// Handles a left-click at terminal coordinates `(col, row)` while the
    /// Dashboard is showing. If the click landed inside the finger
    /// sidebar's list rows (i.e. inside `sidebar_area`, excluding its
    /// rounded border), selects whichever finger is drawn at that row —
    /// the mouse equivalent of pressing Up/Down until it's highlighted.
    /// Clicks outside the sidebar (or before it's ever been drawn) are
    /// silently ignored.
    pub fn handle_sidebar_click(&mut self, col: u16, row: u16) {
        let area = self.sidebar_area;
        if area.width < 2 || area.height < 2 {
            return; // never drawn, or too small to have any inner rows
        }
        // `theme::block` draws a one-cell rounded border on every side, so
        // the list's actual rows start one cell in from the block's edge.
        let inner_x0 = area.x + 1;
        let inner_x1 = area.x + area.width - 1;
        let inner_y0 = area.y + 1;
        let inner_y1 = area.y + area.height - 1;
        if col < inner_x0 || col >= inner_x1 || row < inner_y0 || row >= inner_y1 {
            return;
        }
        let row_in_list = (row - inner_y0) as usize;
        let idx = self.finger_list_state.offset() + row_in_list;
        if idx < Finger::ALL.len() {
            self.finger_list_state.select(Some(idx));
        }
    }

    pub fn cycle_dashboard_tab(&mut self) {
        let idx = DashboardTab::ALL
            .iter()
            .position(|t| *t == self.dashboard_tab)
            .unwrap_or(0);
        self.dashboard_tab = DashboardTab::ALL[(idx + 1) % DashboardTab::ALL.len()];
    }
}

/// Builds the practice text for a round from the current difficulty level
/// and training mode.
///
/// - `Beginner` always uses the short, weak-finger-biased word generator
///   from `generator.rs` (works fine with zero history, since it falls
///   back to a random sample when no finger has been used yet).
/// - `Intermediate`/`Advanced` pull from the finger-tagged paragraph bank
///   in `paragraphs.rs`, shaped to length/complexity by level.
/// - In `TrainingMode::Random`, the weakest-finger lookup is skipped
///   entirely so word/paragraph choice doesn't favor any particular
///   finger.
/// - `round_length` controls how much text is produced (word count),
///   independently of `level`: `level` only shapes *how* the text looks
///   (Beginner strips punctuation and lowercases; Intermediate/Advanced
///   keep the paragraph bank's natural casing and punctuation).
fn build_target_text(
    engine: &TypingEngine,
    level: Level,
    mode: TrainingMode,
    round_length: RoundLength,
) -> Vec<char> {
    let target_finger = match mode {
        TrainingMode::FocusWeakest => engine.get_most_faulty_finger().map(|(f, _)| f),
        TrainingMode::Random => None,
    };
    let word_count = round_length.word_count();

    match level {
        Level::Beginner => {
            let text = generator::generate_for_finger(engine, target_finger, word_count);
            text.chars().collect()
        }
        Level::Intermediate | Level::Advanced => match target_finger {
            Some(f) => paragraphs::pick_for_finger(f, level, word_count),
            None => paragraphs::pick_random(level, word_count),
        },
    }
}
