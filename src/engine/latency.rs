//! Keystroke Inter-Key Interval (IKI) measurement and cadence analysis engine.
//!
//! Provides monotonic keystroke latency tracking, rolling cadence consistency (CV),
//! biomechanical transition classification, and bigram hesitation matrix tracking.

use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use serde::{Deserialize, Serialize};

use crate::finger::{Finger, TypingEngine};

/// Biomechanical classification of a two-key transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum TransitionClass {
    /// Alternating hands (e.g. Left -> Right). Typically lowest latency.
    AlternatingHands,
    /// Same hand, different fingers (finger roll).
    SameHandRoll,
    /// Same physical finger typing consecutive keys (e.g. 'c' -> 'e').
    SameFingerLeap,
    /// Involves pinky or ring fingers on awkward reaches.
    WeakFinger,
}

/// Classifies a two-character transition into a biomechanical transition class.
#[allow(dead_code)]
pub fn classify_transition(c1: char, c2: char, engine: &TypingEngine) -> TransitionClass {
    let f1 = engine.finger_for(c1);
    let f2 = engine.finger_for(c2);
    match (f1, f2) {
        (Some(a), Some(b)) if a.is_left_hand() != b.is_left_hand() => TransitionClass::AlternatingHands,
        (Some(a), Some(b)) if a == b => TransitionClass::SameFingerLeap,
        (Some(a), Some(b))
            if matches!(a, Finger::LeftPinky | Finger::RightPinky | Finger::LeftRing | Finger::RightRing)
                || matches!(b, Finger::LeftPinky | Finger::RightPinky | Finger::LeftRing | Finger::RightRing) =>
        {
            TransitionClass::WeakFinger
        }
        _ => TransitionClass::SameHandRoll,
    }
}

/// Cumulative latency and error stats for a specific bigram transition (c1 -> c2).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BigramStat {
    pub total_occurrences: u64,
    pub total_latency_ms: u64,
    pub error_count: u64,
}

impl BigramStat {
    #[allow(dead_code)]
    pub fn avg_latency_ms(&self) -> f64 {
        if self.total_occurrences == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.total_occurrences as f64
        }
    }
}

/// Rolling cadence tracker computing rhythm consistency across a sliding window of IKIs.
#[derive(Debug, Clone)]
pub struct CadenceTracker {
    pub window_size: usize,
    pub window: VecDeque<u64>,
}

impl Default for CadenceTracker {
    fn default() -> Self {
        Self::new(15)
    }
}

impl CadenceTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            window: VecDeque::with_capacity(window_size),
        }
    }

    pub fn record_iki(&mut self, iki_ms: u64) {
        if iki_ms == 0 {
            return;
        }
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
        self.window.push_back(iki_ms);
    }

    pub fn reset(&mut self) {
        self.window.clear();
    }

    pub fn mean_iki(&self) -> f64 {
        if self.window.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.window.iter().sum();
        sum as f64 / self.window.len() as f64
    }

    pub fn std_dev(&self) -> f64 {
        if self.window.len() < 2 {
            return 0.0;
        }
        let mean = self.mean_iki();
        let variance = self.window.iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / self.window.len() as f64;
        variance.sqrt()
    }

    pub fn coefficient_of_variation(&self) -> f64 {
        let mean = self.mean_iki();
        if mean <= 0.0 {
            0.0
        } else {
            self.std_dev() / mean
        }
    }

    pub fn consistency_pct(&self) -> f64 {
        if self.window.len() < 3 {
            return 100.0;
        }
        let cv = self.coefficient_of_variation();
        let penalty = (cv * 1.25).min(1.0);
        let score = 100.0 * (1.0 - penalty);
        score.clamp(0.0, 100.0)
    }
}

/// Individual keystroke timing log with inter-key interval in milliseconds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KeystrokeLog {
    pub expected: char,
    pub actual: char,
    pub timestamp_ms: u64,
    pub iki_ms: u64,
}

/// Complete keystroke latency and bigram hesitation engine.
#[derive(Debug, Clone)]
pub struct LatencyEngine {
    pub last_keystroke_at: Option<Instant>,
    pub session_start: Instant,
    pub cadence: CadenceTracker,
    pub bigram_latencies: HashMap<(char, char), BigramStat>,
    pub logs: Vec<KeystrokeLog>,
}

