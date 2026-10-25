//! Deterministic ghost pacer velocity benchmark and real-time delta calculations.

use serde::{Deserialize, Serialize};

/// Pacer velocity mode configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PacerMode {
    #[default]
    Off,
    Target(u32),
    PersonalBest,
}

impl PacerMode {
    pub const ALL: [PacerMode; 7] = [
        PacerMode::Off,
        PacerMode::Target(40),
        PacerMode::Target(60),
        PacerMode::Target(80),
        PacerMode::Target(100),
        PacerMode::Target(120),
        PacerMode::PersonalBest,
    ];

    pub fn label(&self) -> String {
        match self {
            PacerMode::Off => "Off".to_string(),
            PacerMode::Target(wpm) => format!("{wpm} WPM"),
            PacerMode::PersonalBest => "Personal Best".to_string(),
        }
    }

    pub fn next(&self) -> PacerMode {
        let idx = PacerMode::ALL.iter().position(|m| m == self).unwrap_or(0);
        PacerMode::ALL[(idx + 1) % PacerMode::ALL.len()]
    }

    pub fn prev(&self) -> PacerMode {
        let idx = PacerMode::ALL.iter().position(|m| m == self).unwrap_or(0);
        PacerMode::ALL[(idx + PacerMode::ALL.len() - 1) % PacerMode::ALL.len()]
    }
}

/// The ghost pacer state moving deterministically through text.
#[derive(Debug, Clone)]
pub struct GhostPacer {
    pub mode: PacerMode,
    pub target_wpm: f64,
}

impl Default for GhostPacer {
    fn default() -> Self {
        Self::new(PacerMode::default(), 0.0)
    }
}

impl GhostPacer {
    pub fn new(mode: PacerMode, target_wpm: f64) -> Self {
        Self { mode, target_wpm }
    }

    pub fn is_active(&self) -> bool {
        self.mode != PacerMode::Off && self.target_wpm > 0.0
    }

    /// Computes the deterministic character index of the ghost at `elapsed_secs`.
    ///
    /// Formula: floor((target_wpm * 5 / 60) * elapsed_secs).min(target_len)
    pub fn ghost_char_index(&self, elapsed_secs: f64, target_len: usize) -> usize {
        if !self.is_active() || elapsed_secs <= 0.0 {
            return 0;
        }
        let chars_per_sec = (self.target_wpm * 5.0) / 60.0;
        let expected = (chars_per_sec * elapsed_secs).floor() as usize;
        expected.min(target_len)
    }

    pub fn delta_wpm(&self, current_wpm: f64) -> f64 {
        current_wpm - self.target_wpm
    }

    pub fn delta_chars(&self, typed_len: usize, ghost_idx: usize) -> i64 {
        typed_len as i64 - ghost_idx as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ghost_char_index_calculation() {
        // At 60 WPM, velocity is 5 chars per second (300 chars/min)
        let pacer = GhostPacer::new(PacerMode::Target(60), 60.0);
        assert_eq!(pacer.ghost_char_index(0.0, 100), 0);
        assert_eq!(pacer.ghost_char_index(1.0, 100), 5);
        assert_eq!(pacer.ghost_char_index(10.0, 100), 50);
        assert_eq!(pacer.ghost_char_index(25.0, 100), 100);
        assert_eq!(pacer.ghost_char_index(30.0, 100), 100); // capped at target_len
    }

    #[test]
    fn test_delta_calculations() {
        let pacer = GhostPacer::new(PacerMode::Target(80), 80.0);
        assert!((pacer.delta_wpm(86.4) - 6.4).abs() < 1e-6);
        assert!((pacer.delta_wpm(75.0) - (-5.0)).abs() < 1e-6);
        assert_eq!(pacer.delta_chars(50, 42), 8);
        assert_eq!(pacer.delta_chars(40, 45), -5);
    }

    #[test]
    fn test_pacer_mode_cycling() {
        let mut m = PacerMode::Off;
        m = m.next();
        assert_eq!(m, PacerMode::Target(40));
        m = m.prev();
        assert_eq!(m, PacerMode::Off);
        m = m.prev();
        assert_eq!(m, PacerMode::PersonalBest);
    }
}
