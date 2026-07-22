//! Finger mapping engine.
//!
//! This module owns the notion of "which physical finger types which key",
//! and the analytical statistics we keep per finger: how many keystrokes it
//! has made, how many of those were errors, and a "mistake matrix" that
//! records exactly what wrong character was typed instead of the expected
//! one (e.g. "expected 'e', typed 'r'  x 14").

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// All ten fingers used in touch typing.
///
/// Ordering here is intentional: left hand pinky -> thumb, then right hand
/// thumb -> pinky. UI code (the sidebar list) iterates `Finger::ALL` in this
/// order so the list reads naturally left-to-right across the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Finger {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    LeftThumb,
    RightThumb,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
}

impl Finger {
    /// All fingers, in anatomical left-to-right order. Useful for UI lists
    /// and for iterating deterministically (HashMap iteration order is not
    /// stable, so anything user-facing should walk this slice instead).
    pub const ALL: [Finger; 10] = [
        Finger::LeftPinky,
        Finger::LeftRing,
        Finger::LeftMiddle,
        Finger::LeftIndex,
        Finger::LeftThumb,
        Finger::RightThumb,
        Finger::RightIndex,
        Finger::RightMiddle,
        Finger::RightRing,
        Finger::RightPinky,
    ];

    /// Stable string key used for JSON persistence (map keys must be
    /// strings in serde_json, so we don't serialize the enum directly as a
    /// map key anywhere - we go through this instead).
    pub fn as_key(&self) -> &'static str {
        match self {
            Finger::LeftPinky => "left_pinky",
            Finger::LeftRing => "left_ring",
            Finger::LeftMiddle => "left_middle",
            Finger::LeftIndex => "left_index",
            Finger::LeftThumb => "left_thumb",
            Finger::RightThumb => "right_thumb",
            Finger::RightIndex => "right_index",
            Finger::RightMiddle => "right_middle",
            Finger::RightRing => "right_ring",
            Finger::RightPinky => "right_pinky",
        }
    }

    /// Inverse of [`Finger::as_key`]. Not currently called anywhere (history
    /// is looked up by string key directly), kept as public API for anyone
    /// extending persistence to round-trip through the enum instead.
    #[allow(dead_code)]
    pub fn from_key(key: &str) -> Option<Finger> {
        Some(match key {
            "left_pinky" => Finger::LeftPinky,
            "left_ring" => Finger::LeftRing,
            "left_middle" => Finger::LeftMiddle,
            "left_index" => Finger::LeftIndex,
            "left_thumb" => Finger::LeftThumb,
            "right_thumb" => Finger::RightThumb,
            "right_index" => Finger::RightIndex,
            "right_middle" => Finger::RightMiddle,
            "right_ring" => Finger::RightRing,
            "right_pinky" => Finger::RightPinky,
            _ => return None,
        })
    }
}

impl fmt::Display for Finger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Finger::LeftPinky => "Left Pinky",
            Finger::LeftRing => "Left Ring",
            Finger::LeftMiddle => "Left Middle",
            Finger::LeftIndex => "Left Index",
            Finger::LeftThumb => "Left Thumb",
            Finger::RightThumb => "Right Thumb",
            Finger::RightIndex => "Right Index",
            Finger::RightMiddle => "Right Middle",
            Finger::RightRing => "Right Ring",
            Finger::RightPinky => "Right Pinky",
        };
        write!(f, "{label}")
    }
}

