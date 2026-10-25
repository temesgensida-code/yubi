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
use ratatui::widgets::{ListState, TableState};
use serde::{Deserialize, Serialize};

use crate::engine::latency::LatencyEngine;
use crate::engine::pacer::{GhostPacer, PacerMode};
use crate::finger::{Finger, KeyboardLayout, ThumbPreference, TypingEngine};
use crate::generator;
use crate::paragraphs;
use crate::persistence::{self, AppSettings, SessionSnapshot, UserHistory};
use crate::theme::{self, Theme, ThemeId};

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
    Debrief,
    Dashboard,
}

/// Instant debrief analytics captured at the conclusion of a practice round.
#[derive(Debug, Clone)]
pub struct RoundSummary {
    pub net_wpm: f64,
    pub raw_wpm: f64,
    pub accuracy_pct: f64,
    pub consistency_pct: f64,
    pub duration_secs: f64,
    pub total_keystrokes: u64,
    pub errors: u64,
    pub velocity_timeline: Vec<(f64, f64)>,        // (timestamp_secs, instant_wpm)
    pub error_points: Vec<(f64, f64)>,             // (timestamp_secs, instant_wpm)
    pub slowest_bigrams: Vec<((char, char), f64)>, // (bigram, avg_latency_ms)
    pub mistake_counts: Vec<((char, char), u64)>,  // top mistake substitutions
    #[allow(dead_code)]
    pub target_text: Vec<char>,                    // retained for instant [r] retry
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
    BigramLadder,
    CodeMode,
}

impl TrainingMode {
    pub const ALL: [TrainingMode; 4] = [
        TrainingMode::FocusWeakest,
        TrainingMode::Random,
        TrainingMode::BigramLadder,
        TrainingMode::CodeMode,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            TrainingMode::FocusWeakest => "Target weakest finger",
            TrainingMode::Random => "Random distribution",
            TrainingMode::BigramLadder => "Bigram hesitation ladder",
            TrainingMode::CodeMode => "Developer syntax mode",
        }
    }

    fn index(&self) -> usize {
        TrainingMode::ALL.iter().position(|m| m == self).unwrap_or(0)
    }

    pub fn next(&self) -> TrainingMode {
        TrainingMode::ALL[(self.index() + 1) % TrainingMode::ALL.len()]
    }

    pub fn prev(&self) -> TrainingMode {
        TrainingMode::ALL[(self.index() + TrainingMode::ALL.len() - 1) % TrainingMode::ALL.len()]
    }

    #[allow(dead_code)]
    pub fn toggle(&self) -> TrainingMode {
        self.next()
    }
}

/// How long a round's practice text is, in whole words. Applies uniformly
/// across every `Level`: it's the knob for "how much text", while `Level`
/// stays the knob for "how simple/complex the text is" (Beginner strips
/// punctuation and lowercases; Intermediate/Advanced keep the paragraph
/// bank's natural casing and punctuation). The two settings are stored and
/// changed independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RoundLength {
    Burst5,
    Words10,
    #[serde(alias = "Short")]
    Words15,
    Words20,
    #[default]
    #[serde(alias = "Medium")]
    Words30,
    Words40,
    #[serde(alias = "Long")]
    Words50,
    Words60,
    Sprint15,
    Sprint30,
    Sprint60,
}

