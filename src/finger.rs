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

    pub fn is_left_hand(&self) -> bool {
        matches!(
            self,
            Finger::LeftPinky
                | Finger::LeftRing
                | Finger::LeftMiddle
                | Finger::LeftIndex
                | Finger::LeftThumb
        )
    }

    pub fn is_right_hand(&self) -> bool {
        matches!(
            self,
            Finger::RightPinky
                | Finger::RightRing
                | Finger::RightMiddle
                | Finger::RightIndex
                | Finger::RightThumb
        )
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

/// Spacebar thumb preference setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThumbPreference {
    #[default]
    RightOnly,
    LeftOnly,
    AutoAlternate,
}

impl ThumbPreference {
    pub const ALL: [ThumbPreference; 3] = [
        ThumbPreference::RightOnly,
        ThumbPreference::LeftOnly,
        ThumbPreference::AutoAlternate,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ThumbPreference::RightOnly => "Right Thumb Only",
            ThumbPreference::LeftOnly => "Left Thumb Only",
            ThumbPreference::AutoAlternate => "Auto-Alternate (Opposite Hand)",
        }
    }

    pub fn next(&self) -> ThumbPreference {
        let idx = ThumbPreference::ALL.iter().position(|t| t == self).unwrap_or(0);
        ThumbPreference::ALL[(idx + 1) % ThumbPreference::ALL.len()]
    }

    pub fn prev(&self) -> ThumbPreference {
        let idx = ThumbPreference::ALL.iter().position(|t| t == self).unwrap_or(0);
        ThumbPreference::ALL[(idx + ThumbPreference::ALL.len() - 1) % ThumbPreference::ALL.len()]
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

/// Supported keyboard layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum KeyboardLayout {
    #[default]
    Qwerty,
    Colemak,
    ColemakDh,
    Dvorak,
}

impl KeyboardLayout {
    pub const ALL: [KeyboardLayout; 4] = [
        KeyboardLayout::Qwerty,
        KeyboardLayout::Colemak,
        KeyboardLayout::ColemakDh,
        KeyboardLayout::Dvorak,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            KeyboardLayout::Qwerty => "QWERTY (Standard)",
            KeyboardLayout::Colemak => "Colemak",
            KeyboardLayout::ColemakDh => "Colemak-DH (Ergo Mod)",
            KeyboardLayout::Dvorak => "Dvorak",
        }
    }

    pub fn next(&self) -> KeyboardLayout {
        let idx = KeyboardLayout::ALL.iter().position(|l| l == self).unwrap_or(0);
        KeyboardLayout::ALL[(idx + 1) % KeyboardLayout::ALL.len()]
    }

    pub fn prev(&self) -> KeyboardLayout {
        let idx = KeyboardLayout::ALL.iter().position(|l| l == self).unwrap_or(0);
        KeyboardLayout::ALL[(idx + KeyboardLayout::ALL.len() - 1) % KeyboardLayout::ALL.len()]
    }
}

fn populate_keymap(map: &mut HashMap<char, Finger>, rows: &[(&str, Finger)]) {
    for (chars, finger) in rows {
        for c in chars.chars() {
            map.insert(c, *finger);
            for upper in c.to_uppercase() {
                map.insert(upper, *finger);
            }
        }
    }
}

fn apply_shifted_symbols(map: &mut HashMap<char, Finger>) {
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
}

pub fn build_qwerty_keymap() -> HashMap<char, Finger> {
    use Finger::*;
    let mut map = HashMap::new();
    let rows: &[(&str, Finger)] = &[
        ("`1", LeftPinky),
        ("2", LeftRing),
        ("3", LeftMiddle),
        ("45", LeftIndex),
        ("67", RightIndex),
        ("8", RightMiddle),
        ("9", RightRing),
        ("0-=", RightPinky),
        ("q", LeftPinky),
        ("w", LeftRing),
        ("e", LeftMiddle),
        ("rt", LeftIndex),
        ("yu", RightIndex),
        ("i", RightMiddle),
        ("o", RightRing),
        ("p[]\\", RightPinky),
        ("a", LeftPinky),
        ("s", LeftRing),
        ("d", LeftMiddle),
        ("fg", LeftIndex),
        ("hj", RightIndex),
        ("k", RightMiddle),
        ("l", RightRing),
        (";'", RightPinky),
        ("z", LeftPinky),
        ("x", LeftRing),
        ("c", LeftMiddle),
        ("vb", LeftIndex),
        ("nm", RightIndex),
        (",", RightMiddle),
        (".", RightRing),
        ("/", RightPinky),
    ];
    populate_keymap(&mut map, rows);
    apply_shifted_symbols(&mut map);
    map.insert(' ', RightThumb);
    map
}