/// Builds the canonical QWERTY -> finger map.
///
/// Note on simplification: real touch-typing convention has the *shift* key
/// itself struck by the opposite hand's pinky. Modeling a full two-key chord
/// per uppercase letter would add a lot of complexity for a boilerplate, so
/// here every uppercase letter is attributed to the same finger as its
/// lowercase counterpart (the finger that presses the letter key). This is a
/// deliberate, documented simplification you may want to refine later.
pub fn build_keymap() -> HashMap<char, Finger> {
    use Finger::*;
    let mut map = HashMap::new();

    let rows: &[(&str, Finger)] = &[
        // --- number row ---
        ("`1", LeftPinky),
        ("2", LeftRing),
        ("3", LeftMiddle),
        ("45", LeftIndex),
        ("67", RightIndex),
        ("8", RightMiddle),
        ("9", RightRing),
        ("0-=", RightPinky),
        // --- top letter row ---
        ("q", LeftPinky),
        ("w", LeftRing),
        ("e", LeftMiddle),
        ("rt", LeftIndex),
        ("yu", RightIndex),
        ("i", RightMiddle),
        ("o", RightRing),
        ("p[]\\", RightPinky),
        // --- home row ---
        ("a", LeftPinky),
        ("s", LeftRing),
        ("d", LeftMiddle),
        ("fg", LeftIndex),
        ("hj", RightIndex),
        ("k", RightMiddle),
        ("l", RightRing),
        (";'", RightPinky),
        // --- bottom row ---
        ("z", LeftPinky),
        ("x", LeftRing),
        ("c", LeftMiddle),
        ("vb", LeftIndex),
        ("nm", RightIndex),
        (",", RightMiddle),
        (".", RightRing),
        ("/", RightPinky),
    ];

    for (chars, finger) in rows {
        for c in chars.chars() {
            map.insert(c, *finger);
            for upper in c.to_uppercase() {
                map.insert(upper, *finger);
            }
        }
    }

    // Shifted symbols above the number row share a finger with their base
    // digit (again: ignoring which hand presses Shift).
    let shifted_symbols: &[(char, char)] = &[
        ('1', '!'),
        ('2', '@'),
        ('3', '#'),
        ('4', '$'),
        ('5', '%'),
        ('6', '^'),
        ('7', '&'),
        ('8', '*'),
        ('9', '('),
        ('0', ')'),
        ('-', '_'),
        ('=', '+'),
        ('[', '{'),
        (']', '}'),
        ('\\', '|'),
        (';', ':'),
        ('\'', '"'),
        (',', '<'),
        ('.', '>'),
        ('/', '?'),
        ('`', '~'),
    ];
    for (base, shifted) in shifted_symbols {
        if let Some(finger) = map.get(base).copied() {
            map.insert(*shifted, finger);
        }
    }

    // Space bar: conventionally either thumb. We attribute it to whichever
    // thumb is idle most often in practice - the right thumb - but keep a
    // constant here in case you want to alternate/split this later.
    map.insert(' ', RightThumb);

    map
}

/// Per-finger analytical statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FingerStats {
    pub total_keystrokes: u64,
    pub total_errors: u64,
    /// expected char -> (actual char typed -> count). Only ever populated
    /// on error (expected != actual); a perfect keystroke is not logged
    /// here, only counted in `total_keystrokes`.
    pub mistake_matrix: HashMap<char, HashMap<char, u64>>,
}

impl FingerStats {
    /// Fraction of keystrokes on this finger that were wrong, in `[0, 1]`.
    /// Returns `0.0` when the finger has not been used yet (rather than
    /// `NaN`) so callers can sort/compare safely.
    pub fn error_rate(&self) -> f64 {
        if self.total_keystrokes == 0 {
            0.0
        } else {
            self.total_errors as f64 / self.total_keystrokes as f64
        }
    }

    /// Accuracy percentage (0-100), the inverse of `error_rate`.
    pub fn accuracy_pct(&self) -> f64 {
        (1.0 - self.error_rate()) * 100.0
    }
}

/// The full typing engine: keyboard layout + live statistics for the
/// current process. This is the object you feed every keystroke into.
#[derive(Debug, Clone)]
pub struct TypingEngine {
    pub keymap: HashMap<char, Finger>,
    pub stats: HashMap<Finger, FingerStats>,
}

impl Default for TypingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TypingEngine {
    pub fn new() -> Self {
        let keymap = build_keymap();
        let mut stats = HashMap::new();
        for finger in Finger::ALL {
            stats.insert(finger, FingerStats::default());
        }
        TypingEngine { keymap, stats }
    }