impl RoundLength {
    pub const ALL: [RoundLength; 11] = [
        RoundLength::Burst5,
        RoundLength::Words10,
        RoundLength::Words15,
        RoundLength::Words20,
        RoundLength::Words30,
        RoundLength::Words40,
        RoundLength::Words50,
        RoundLength::Words60,
        RoundLength::Sprint15,
        RoundLength::Sprint30,
        RoundLength::Sprint60,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            RoundLength::Burst5 => "Burst — 5 words",
            RoundLength::Words10 => "Quick — 10 words",
            RoundLength::Words15 => "Short — 15 words",
            RoundLength::Words20 => "Standard — 20 words",
            RoundLength::Words30 => "Medium — 30 words",
            RoundLength::Words40 => "Extended — 40 words",
            RoundLength::Words50 => "Long — 50 words",
            RoundLength::Words60 => "Full — 60 words",
            RoundLength::Sprint15 => "Sprint — 15 seconds",
            RoundLength::Sprint30 => "Sprint — 30 seconds",
            RoundLength::Sprint60 => "Sprint — 60 seconds",
        }
    }

    /// Target word count fed to the generator (Beginner) or used to cap the
    /// paragraph bank text (Intermediate/Advanced).
    pub fn word_count(&self) -> usize {
        match self {
            RoundLength::Burst5 => 5,
            RoundLength::Words10 => 10,
            RoundLength::Words15 => 15,
            RoundLength::Words20 => 20,
            RoundLength::Words30 => 30,
            RoundLength::Words40 => 40,
            RoundLength::Words50 => 50,
            RoundLength::Words60 => 60,
            RoundLength::Sprint15 => 30,
            RoundLength::Sprint30 => 60,
            RoundLength::Sprint60 => 100,
        }
    }

    /// Fixed time limit in seconds for timed sprints, or `None` for word rounds.
    pub fn timed_duration_secs(&self) -> Option<f64> {
        match self {
            RoundLength::Sprint15 => Some(15.0),
            RoundLength::Sprint30 => Some(30.0),
            RoundLength::Sprint60 => Some(60.0),
            _ => None,
        }
    }

    pub fn is_timed_sprint(&self) -> bool {
        self.timed_duration_secs().is_some()
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
    Layout,
    Thumb,
    Pacer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeFilter {
    #[default]
    All,
    Last7Days,
    Last30Days,
}

impl TimeFilter {
    pub const ALL: [TimeFilter; 3] = [
        TimeFilter::All,
        TimeFilter::Last7Days,
        TimeFilter::Last30Days,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            TimeFilter::All => "All Time",
            TimeFilter::Last7Days => "Last 7 Days",
            TimeFilter::Last30Days => "Last 30 Days",
        }
    }

    pub fn next(&self) -> TimeFilter {
        let idx = TimeFilter::ALL.iter().position(|f| f == self).unwrap_or(0);
        TimeFilter::ALL[(idx + 1) % TimeFilter::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelFilter {
    #[default]
    All,
    Beginner,
    Intermediate,
    Advanced,
}

impl LevelFilter {
    pub const ALL: [LevelFilter; 4] = [
        LevelFilter::All,
        LevelFilter::Beginner,
        LevelFilter::Intermediate,
        LevelFilter::Advanced,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            LevelFilter::All => "All",
            LevelFilter::Beginner => "Beginner",
            LevelFilter::Intermediate => "Intermediate",
            LevelFilter::Advanced => "Advanced",
        }
    }

    pub fn next(&self) -> LevelFilter {
        let idx = LevelFilter::ALL.iter().position(|f| f == self).unwrap_or(0);
        LevelFilter::ALL[(idx + 1) % LevelFilter::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModeFilter {
    #[default]
    All,
    Targeted,
    Random,
}

impl ModeFilter {
    pub const ALL: [ModeFilter; 3] = [
        ModeFilter::All,
        ModeFilter::Targeted,
        ModeFilter::Random,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ModeFilter::All => "All",
            ModeFilter::Targeted => "Targeted",
            ModeFilter::Random => "Random",
        }
    }

    pub fn next(&self) -> ModeFilter {
        let idx = ModeFilter::ALL.iter().position(|f| f == self).unwrap_or(0);
        ModeFilter::ALL[(idx + 1) % ModeFilter::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeatmapMode {
    #[default]
    FingerAccuracy,
    KeyErrors,
    KeystrokeLoad,
}

impl HeatmapMode {
    pub const ALL: [HeatmapMode; 3] = [
        HeatmapMode::FingerAccuracy,
        HeatmapMode::KeyErrors,
        HeatmapMode::KeystrokeLoad,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            HeatmapMode::FingerAccuracy => "Finger Accuracy",
            HeatmapMode::KeyErrors => "Key Errors",
            HeatmapMode::KeystrokeLoad => "Keystroke Load",
        }
    }

    pub fn next(&self) -> HeatmapMode {
        let idx = HeatmapMode::ALL.iter().position(|m| m == self).unwrap_or(0);
        HeatmapMode::ALL[(idx + 1) % HeatmapMode::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashboardTab {
    #[default]
    Trends,           // [1] Trends & Progress
    KeyboardHeatmap,  // [2] Keyboard Heatmap
    FingerLoad,       // [3] Finger Load
    LevelComparison,  // [4] Level Comparison
    SessionHistory,   // [5] Session History
}

impl DashboardTab {
    pub const ALL: [DashboardTab; 5] = [
        DashboardTab::Trends,
        DashboardTab::KeyboardHeatmap,
        DashboardTab::FingerLoad,
        DashboardTab::LevelComparison,
        DashboardTab::SessionHistory,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            DashboardTab::Trends => "[1] Trends & Progress",
            DashboardTab::KeyboardHeatmap => "[2] Keyboard Heatmap",
            DashboardTab::FingerLoad => "[3] Finger Load",
            DashboardTab::LevelComparison => "[4] Level Comparison",
            DashboardTab::SessionHistory => "[5] Session History",
        }
    }

    pub fn index(&self) -> usize {
        DashboardTab::ALL.iter().position(|t| t == self).unwrap_or(0)
    }

    pub fn next(&self) -> DashboardTab {
        DashboardTab::ALL[(self.index() + 1) % DashboardTab::ALL.len()]
    }

    pub fn prev(&self) -> DashboardTab {
        DashboardTab::ALL[(self.index() + DashboardTab::ALL.len() - 1) % DashboardTab::ALL.len()]
    }
}

pub const HEATMAP_KEYS: &[&[char]] = &[
    &['`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '='],
    &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\\'],
    &['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\''],
    &['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'],
    &[' '],
];

#[derive(Debug, Clone)]
pub struct FilteredSummary {
    pub count: usize,
    pub total_count: usize,
    pub avg_wpm: f64,
    pub peak_wpm: f64,
    pub avg_accuracy: f64,
    pub total_keystrokes: u64,
    pub total_errors: u64,
    pub streak_90: usize,
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
    pub latency_engine: LatencyEngine,
    pub history: UserHistory,
    pub history_path: PathBuf,
    pub settings_path: PathBuf,

    pub mode: Mode,
    pub should_quit: bool,

    pub level: Level,
    pub training_mode: TrainingMode,
    pub round_length: RoundLength,
    pub layout: KeyboardLayout,
    pub thumb_preference: ThumbPreference,
    pub ghost_pacer: GhostPacer,
    pub setup_focus: SetupField,

    /// `None` until the user answers the one-time consent prompt (see
    /// `Mode::Consent`). `Some(true)` means sessions are appended to
    /// `history` and saved to disk; `Some(false)` means rounds still work
    /// (live stats, dashboard) but nothing is written to disk.
    pub track_progress: Option<bool>,

    pub target: Vec<char>,
    pub typed: Vec<TypedChar>,
    pub session_start: Instant,
    pub last_session_wpm: f64,
    pub last_session_errors: u64,

    pub finger_list_state: ListState,
    pub dashboard_tab: DashboardTab,
    pub time_filter: TimeFilter,
    pub level_filter: LevelFilter,
    pub mode_filter: ModeFilter,
    pub heatmap_mode: HeatmapMode,
    pub selected_heatmap_key: char,
    pub history_table_state: TableState,
    /// The screen area the finger sidebar's `List` was last drawn into,
    /// recorded by `ui::draw_finger_sidebar` each frame so mouse clicks can
    /// be hit-tested against it in `handle_sidebar_click`.
    pub sidebar_area: Rect,

    /// Rolling WPM samples for the live sparkline during a typing round.
    pub wpm_samples: VecDeque<u64>,
    last_wpm_sample_at: Instant,

    pub status_message: Option<String>,
    pub theme_id: ThemeId,
    pub keyboard_state: crate::ui::keyboard_live::LiveKeyboardState,
    last_tick_at: Instant,
    pub last_round_summary: Option<RoundSummary>,
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
        let layout = settings.layout.unwrap_or_default();
        let thumb_preference = settings.thumb_preference.unwrap_or_default();
        let engine = TypingEngine::new_with_options(layout, thumb_preference);

        let pacer_mode = settings.pacer_mode.unwrap_or_default();
        let target_wpm = match pacer_mode {
            PacerMode::Off => 0.0,
            PacerMode::Target(wpm) => wpm as f64,
            PacerMode::PersonalBest => {
                let pb = history.best_wpm();
                if pb > 0.0 { pb } else { 60.0 }
            }
        };
        let ghost_pacer = GhostPacer::new(pacer_mode, target_wpm);

        let mut finger_list_state = ListState::default();
        finger_list_state.select(Some(0));

        // Only ask once: if `track_progress` was already answered on a
        // previous launch, skip straight past the consent screen.
        let mode = if settings.track_progress.is_none() {
            Mode::Consent
        } else {
            Mode::Setup
        };

        let theme_id = settings.theme_id.unwrap_or_default();

        App {
            engine,
            latency_engine: LatencyEngine::new(),
            history,
            history_path,
            settings_path,
            mode,
            should_quit: false,
            level: settings.level,
            training_mode: settings.training_mode,
            round_length: settings.round_length,
            layout,
            thumb_preference,
            ghost_pacer,
            setup_focus: SetupField::Level,
            track_progress: settings.track_progress,
            target: Vec::new(),
            typed: Vec::new(),
            session_start: Instant::now(),
            last_session_wpm: 0.0,
            last_session_errors: 0,
            finger_list_state,
            dashboard_tab: DashboardTab::Trends,
            time_filter: TimeFilter::All,
            level_filter: LevelFilter::All,
            mode_filter: ModeFilter::All,
            heatmap_mode: HeatmapMode::FingerAccuracy,
            selected_heatmap_key: 'e',
            history_table_state: {
                let mut state = TableState::default();
                state.select(Some(0));
                state
            },
            sidebar_area: Rect::default(),
            wpm_samples: VecDeque::with_capacity(WPM_SAMPLE_CAPACITY),
            last_wpm_sample_at: Instant::now(),
            status_message: None,
            theme_id,
            keyboard_state: crate::ui::keyboard_live::LiveKeyboardState::new(),
            last_tick_at: Instant::now(),
            last_round_summary: None,
        }
    }

    pub fn cycle_theme(&mut self) {
        self.theme_id = self.theme_id.next();
        self.save_settings();
        self.status_message = Some(format!("Theme: {}", self.theme_id.label()));
    }

    pub fn theme(&self) -> Theme {
        theme::get_theme(self.theme_id)
    }

    // ---- Settings persistence ---------------------------------------

    fn save_settings(&self) {
        let settings = AppSettings {
            level: self.level,
            training_mode: self.training_mode,
            round_length: self.round_length,
            track_progress: self.track_progress,
            theme_id: Some(self.theme_id),
            thumb_preference: Some(self.thumb_preference),
            layout: Some(self.layout),
            pacer_mode: Some(self.ghost_pacer.mode),
        };
        // Best-effort: a failed settings write shouldn't crash a typing
        // round, so errors are swallowed here (unlike history saves, which
        // do report failures via `status_message` since losing recorded
        // progress is more surprising to a user than losing a UI setting).
        let _ = persistence::save_settings(&self.settings_path, &settings);
    }

    pub fn set_layout(&mut self, layout: KeyboardLayout) {
        self.layout = layout;
        self.engine.set_layout(layout);
        self.save_settings();
    }

    pub fn set_thumb_preference(&mut self, pref: ThumbPreference) {
        self.thumb_preference = pref;
        self.engine.set_thumb_preference(pref);
        self.save_settings();
    }

    pub fn resolve_pacer_wpm(&self, mode: PacerMode) -> f64 {
        match mode {
            PacerMode::Off => 0.0,
            PacerMode::Target(wpm) => wpm as f64,
            PacerMode::PersonalBest => {
                let pb = self.history.best_wpm();
                if pb > 0.0 { pb } else { 60.0 }
            }
        }
    }

    pub fn set_pacer_mode(&mut self, mode: PacerMode) {
        let target_wpm = self.resolve_pacer_wpm(mode);
        self.ghost_pacer = GhostPacer::new(mode, target_wpm);
        self.save_settings();
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
            SetupField::Length => SetupField::Layout,
            SetupField::Layout => SetupField::Thumb,
            SetupField::Thumb => SetupField::Pacer,
            SetupField::Pacer => SetupField::Level,
        };
    }

    pub fn setup_cycle_left(&mut self) {
        match self.setup_focus {
            SetupField::Level => self.level = self.level.prev(),
            SetupField::Training => self.training_mode = self.training_mode.prev(),
            SetupField::Length => self.round_length = self.round_length.prev(),
            SetupField::Layout => self.set_layout(self.layout.prev()),
            SetupField::Thumb => self.set_thumb_preference(self.thumb_preference.prev()),
            SetupField::Pacer => self.set_pacer_mode(self.ghost_pacer.mode.prev()),
        }
        // Persist immediately (not just at round-start) so a value chosen
        // and then abandoned via Esc is still remembered next launch.
        self.save_settings();
    }

    pub fn setup_cycle_right(&mut self) {
        match self.setup_focus {
            SetupField::Level => self.level = self.level.next(),
            SetupField::Training => self.training_mode = self.training_mode.next(),
            SetupField::Length => self.round_length = self.round_length.next(),
            SetupField::Layout => self.set_layout(self.layout.next()),
            SetupField::Thumb => self.set_thumb_preference(self.thumb_preference.next()),
            SetupField::Pacer => self.set_pacer_mode(self.ghost_pacer.mode.next()),
        }
        self.save_settings();
    }

    // ---- Typing round -----------------------------------------------

    /// Builds fresh practice text from the current `level` + `training_mode`
    /// and enters `Mode::Typing`. Called from the Setup screen (Enter) and
    /// from the Dashboard's "practice again with same settings" flow.
    pub fn start_new_round(&mut self) {
        let slowest = self.latency_engine.slowest_bigrams(5);
        self.target = build_target_text(&self.engine, &slowest, self.level, self.training_mode, self.round_length);
        self.typed.clear();
        self.engine.last_char = None;
        self.latency_engine.reset();
        self.keyboard_state.reset_heat();
        self.last_tick_at = Instant::now();
        self.ghost_pacer.target_wpm = self.resolve_pacer_wpm(self.ghost_pacer.mode);
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

        if actual != expected {
            self.keyboard_state.record_error(expected);
        }

        let prev_char = if idx > 0 { self.target.get(idx - 1).copied() } else { None };
        self.engine.record_keystroke_with_context(expected, actual, prev_char);
        self.latency_engine.record_keystroke(expected, actual, prev_char);
        self.typed.push(TypedChar {
            expected,
            actual,
            at: Instant::now(),
        });

        if self.round_length.is_timed_sprint() {
            // Streaming replenishment: append words as user nears the end
            if self.target.len().saturating_sub(self.typed.len()) < 20 {
                let slowest = self.latency_engine.slowest_bigrams(5);
                let more = build_target_text(&self.engine, &slowest, self.level, self.training_mode, RoundLength::Words15);
                if !self.target.is_empty() && self.target.last() != Some(&' ') {
                    self.target.push(' ');
                }
                self.target.extend(more);
            }
            false
        } else if self.typed.len() >= self.target.len() {
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

    /// Deletes the current or previous word up to whitespace, matching
    /// standard terminal / editor Ctrl+W / Ctrl+Backspace behavior.
    pub fn handle_delete_word(&mut self) {
        if self.typed.is_empty() {
            return;
        }

        // Step 1: Strip trailing spaces if user is on whitespace boundary
        while let Some(last) = self.typed.last() {
            if last.actual == ' ' {
                self.typed.pop();
            } else {
                break;
            }
        }

        // Step 2: Strip word characters up to previous whitespace
        while let Some(last) = self.typed.last() {
            if last.actual != ' ' {
                self.typed.pop();
            } else {
                break;
            }
        }
    }

    /// Instantly restarts the current round from the beginning with the
    /// exact same target text (e.g. on Ctrl+R).
    pub fn restart_current_round(&mut self) {
        self.typed.clear();
        self.engine.last_char = None;
        self.latency_engine.reset();
        self.keyboard_state.reset_heat();
        self.last_tick_at = Instant::now();
        self.session_start = Instant::now();
        self.wpm_samples.clear();
        self.last_wpm_sample_at = Instant::now();
        self.status_message = Some("Round restarted. Ready.".to_string());
    }

    /// Called once per event-loop iteration regardless of whether a key was
    /// pressed, so timing-based effects (the live WPM sparkline, the error
    /// flash fade) keep animating even when the user pauses.
    pub fn tick(&mut self) {
        if self.mode != Mode::Typing {
            return;
        }

        let now = Instant::now();
        let dt = now.duration_since(self.last_tick_at).as_secs_f64();
        self.last_tick_at = now;
        self.keyboard_state.decay(dt);

        if let Some(limit) = self.round_length.timed_duration_secs() {
            let elapsed = self.session_start.elapsed().as_secs_f64();
            if elapsed >= limit {
                self.finish_session();
                return;
            }
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
        let elapsed_secs = if let Some(limit) = self.round_length.timed_duration_secs() {
            limit.min(self.session_start.elapsed().as_secs_f64())
        } else {
            self.session_start.elapsed().as_secs_f64()
        };
        let elapsed_minutes = elapsed_secs / 60.0;
        let words_typed = if self.round_length.is_timed_sprint() {
            self.typed.len() as f64 / 5.0
        } else {
            self.target.len() as f64 / 5.0
        };
        let wpm = if elapsed_minutes > 0.0 {
            words_typed / elapsed_minutes
        } else {
            0.0
        };

        let total_keystrokes = self.typed.len() as u64;
        let correct_keystrokes = self.typed.iter().filter(|t| t.is_correct()).count() as u64;
        let errors = total_keystrokes.saturating_sub(correct_keystrokes);
        let overall_accuracy = if total_keystrokes > 0 {
            (correct_keystrokes as f64 / total_keystrokes as f64) * 100.0
        } else {
            100.0
        };

        self.last_session_wpm = wpm;
        self.last_session_errors = errors;

        // Build Keystroke Velocity timeline and error scatter points for instant debrief
        let mut velocity_timeline = Vec::new();
        if !self.typed.is_empty() {
            velocity_timeline.push((0.0, 0.0));
            for (i, t) in self.typed.iter().enumerate() {
                let t_secs = t.at.duration_since(self.session_start).as_secs_f64();
                let start_idx = i.saturating_sub(4);
                let count = i - start_idx + 1;
                let window_dt = t.at.duration_since(self.typed[start_idx].at).as_secs_f64();
                let inst_wpm = if window_dt > 0.05 {
                    (count as f64 / 5.0) / (window_dt / 60.0)
                } else if t_secs > 0.05 {
                    ((i + 1) as f64 / 5.0) / (t_secs / 60.0)
                } else {
                    0.0
                };
                velocity_timeline.push((t_secs, inst_wpm.min(250.0)));
            }
        }

        let error_points: Vec<(f64, f64)> = self
            .typed
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.is_correct())
            .map(|(i, t)| {
                let t_secs = t.at.duration_since(self.session_start).as_secs_f64();
                let wpm_at = velocity_timeline.get(i + 1).map(|p| p.1).unwrap_or(0.0);
                (t_secs, wpm_at)
            })
            .collect();

        let raw_wpm = if elapsed_minutes > 0.0 {
            (total_keystrokes as f64 / 5.0) / elapsed_minutes
        } else {
            0.0
        };

        let slowest_bigrams = self.latency_engine.slowest_bigrams(5);

        let mut mistake_counts: std::collections::HashMap<(char, char), u64> = std::collections::HashMap::new();
        for t in &self.typed {
            if !t.is_correct() {
                *mistake_counts.entry((t.expected, t.actual)).or_insert(0) += 1;
            }
        }
        let mut mistake_vec: Vec<((char, char), u64)> = mistake_counts.into_iter().collect();
        mistake_vec.sort_by_key(|a| std::cmp::Reverse(a.1));

        if self.track_progress == Some(true) {
            use std::collections::HashMap;

            let mut finger_keystrokes: HashMap<String, u64> = HashMap::new();
            let mut finger_errors: HashMap<String, u64> = HashMap::new();

            for (i, t) in self.typed.iter().enumerate() {
                let prev = if i > 0 { self.typed.get(i - 1).map(|p| p.expected) } else { None };
                if let Some(finger) = self.engine.finger_for_context(t.expected, prev) {
                    let key = finger.as_key().to_string();
                    *finger_keystrokes.entry(key.clone()).or_insert(0) += 1;
                    if !t.is_correct() {
                        *finger_errors.entry(key.clone()).or_insert(0) += 1;
                    }
                }
            }

            let mut finger_accuracy: HashMap<String, f64> = HashMap::new();
            for finger in Finger::ALL {
                let key = finger.as_key().to_string();
                let total = *finger_keystrokes.get(&key).unwrap_or(&0);
                let errs = *finger_errors.get(&key).unwrap_or(&0);
                let acc = if total > 0 {
                    ((total - errs) as f64 / total as f64) * 100.0
                } else {
                    100.0
                };
                finger_accuracy.insert(key, acc);
            }

            let top_mistakes_str = mistake_vec
                .iter()
                .take(10)
                .map(|((exp, act), cnt)| format!("{exp}->{act}:{cnt}"))
                .collect::<Vec<_>>()
                .join(";");

            let snapshot = SessionSnapshot {
                timestamp: persistence::current_unix_timestamp(),
                wpm,
                finger_accuracy,
                session_duration_secs: Some((elapsed_secs * 10.0).round() / 10.0),
                level: Some(match self.level {
                    Level::Beginner => "Beginner".to_string(),
                    Level::Intermediate => "Intermediate".to_string(),
                    Level::Advanced => "Advanced".to_string(),
                }),
                training_mode: Some(match self.training_mode {
                    TrainingMode::FocusWeakest => "Targeted".to_string(),
                    TrainingMode::Random => "Random".to_string(),
                    TrainingMode::BigramLadder => "Bigram Ladder".to_string(),
                    TrainingMode::CodeMode => "Code Syntax".to_string(),
                }),
                round_length: Some(match self.round_length {
                    RoundLength::Burst5 => "Burst".to_string(),
                    RoundLength::Words10 => "10 words".to_string(),
                    RoundLength::Words15 => "15 words".to_string(),
                    RoundLength::Words20 => "20 words".to_string(),
                    RoundLength::Words30 => "30 words".to_string(),
                    RoundLength::Words40 => "40 words".to_string(),
                    RoundLength::Words50 => "50 words".to_string(),
                    RoundLength::Words60 => "60 words".to_string(),
                    RoundLength::Sprint15 => "Sprint 15s".to_string(),
                    RoundLength::Sprint30 => "Sprint 30s".to_string(),
                    RoundLength::Sprint60 => "Sprint 60s".to_string(),
                }),
                overall_accuracy: Some((overall_accuracy * 100.0).round() / 100.0),
                total_keystrokes: Some(total_keystrokes),
                correct_keystrokes: Some(correct_keystrokes),
                error_count: Some(errors),
                finger_keystrokes,
                finger_errors,
                top_mistakes: if top_mistakes_str.is_empty() { None } else { Some(top_mistakes_str) },
                avg_iki_ms: Some((self.latency_engine.avg_iki_ms() * 10.0).round() / 10.0),
                consistency_pct: Some((self.latency_engine.cadence.consistency_pct() * 10.0).round() / 10.0),
            };

            self.history.push(snapshot);

            let save_status = match persistence::save_history(&self.history_path, &self.history) {
                Ok(()) => format!("Session saved to {}", self.history_path.display()),
                Err(e) => format!("Failed to save history: {}", e),
            };
            self.status_message = Some(save_status);
        } else {
            self.status_message =
                Some("Session complete. Progress tracking is off, so it wasn't saved.".to_string());
        }

        if self.ghost_pacer.is_active() && self.ghost_pacer.target_wpm > 0.0 {
            if overall_accuracy >= 96.0 && wpm >= self.ghost_pacer.target_wpm {
                self.ghost_pacer.target_wpm += 5.0;
                self.status_message = Some(format!(
                    "Sprint Target raised to {:.0} WPM! Outstanding burst.",
                    self.ghost_pacer.target_wpm
                ));
            } else if overall_accuracy < 90.0 {
                self.status_message = Some("Focus on clean execution (≥90% accuracy) before speed.".to_string());
            }
        }

        self.last_round_summary = Some(RoundSummary {
            net_wpm: wpm,
            raw_wpm,
            accuracy_pct: overall_accuracy,
            consistency_pct: self.latency_engine.cadence.consistency_pct(),
            duration_secs: elapsed_secs,
            total_keystrokes,
            errors,
            velocity_timeline,
            error_points,
            slowest_bigrams,
            mistake_counts: mistake_vec,
            target_text: self.target.clone(),
        });

        self.mode = Mode::Debrief;
    }

    // ---- Dashboard ----------------------------------------------------

    /// All ten fingers, sorted worst-accuracy-first. Both the sidebar list
    /// and `selected_finger` walk this same order so "the finger currently
    /// highlighted" always matches "the row currently drawn as selected".
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
        self.dashboard_tab = self.dashboard_tab.next();
    }

    pub fn prev_dashboard_tab(&mut self) {
        self.dashboard_tab = self.dashboard_tab.prev();
    }

    pub fn set_dashboard_tab(&mut self, tab: DashboardTab) {
        self.dashboard_tab = tab;
    }

    pub fn cycle_time_filter(&mut self) {
        self.time_filter = self.time_filter.next();
    }

    pub fn cycle_level_filter(&mut self) {
        self.level_filter = self.level_filter.next();
    }

    pub fn cycle_mode_filter(&mut self) {
        self.mode_filter = self.mode_filter.next();
    }

    pub fn cycle_heatmap_mode(&mut self) {
        self.heatmap_mode = self.heatmap_mode.next();
    }

    pub fn move_heatmap_selection(&mut self, row_delta: i32, col_delta: i32) {
        let key = self.selected_heatmap_key;
        let mut cur_row = 1;
        let mut cur_col = 2; // default to 'e' in row 1
        'outer: for (r_idx, row) in HEATMAP_KEYS.iter().enumerate() {
            for (c_idx, &c) in row.iter().enumerate() {
                if c == key {
                    cur_row = r_idx;
                    cur_col = c_idx;
                    break 'outer;
                }
            }
        }

        let num_rows = HEATMAP_KEYS.len() as i32;
        let new_row = (cur_row as i32 + row_delta).rem_euclid(num_rows) as usize;
        let row_len = HEATMAP_KEYS[new_row].len() as i32;
        let new_col = (cur_col as i32 + col_delta).rem_euclid(row_len) as usize;

        self.selected_heatmap_key = HEATMAP_KEYS[new_row][new_col];
    }

    pub fn history_next(&mut self, max: usize) {
        if max == 0 {
            self.history_table_state.select(None);
            return;
        }
        let next = match self.history_table_state.selected() {
            Some(i) if i + 1 < max => i + 1,
            _ => 0,
        };
        self.history_table_state.select(Some(next));
    }

    pub fn history_prev(&mut self, max: usize) {
        if max == 0 {
            self.history_table_state.select(None);
            return;
        }
        let prev = match self.history_table_state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    max.saturating_sub(1)
                }
            }
            None => 0,
        };
        self.history_table_state.select(Some(prev));
    }

    pub fn history_page_down(&mut self, max: usize, page: usize) {
        if max == 0 {
            return;
        }
        let next = match self.history_table_state.selected() {
            Some(i) => (i + page).min(max.saturating_sub(1)),
            None => 0,
        };
        self.history_table_state.select(Some(next));
    }

    pub fn history_page_up(&mut self, page: usize) {
        let prev = match self.history_table_state.selected() {
            Some(i) => i.saturating_sub(page),
            None => 0,
        };
        self.history_table_state.select(Some(prev));
    }

    pub fn history_first(&mut self) {
        self.history_table_state.select(Some(0));
    }

    pub fn history_last(&mut self, max: usize) {
        if max > 0 {
            self.history_table_state.select(Some(max - 1));
        }
    }

    pub fn filtered_sessions(&self) -> Vec<&SessionSnapshot> {
        let now = persistence::current_unix_timestamp();
        self.history
            .sessions
            .iter()
            .filter(|s| match self.time_filter {
                TimeFilter::All => true,
                TimeFilter::Last7Days => s.timestamp >= now.saturating_sub(7 * 86400),
                TimeFilter::Last30Days => s.timestamp >= now.saturating_sub(30 * 86400),
            })
            .filter(|s| match self.level_filter {
                LevelFilter::All => true,
                LevelFilter::Beginner => s.level.as_deref() == Some("Beginner"),
                LevelFilter::Intermediate => s.level.as_deref() == Some("Intermediate"),
                LevelFilter::Advanced => s.level.as_deref() == Some("Advanced"),
            })
            .filter(|s| match self.mode_filter {
                ModeFilter::All => true,
                ModeFilter::Targeted => s.training_mode.as_deref() == Some("Targeted"),
                ModeFilter::Random => s.training_mode.as_deref() == Some("Random"),
            })
            .collect()
    }

    pub fn filtered_summary(&self) -> FilteredSummary {
        let filtered = self.filtered_sessions();
        let total_count = self.history.sessions.len();
        let count = filtered.len();
        if count == 0 {
            return FilteredSummary {
                count: 0,
                total_count,
                avg_wpm: 0.0,
                peak_wpm: 0.0,
                avg_accuracy: 0.0,
                total_keystrokes: 0,
                total_errors: 0,
                streak_90: 0,
            };
        }
        let sum_wpm: f64 = filtered.iter().map(|s| s.wpm).sum();
        let peak_wpm = filtered.iter().map(|s| s.wpm).fold(0.0_f64, f64::max);
        let sum_acc: f64 = filtered.iter().map(|s| s.overall_accuracy.unwrap_or(0.0)).sum();
        let total_keystrokes: u64 = filtered.iter().map(|s| s.total_keystrokes.unwrap_or(0)).sum();
        let total_errors: u64 = filtered.iter().map(|s| s.error_count.unwrap_or(0)).sum();

        let mut streak_90 = 0;
        for s in filtered.iter().rev() {
            if s.overall_accuracy.unwrap_or(0.0) >= 90.0 {
                streak_90 += 1;
            } else {
                break;
            }
        }

        FilteredSummary {
            count,
            total_count,
            avg_wpm: sum_wpm / count as f64,
            peak_wpm,
            avg_accuracy: sum_acc / count as f64,
            total_keystrokes,
            total_errors,
            streak_90,
        }
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
    slowest_bigrams: &[((char, char), f64)],
    level: Level,
    mode: TrainingMode,
    round_length: RoundLength,
) -> Vec<char> {
    match mode {
        TrainingMode::BigramLadder => {
            crate::drills::bigrams::generate_bigram_drill(slowest_bigrams, level, round_length)
        }
        TrainingMode::CodeMode => {
            crate::drills::code::generate_code_drill(level, round_length)
        }
        TrainingMode::FocusWeakest => {
            let target_finger = engine.get_most_faulty_finger().map(|(f, _)| f);
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
        TrainingMode::Random => {
            let word_count = round_length.word_count();
            match level {
                Level::Beginner => {
                    let text = generator::generate_for_finger(engine, None, word_count);
                    text.chars().collect()
                }
                Level::Intermediate | Level::Advanced => {
                    paragraphs::pick_random(level, word_count)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_delete_word() {
        let mut app = App::new();
        let chars = "hello world ";
        for c in chars.chars() {
            app.typed.push(TypedChar {
                expected: c,
                actual: c,
                at: Instant::now(),
            });
        }
        assert_eq!(app.typed.len(), 12);
        // First delete word should strip trailing space and "world"
        app.handle_delete_word();
        let remaining: String = app.typed.iter().map(|t| t.actual).collect();
        assert_eq!(remaining, "hello ");

        // Second delete word should strip trailing space and "hello"
        app.handle_delete_word();
        assert!(app.typed.is_empty());
    }

    #[test]
    fn test_restart_current_round() {
        let mut app = App::new();
        app.target = vec!['a', 'b', 'c'];
        app.typed.push(TypedChar {
            expected: 'a',
            actual: 'a',
            at: Instant::now(),
        });
        app.wpm_samples.push_back(60);

        app.restart_current_round();
        assert!(app.typed.is_empty());
        assert!(app.wpm_samples.is_empty());
        assert_eq!(app.target, vec!['a', 'b', 'c']);
        assert_eq!(app.status_message.as_deref(), Some("Round restarted. Ready."));
    }

    #[test]
    fn test_round_length_cycling_and_timed_sprint() {
        assert_eq!(RoundLength::Sprint15.timed_duration_secs(), Some(15.0));
        assert_eq!(RoundLength::Sprint30.timed_duration_secs(), Some(30.0));
        assert_eq!(RoundLength::Sprint60.timed_duration_secs(), Some(60.0));
        assert_eq!(RoundLength::Words15.timed_duration_secs(), None);
        assert!(RoundLength::Sprint15.is_timed_sprint());
        assert!(!RoundLength::Words15.is_timed_sprint());

        // Test word counts across the ladder
        assert_eq!(RoundLength::Burst5.word_count(), 5);
        assert_eq!(RoundLength::Words10.word_count(), 10);
        assert_eq!(RoundLength::Words15.word_count(), 15);
        assert_eq!(RoundLength::Words20.word_count(), 20);
        assert_eq!(RoundLength::Words30.word_count(), 30);
        assert_eq!(RoundLength::Words40.word_count(), 40);
        assert_eq!(RoundLength::Words50.word_count(), 50);
        assert_eq!(RoundLength::Words60.word_count(), 60);

        // Test cycling wraps cleanly
        let mut rl = RoundLength::Sprint60;
        rl = rl.next();
        assert_eq!(rl, RoundLength::Burst5);
        rl = rl.prev();
        assert_eq!(rl, RoundLength::Sprint60);

        // Test serde backward-compatibility for legacy "Short", "Medium", "Long"
        let legacy_short: RoundLength = serde_json::from_str("\"Short\"").unwrap();
        assert_eq!(legacy_short, RoundLength::Words15);
        let legacy_med: RoundLength = serde_json::from_str("\"Medium\"").unwrap();
        assert_eq!(legacy_med, RoundLength::Words30);
        let legacy_long: RoundLength = serde_json::from_str("\"Long\"").unwrap();
        assert_eq!(legacy_long, RoundLength::Words50);
    }

    #[test]
    fn test_timed_sprint_streaming_replenishment() {
        let mut app = App::new();
        app.round_length = RoundLength::Sprint15;
        app.target = "short initial line".chars().collect();
        let initial_len = app.target.len();

        // Type almost to the end
        for &c in &app.target.clone() {
            app.handle_char_input(c);
        }

        // Target should have replenished with new words automatically
        assert!(app.target.len() > initial_len);
    }

    #[test]
    fn test_debrief_summary_generation() {
        let mut app = App::new();
        app.target = vec!['c', 'a', 't'];
        app.start_new_round();
        app.target = vec!['c', 'a', 't'];

        app.handle_char_input('c');
        app.handle_char_input('x'); // mistake ('a' -> 'x')
        app.handle_char_input('t');

        assert_eq!(app.mode, Mode::Debrief);
        let summary = app.last_round_summary.as_ref().expect("summary should be populated");
        assert_eq!(summary.total_keystrokes, 3);
        assert_eq!(summary.errors, 1);
        assert!(!summary.velocity_timeline.is_empty());
        assert_eq!(summary.error_points.len(), 1);
    }

    #[test]
    fn test_dashboard_tab_and_filter_cycling() {
        let mut app = App::new();
        assert_eq!(app.dashboard_tab, DashboardTab::Trends);
        app.cycle_dashboard_tab();
        assert_eq!(app.dashboard_tab, DashboardTab::KeyboardHeatmap);
        app.cycle_dashboard_tab();
        assert_eq!(app.dashboard_tab, DashboardTab::FingerLoad);
        app.cycle_dashboard_tab();
        assert_eq!(app.dashboard_tab, DashboardTab::LevelComparison);
        app.cycle_dashboard_tab();
        assert_eq!(app.dashboard_tab, DashboardTab::SessionHistory);
        app.cycle_dashboard_tab();
        assert_eq!(app.dashboard_tab, DashboardTab::Trends);
        app.prev_dashboard_tab();
        assert_eq!(app.dashboard_tab, DashboardTab::SessionHistory);

        // Filters cycling
        assert_eq!(app.time_filter, TimeFilter::All);
        app.cycle_time_filter();
        assert_eq!(app.time_filter, TimeFilter::Last7Days);
        app.cycle_time_filter();
        assert_eq!(app.time_filter, TimeFilter::Last30Days);
        app.cycle_time_filter();
        assert_eq!(app.time_filter, TimeFilter::All);

        assert_eq!(app.level_filter, LevelFilter::All);
        app.cycle_level_filter();
        assert_eq!(app.level_filter, LevelFilter::Beginner);

        assert_eq!(app.mode_filter, ModeFilter::All);
        app.cycle_mode_filter();
        assert_eq!(app.mode_filter, ModeFilter::Targeted);

        assert_eq!(app.heatmap_mode, HeatmapMode::FingerAccuracy);
        app.cycle_heatmap_mode();
        assert_eq!(app.heatmap_mode, HeatmapMode::KeyErrors);
    }

    #[test]
    fn test_filtered_sessions_and_summary() {
        let mut app = App::new();
        app.history.sessions.clear();
        let now = crate::persistence::current_unix_timestamp();

        let s1 = SessionSnapshot {
            timestamp: now,
            wpm: 60.0,
            finger_accuracy: std::collections::HashMap::new(),
            session_duration_secs: Some(30.0),
            level: Some("Beginner".to_string()),
            training_mode: Some("Random".to_string()),
            round_length: Some("Short".to_string()),
            overall_accuracy: Some(95.0),
            total_keystrokes: Some(150),
            correct_keystrokes: Some(142),
            error_count: Some(8),
            finger_keystrokes: std::collections::HashMap::new(),
            finger_errors: std::collections::HashMap::new(),
            top_mistakes: None,
            avg_iki_ms: Some(150.0),
            consistency_pct: Some(88.0),
        };

        let s2 = SessionSnapshot {
            timestamp: now - 10 * 86400, // 10 days ago
            wpm: 80.0,
            finger_accuracy: std::collections::HashMap::new(),
            session_duration_secs: Some(45.0),
            level: Some("Advanced".to_string()),
            training_mode: Some("Targeted".to_string()),
            round_length: Some("Medium".to_string()),
            overall_accuracy: Some(98.0),
            total_keystrokes: Some(250),
            correct_keystrokes: Some(245),
            error_count: Some(5),
            finger_keystrokes: std::collections::HashMap::new(),
            finger_errors: std::collections::HashMap::new(),
            top_mistakes: None,
            avg_iki_ms: Some(130.0),
            consistency_pct: Some(94.0),
        };

        app.history.push(s1);
        app.history.push(s2);

        // Filter: All
        let all_filtered = app.filtered_sessions();
        assert_eq!(all_filtered.len(), 2);
        let summary = app.filtered_summary();
        assert_eq!(summary.count, 2);
        assert_eq!(summary.peak_wpm, 80.0);
        assert_eq!(summary.total_keystrokes, 400);

        // Filter: Last 7 days
        app.time_filter = TimeFilter::Last7Days;
        let last_7 = app.filtered_sessions();
        assert_eq!(last_7.len(), 1);
        assert_eq!(last_7[0].level.as_deref(), Some("Beginner"));

        // Filter: Level Advanced
        app.time_filter = TimeFilter::All;
        app.level_filter = LevelFilter::Advanced;
        let adv_filtered = app.filtered_sessions();
        assert_eq!(adv_filtered.len(), 1);
        assert_eq!(adv_filtered[0].wpm, 80.0);
    }

    #[test]
    fn test_heatmap_and_history_navigation() {
        let mut app = App::new();
        app.selected_heatmap_key = 'e';

        // Move right from 'e' in row 1 -> 'r'
        app.move_heatmap_selection(0, 1);
        assert_eq!(app.selected_heatmap_key, 'r');

        // Move down from 'r' -> 'f'
        app.move_heatmap_selection(1, 0);
        assert_eq!(app.selected_heatmap_key, 'f');

        // History table navigation
        app.history_next(5);
        assert_eq!(app.history_table_state.selected(), Some(1));
        app.history_prev(5);
        assert_eq!(app.history_table_state.selected(), Some(0));
        app.history_last(5);
        assert_eq!(app.history_table_state.selected(), Some(4));
        app.history_first();
        assert_eq!(app.history_table_state.selected(), Some(0));
    }
}