pub fn build_colemak_keymap() -> HashMap<char, Finger> {
    use Finger::*;
    let mut map = HashMap::new();
    let rows: &[(&str, Finger)] = &[
        ("`1", LeftPinky),
        ("2", LeftRing),
        ("3", LeftMiddle),
        ("45", LeftIndex),
        ("67", RightIndex),
        ("8", RightMiddle),
        ("9", RightRing),
        ("0-=", RightPinky),
        ("q", LeftPinky),
        ("w", LeftRing),
        ("f", LeftMiddle),
        ("pg", LeftIndex),
        ("jl", RightIndex),
        ("u", RightMiddle),
        ("y", RightRing),
        (";[]\\", RightPinky),
        ("a", LeftPinky),
        ("r", LeftRing),
        ("s", LeftMiddle),
        ("td", LeftIndex),
        ("hn", RightIndex),
        ("e", RightMiddle),
        ("i", RightRing),
        ("o'", RightPinky),
        ("z", LeftPinky),
        ("x", LeftRing),
        ("c", LeftMiddle),
        ("vb", LeftIndex),
        ("km", RightIndex),
        (",", RightMiddle),
        (".", RightRing),
        ("/", RightPinky),
    ];
    populate_keymap(&mut map, rows);
    apply_shifted_symbols(&mut map);
    map.insert(' ', RightThumb);
    map
}

pub fn build_colemak_dh_keymap() -> HashMap<char, Finger> {
    use Finger::*;
    let mut map = HashMap::new();
    let rows: &[(&str, Finger)] = &[
        ("`1", LeftPinky),
        ("2", LeftRing),
        ("3", LeftMiddle),
        ("45", LeftIndex),
        ("67", RightIndex),
        ("8", RightMiddle),
        ("9", RightRing),
        ("0-=", RightPinky),
        ("q", LeftPinky),
        ("w", LeftRing),
        ("f", LeftMiddle),
        ("pb", LeftIndex),
        ("jl", RightIndex),
        ("u", RightMiddle),
        ("y", RightRing),
        ("'[]\\", RightPinky),
        ("a", LeftPinky),
        ("r", LeftRing),
        ("s", LeftMiddle),
        ("tg", LeftIndex),
        ("mn", RightIndex),
        ("e", RightMiddle),
        ("i", RightRing),
        ("o;", RightPinky),
        ("z", LeftPinky),
        ("x", LeftRing),
        ("c", LeftMiddle),
        ("dv", LeftIndex),
        ("kh", RightIndex),
        (",", RightMiddle),
        (".", RightRing),
        ("/", RightPinky),
    ];
    populate_keymap(&mut map, rows);
    apply_shifted_symbols(&mut map);
    map.insert(' ', RightThumb);
    map
}

pub fn build_dvorak_keymap() -> HashMap<char, Finger> {
    use Finger::*;
    let mut map = HashMap::new();
    let rows: &[(&str, Finger)] = &[
        ("`1", LeftPinky),
        ("2", LeftRing),
        ("3", LeftMiddle),
        ("45", LeftIndex),
        ("67", RightIndex),
        ("8", RightMiddle),
        ("9", RightRing),
        ("0[]", RightPinky),
        ("'", LeftPinky),
        (",", LeftRing),
        (".", LeftMiddle),
        ("py", LeftIndex),
        ("fg", RightIndex),
        ("c", RightMiddle),
        ("r", RightRing),
        ("l/=\\", RightPinky),
        ("a", LeftPinky),
        ("o", LeftRing),
        ("e", LeftMiddle),
        ("ui", LeftIndex),
        ("dh", RightIndex),
        ("t", RightMiddle),
        ("n", RightRing),
        ("s-", RightPinky),
        (";", LeftPinky),
        ("q", LeftRing),
        ("j", LeftMiddle),
        ("kx", LeftIndex),
        ("bm", RightIndex),
        ("w", RightMiddle),
        ("v", RightRing),
        ("z", RightPinky),
    ];
    populate_keymap(&mut map, rows);
    apply_shifted_symbols(&mut map);
    map.insert(' ', RightThumb);
    map
}