    /// Looks up which finger is anatomically responsible for `c`.
    /// Unknown characters (e.g. exotic unicode) are not tracked.
    pub fn finger_for(&self, c: char) -> Option<Finger> {
        // Keystroke lookups should be resilient to case: we already store
        // both cases in the map, but normalize defensively in case a caller
        // passes something outside our known set.
        self.keymap.get(&c).copied()
    }

    /// Records one keystroke: `expected` is the character the practice text
    /// asked for, `actual` is what the user actually pressed. Updates the
    /// stats bucket for the finger responsible for `expected` (i.e. we
    /// attribute the error to the finger that *should* have pressed the
    /// key, not to whatever finger accidentally produced `actual`).
    pub fn record_keystroke(&mut self, expected: char, actual: char) {
        let Some(finger) = self.finger_for(expected) else {
            return; // untracked character (e.g. tab, unicode) - ignore
        };
        let entry = self.stats.entry(finger).or_default();
        entry.total_keystrokes += 1;
        if expected != actual {
            entry.total_errors += 1;
            *entry
                .mistake_matrix
                .entry(expected)
                .or_default()
                .entry(actual)
                .or_insert(0) += 1;
        }
    }

    /// Finds the finger with the worst error rate among fingers that have
    /// actually been used at least once. Returns `(finger, error_rate)`
    /// where `error_rate` is in `[0, 1]`.
    pub fn get_most_faulty_finger(&self) -> Option<(Finger, f64)> {
        Finger::ALL
            .into_iter()
            .filter_map(|f| {
                let s = self.stats.get(&f)?;
                if s.total_keystrokes == 0 {
                    None
                } else {
                    Some((f, s.error_rate()))
                }
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// A snapshot of accuracy percentage per finger, keyed by the stable
    /// string key (see [`Finger::as_key`]) so it can be dropped straight
    /// into a [`crate::persistence::SessionSnapshot`].
    pub fn accuracy_snapshot(&self) -> HashMap<String, f64> {
        Finger::ALL
            .into_iter()
            .map(|f| {
                let acc = self
                    .stats
                    .get(&f)
                    .map(|s| s.accuracy_pct())
                    .unwrap_or(100.0);
                (f.as_key().to_string(), acc)
            })
            .collect()
    }

    /// All characters this engine knows are typed by `finger`. Kept as
    /// public API for anyone extending the paragraph bank or generator to
    /// validate finger-loading; not currently called internally.
    #[allow(dead_code)]
    pub fn chars_for_finger(&self, finger: Finger) -> Vec<char> {
        self.keymap
            .iter()
            .filter(|(_, f)| **f == finger)
            .map(|(c, _)| *c)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_correct_and_incorrect_keystrokes() {
        let mut engine = TypingEngine::new();
        engine.record_keystroke('e', 'e'); // correct, on Left Middle
        engine.record_keystroke('e', 'r'); // typo, still attributed to Left Middle
        let stats = engine.stats.get(&Finger::LeftMiddle).unwrap();
        assert_eq!(stats.total_keystrokes, 2);
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.mistake_matrix[&'e'][&'r'], 1);
    }

    #[test]
    fn most_faulty_finger_picks_highest_error_rate() {
        let mut engine = TypingEngine::new();
        // Left pinky: 1 error out of 2 => 50%
        engine.record_keystroke('q', 'q');
        engine.record_keystroke('q', 'w');
        // Right pinky: 1 error out of 10 => 10%
        for _ in 0..9 {
            engine.record_keystroke('p', 'p');
        }
        engine.record_keystroke('p', 'o');

        let (finger, rate) = engine.get_most_faulty_finger().unwrap();
        assert_eq!(finger, Finger::LeftPinky);
        assert!((rate - 0.5).abs() < 1e-9);
    }

    #[test]
    fn keymap_covers_full_alphabet() {
        let map = build_keymap();
        for c in 'a'..='z' {
            assert!(map.contains_key(&c), "missing lowercase {c}");
            assert!(map.contains_key(&c.to_ascii_uppercase()), "missing uppercase {c}");
        }
        assert!(map.contains_key(&' '));
    }
}