impl Default for LatencyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyEngine {
    pub fn new() -> Self {
        Self {
            last_keystroke_at: None,
            session_start: Instant::now(),
            cadence: CadenceTracker::default(),
            bigram_latencies: HashMap::new(),
            logs: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.last_keystroke_at = None;
        self.session_start = Instant::now();
        self.cadence.reset();
        self.logs.clear();
    }

    #[allow(dead_code)]
    pub fn reset_all(&mut self) {
        self.reset();
        self.bigram_latencies.clear();
    }

    pub fn record_keystroke(
        &mut self,
        expected: char,
        actual: char,
        prev_char: Option<char>,
    ) -> u64 {
        let now = Instant::now();
        let iki_ms = match self.last_keystroke_at {
            Some(prev) => prev.elapsed().as_millis() as u64,
            None => 0,
        };
        self.last_keystroke_at = Some(now);

        let elapsed_from_start = now.duration_since(self.session_start).as_millis() as u64;

        if iki_ms > 0 {
            self.cadence.record_iki(iki_ms);
        }

        self.logs.push(KeystrokeLog {
            expected,
            actual,
            timestamp_ms: elapsed_from_start,
            iki_ms,
        });

        if let Some(prev) = prev_char {
            let stat = self.bigram_latencies.entry((prev, expected)).or_default();
            stat.total_occurrences += 1;
            stat.total_latency_ms += iki_ms;
            if expected != actual {
                stat.error_count += 1;
            }
        }

        iki_ms
    }

    /// Returns the top slowest bigrams (minimum 2 occurrences) sorted descending by average latency.
    #[allow(dead_code)]
    pub fn slowest_bigrams(&self, limit: usize) -> Vec<((char, char), f64)> {
        let mut list: Vec<_> = self
            .bigram_latencies
            .iter()
            .filter(|(_, stat)| stat.total_occurrences >= 2)
            .map(|(bigram, stat)| (*bigram, stat.avg_latency_ms()))
            .collect();
        list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        list.truncate(limit);
        list
    }

    /// Average IKI across recorded keystrokes with non-zero latency.
    pub fn avg_iki_ms(&self) -> f64 {
        let non_zero: Vec<u64> = self.logs.iter().map(|l| l.iki_ms).filter(|&ms| ms > 0).collect();
        if non_zero.is_empty() {
            return 0.0;
        }
        let sum: u64 = non_zero.iter().sum();
        sum as f64 / non_zero.len() as f64
    }

    /// Median IKI across recorded keystrokes with non-zero latency.
    #[allow(dead_code)]
    pub fn median_iki(&self) -> f64 {
        let mut ikis: Vec<u64> = self.logs.iter().map(|l| l.iki_ms).filter(|&ms| ms > 0).collect();
        if ikis.is_empty() {
            return 0.0;
        }
        ikis.sort_unstable();
        let mid = ikis.len() / 2;
        if ikis.len().is_multiple_of(2) {
            (ikis[mid - 1] + ikis[mid]) as f64 / 2.0
        } else {
            ikis[mid] as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cadence_tracker_constant_interval() {
        let mut tracker = CadenceTracker::new(10);
        for _ in 0..10 {
            tracker.record_iki(100);
        }
        assert!((tracker.mean_iki() - 100.0).abs() < 1e-6);
        assert!((tracker.std_dev() - 0.0).abs() < 1e-6);
        assert!((tracker.consistency_pct() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_cadence_tracker_high_jitter() {
        let mut tracker = CadenceTracker::new(10);
        for i in 0..10 {
            tracker.record_iki(if i % 2 == 0 { 50 } else { 400 });
        }
        assert!(tracker.consistency_pct() < 70.0);
    }

    #[test]
    fn test_latency_engine_records_bigrams() {
        let mut engine = LatencyEngine::new();
        engine.record_keystroke('t', 't', None);
        engine.record_keystroke('h', 'h', Some('t'));
        engine.record_keystroke('e', 'e', Some('h'));
        engine.record_keystroke('t', 't', Some('e'));
        engine.record_keystroke('h', 'h', Some('t'));

        assert_eq!(engine.logs.len(), 5);
        let slowest = engine.slowest_bigrams(5);
        assert_eq!(slowest.len(), 1); // ('t', 'h') has 2 occurrences
        assert_eq!(slowest[0].0, ('t', 'h'));
    }

    #[test]
    fn test_transition_classification() {
        let typing_engine = TypingEngine::new();
        // 'f' (LeftIndex) -> 'j' (RightIndex) => Alternating
        assert_eq!(
            classify_transition('f', 'j', &typing_engine),
            TransitionClass::AlternatingHands
        );
        // 'c' (LeftMiddle) -> 'e' (LeftMiddle) => SameFingerLeap
        assert_eq!(
            classify_transition('c', 'e', &typing_engine),
            TransitionClass::SameFingerLeap
        );
        // 'a' (LeftPinky) -> 's' (LeftRing) => WeakFinger
        assert_eq!(
            classify_transition('a', 's', &typing_engine),
            TransitionClass::WeakFinger
        );
        // 'd' (LeftMiddle) -> 'f' (LeftIndex) => SameHandRoll
        assert_eq!(
            classify_transition('d', 'f', &typing_engine),
            TransitionClass::SameHandRoll
        );
    }

    #[test]
    fn test_median_and_avg_iki() {
        let mut engine = LatencyEngine::new();
        engine.logs.push(KeystrokeLog { expected: 'a', actual: 'a', timestamp_ms: 0, iki_ms: 0 });
        engine.logs.push(KeystrokeLog { expected: 'b', actual: 'b', timestamp_ms: 100, iki_ms: 100 });
        engine.logs.push(KeystrokeLog { expected: 'c', actual: 'c', timestamp_ms: 300, iki_ms: 200 });
        engine.logs.push(KeystrokeLog { expected: 'd', actual: 'd', timestamp_ms: 600, iki_ms: 300 });

        assert_eq!(engine.avg_iki_ms(), 200.0);
        assert_eq!(engine.median_iki(), 200.0);
    }
}