pub fn build_keymap(layout: KeyboardLayout) -> HashMap<char, Finger> {
    match layout {
        KeyboardLayout::Qwerty => build_qwerty_keymap(),
        KeyboardLayout::Colemak => build_colemak_keymap(),
        KeyboardLayout::ColemakDh => build_colemak_dh_keymap(),
        KeyboardLayout::Dvorak => build_dvorak_keymap(),
    }
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

/// Resolves which thumb is responsible for a space character when `AutoAlternate` is enabled.
/// Strikes Space with the opposite hand of the preceding character.
pub fn resolve_space_finger(prev_char: Option<char>, engine: &TypingEngine) -> Finger {
    let Some(c) = prev_char else {
        return Finger::RightThumb;
    };
    if c == ' ' {
        return Finger::RightThumb;
    }
    match engine.keymap.get(&c).copied() {
        Some(f) if f.is_left_hand() => Finger::RightThumb,
        Some(f) if f.is_right_hand() => Finger::LeftThumb,
        _ => Finger::RightThumb,
    }
}

/// The full typing engine: keyboard layout + live statistics for the
/// current process. This is the object you feed every keystroke into.
#[derive(Debug, Clone)]
pub struct TypingEngine {
    pub layout: KeyboardLayout,
    pub keymap: HashMap<char, Finger>,
    pub stats: HashMap<Finger, FingerStats>,
    pub thumb_preference: ThumbPreference,
    pub last_char: Option<char>,
}

impl Default for TypingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TypingEngine {
    pub fn new() -> Self {
        Self::new_with_options(KeyboardLayout::default(), ThumbPreference::default())
    }

    #[allow(dead_code)]
    pub fn new_with_thumb(thumb_preference: ThumbPreference) -> Self {
        Self::new_with_options(KeyboardLayout::default(), thumb_preference)
    }

    pub fn new_with_options(layout: KeyboardLayout, thumb_preference: ThumbPreference) -> Self {
        let keymap = build_keymap(layout);
        let mut stats = HashMap::new();
        for finger in Finger::ALL {
            stats.insert(finger, FingerStats::default());
        }
        TypingEngine {
            layout,
            keymap,
            stats,
            thumb_preference,
            last_char: None,
        }
    }

    pub fn set_layout(&mut self, layout: KeyboardLayout) {
        if self.layout != layout {
            self.layout = layout;
            self.keymap = build_keymap(layout);
        }
    }

    pub fn set_thumb_preference(&mut self, pref: ThumbPreference) {
        self.thumb_preference = pref;
    }

    /// Looks up which finger is responsible for `c`, given the preceding character context.
    pub fn finger_for_context(&self, c: char, prev: Option<char>) -> Option<Finger> {
        if c == ' ' {
            Some(match self.thumb_preference {
                ThumbPreference::RightOnly => Finger::RightThumb,
                ThumbPreference::LeftOnly => Finger::LeftThumb,
                ThumbPreference::AutoAlternate => resolve_space_finger(prev, self),
            })
        } else {
            self.keymap.get(&c).copied()
        }
    }

    /// Looks up which finger is anatomically responsible for `c`.
    /// Uses the last recorded keystroke as context if `c` is a space.
    pub fn finger_for(&self, c: char) -> Option<Finger> {
        self.finger_for_context(c, self.last_char)
    }

    /// Records one keystroke using the last recorded character as context.
    #[allow(dead_code)]
    pub fn record_keystroke(&mut self, expected: char, actual: char) {
        self.record_keystroke_with_context(expected, actual, self.last_char);
    }

    /// Records one keystroke with explicit preceding character context for space resolution.
    pub fn record_keystroke_with_context(&mut self, expected: char, actual: char, prev: Option<char>) {
        let Some(finger) = self.finger_for_context(expected, prev) else {
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
        self.last_char = Some(expected);
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
    #[allow(dead_code)]
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
        for layout in KeyboardLayout::ALL {
            let map = build_keymap(layout);
            for c in 'a'..='z' {
                assert!(map.contains_key(&c), "missing lowercase {c} in {layout:?}");
                assert!(map.contains_key(&c.to_ascii_uppercase()), "missing uppercase {c} in {layout:?}");
            }
            assert!(map.contains_key(&' '), "missing space in {layout:?}");
        }
    }

    #[test]
    fn hand_predicates_partition_all_fingers() {
        for f in Finger::ALL {
            assert_ne!(f.is_left_hand(), f.is_right_hand(), "{f:?} must be left OR right, not both or neither");
        }
    }

    #[test]
    fn thumb_preference_space_attribution() {
        // RightOnly
        let mut right_engine = TypingEngine::new_with_thumb(ThumbPreference::RightOnly);
        right_engine.record_keystroke_with_context(' ', ' ', Some('f'));
        assert_eq!(right_engine.stats[&Finger::RightThumb].total_keystrokes, 1);
        assert_eq!(right_engine.stats[&Finger::LeftThumb].total_keystrokes, 0);

        // LeftOnly
        let mut left_engine = TypingEngine::new_with_thumb(ThumbPreference::LeftOnly);
        left_engine.record_keystroke_with_context(' ', ' ', Some('j'));
        assert_eq!(left_engine.stats[&Finger::LeftThumb].total_keystrokes, 1);
        assert_eq!(left_engine.stats[&Finger::RightThumb].total_keystrokes, 0);

        // AutoAlternate: preceding 'f' (LeftIndex) -> RightThumb
        let mut auto_engine = TypingEngine::new_with_thumb(ThumbPreference::AutoAlternate);
        auto_engine.record_keystroke_with_context(' ', ' ', Some('f'));
        assert_eq!(auto_engine.stats[&Finger::RightThumb].total_keystrokes, 1);

        // AutoAlternate: preceding 'j' (RightIndex) -> LeftThumb
        auto_engine.record_keystroke_with_context(' ', ' ', Some('j'));
        assert_eq!(auto_engine.stats[&Finger::LeftThumb].total_keystrokes, 1);
    }

    #[test]
    fn thumb_preference_cycling() {
        let mut p = ThumbPreference::RightOnly;
        p = p.next();
        assert_eq!(p, ThumbPreference::LeftOnly);
        p = p.next();
        assert_eq!(p, ThumbPreference::AutoAlternate);
        p = p.next();
        assert_eq!(p, ThumbPreference::RightOnly);
        p = p.prev();
        assert_eq!(p, ThumbPreference::AutoAlternate);
    }

    #[test]
    fn layout_finger_attributions() {
        let qwerty = build_keymap(KeyboardLayout::Qwerty);
        let colemak = build_keymap(KeyboardLayout::Colemak);
        let dvorak = build_keymap(KeyboardLayout::Dvorak);

        // QWERTY home row
        assert_eq!(qwerty.get(&'a'), Some(&Finger::LeftPinky));
        assert_eq!(qwerty.get(&'s'), Some(&Finger::LeftRing));
        assert_eq!(qwerty.get(&'d'), Some(&Finger::LeftMiddle));
        assert_eq!(qwerty.get(&'f'), Some(&Finger::LeftIndex));
        assert_eq!(qwerty.get(&'j'), Some(&Finger::RightIndex));

        // Colemak home row: A R S T D | H N E I O
        assert_eq!(colemak.get(&'a'), Some(&Finger::LeftPinky));
        assert_eq!(colemak.get(&'r'), Some(&Finger::LeftRing));
        assert_eq!(colemak.get(&'s'), Some(&Finger::LeftMiddle));
        assert_eq!(colemak.get(&'t'), Some(&Finger::LeftIndex));
        assert_eq!(colemak.get(&'n'), Some(&Finger::RightIndex));
        assert_eq!(colemak.get(&'e'), Some(&Finger::RightMiddle));
        assert_eq!(colemak.get(&'i'), Some(&Finger::RightRing));
        assert_eq!(colemak.get(&'o'), Some(&Finger::RightPinky));

        // Dvorak home row: A O E U I | D H T N S
        assert_eq!(dvorak.get(&'a'), Some(&Finger::LeftPinky));
        assert_eq!(dvorak.get(&'o'), Some(&Finger::LeftRing));
        assert_eq!(dvorak.get(&'e'), Some(&Finger::LeftMiddle));
        assert_eq!(dvorak.get(&'u'), Some(&Finger::LeftIndex));
        assert_eq!(dvorak.get(&'d'), Some(&Finger::RightIndex));
        assert_eq!(dvorak.get(&'t'), Some(&Finger::RightMiddle));
        assert_eq!(dvorak.get(&'n'), Some(&Finger::RightRing));
        assert_eq!(dvorak.get(&'s'), Some(&Finger::RightPinky));
    }

    #[test]
    fn layout_cycling() {
        let mut l = KeyboardLayout::Qwerty;
        l = l.next();
        assert_eq!(l, KeyboardLayout::Colemak);
        l = l.next();
        assert_eq!(l, KeyboardLayout::ColemakDh);
        l = l.next();
        assert_eq!(l, KeyboardLayout::Dvorak);
        l = l.next();
        assert_eq!(l, KeyboardLayout::Qwerty);
        l = l.prev();
        assert_eq!(l, KeyboardLayout::Dvorak);
    }
}
