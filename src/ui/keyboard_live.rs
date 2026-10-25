//! Live Interactive ASCII / Unicode Keyboard & Thermal Heatmap widget.
//!
//! Provides real-time proprioceptive feedback during typing:
//! - Active key highlighting & touch-typing finger guidance
//! - Opposite-hand shift key chording visualization
//! - Dynamic thermal decay algorithm ($H_k \in [0.0, 1.0]$) reflecting keystroke errors

use std::collections::HashMap;
use std::time::Instant;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::finger::{Finger, KeyboardLayout};
use crate::theme::Theme;

/// Thermal energy state of a single physical key.
#[derive(Debug, Clone)]
pub struct KeyThermalState {
    #[allow(dead_code)]
    pub key: char,
    pub heat: f64, // 0.0 (cold) to 1.0 (critical heat)
    pub last_error_at: Option<Instant>,
}

/// Global thermal state across all physical keys on the keyboard.
#[derive(Debug, Clone, Default)]
pub struct LiveKeyboardState {
    pub keys: HashMap<char, KeyThermalState>,
}

impl LiveKeyboardState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Spikes the thermal heat on the given key upon mistake: $H_k \leftarrow \min(1.0, H_k + 0.4)$.
    pub fn record_error(&mut self, key: char) {
        let (base, _) = unshift_char(key);
        let entry = self.keys.entry(base).or_insert_with(|| KeyThermalState {
            key: base,
            heat: 0.0,
            last_error_at: None,
        });
        entry.heat = (entry.heat + 0.4).min(1.0);
        entry.last_error_at = Some(Instant::now());
    }

    /// Gets current thermal heat for a key in `[0.0, 1.0]`.
    pub fn heat_for(&self, key: char) -> f64 {
        let (base, _) = unshift_char(key);
        self.keys.get(&base).map(|s| s.heat).unwrap_or(0.0)
    }

    /// Decays thermal heat across all keys over elapsed `dt_secs` ($\lambda = 0.15\text{ s}^{-1}$).
    pub fn decay(&mut self, dt_secs: f64) {
        if dt_secs <= 0.0 {
            return;
        }
        let decay_amount = 0.15 * dt_secs;
        for entry in self.keys.values_mut() {
            entry.heat = (entry.heat - decay_amount).max(0.0);
        }
    }

    /// Resets all key heat levels back to zero (for new round / retry).
    pub fn reset_heat(&mut self) {
        for entry in self.keys.values_mut() {
            entry.heat = 0.0;
            entry.last_error_at = None;
        }
    }

    /// Returns the key with the highest current thermal heat, if any.
    pub fn hottest_key(&self) -> Option<(char, f64)> {
        self.keys
            .iter()
            .filter(|(_, s)| s.heat > 0.05)
            .max_by(|a, b| a.1.heat.partial_cmp(&b.1.heat).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(c, s)| (*c, s.heat))
    }
}

/// Normalizes a character into its physical base key and whether Shift is required.
pub fn unshift_char(c: char) -> (char, bool) {
    if c.is_ascii_uppercase() {
        (c.to_ascii_lowercase(), true)
    } else {
        match c {
            '~' => ('`', true),
            '!' => ('1', true),
            '@' => ('2', true),
            '#' => ('3', true),
            '$' => ('4', true),
            '%' => ('5', true),
            '^' => ('6', true),
            '&' => ('7', true),
            '*' => ('8', true),
            '(' => ('9', true),
            ')' => ('0', true),
            '_' => ('-', true),
            '+' => ('=', true),
            '{' => ('[', true),
            '}' => (']', true),
            '|' => ('\\', true),
            ':' => (';', true),
            '"' => ('\'', true),
            '<' => (',', true),
            '>' => ('.', true),
            '?' => ('/', true),
            other => (other.to_ascii_lowercase(), false),
        }
    }
}

/// Identifies if a key is a tactile home-row anchor for a given layout.
fn is_home_row_anchor(c: char, layout: KeyboardLayout) -> bool {
    let lower = c.to_ascii_lowercase();
    match layout {
        KeyboardLayout::Qwerty => lower == 'f' || lower == 'j',
        KeyboardLayout::Colemak | KeyboardLayout::ColemakDh => lower == 't' || lower == 'n',
        KeyboardLayout::Dvorak => lower == 'u' || lower == 'h',
    }
}

/// Generates the 5 keyboard rows for the specified layout.
fn layout_rows(layout: KeyboardLayout) -> [&'static [&'static str]; 5] {
    match layout {
        KeyboardLayout::Qwerty => [
            &["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", "Bksp"],
            &["Tab", "q", "w", "e", "r", "t", "y", "u", "i", "o", "p", "[", "]", "\\"],
            &["Caps", "a", "s", "d", "f", "g", "h", "j", "k", "l", ";", "'", "Enter"],
            &["Shift", "z", "x", "c", "v", "b", "n", "m", ",", ".", "/", "Shift"],
            &["Ctrl", "Alt", "Space", "Alt", "Ctrl"],
        ],
        KeyboardLayout::Colemak => [
            &["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", "Bksp"],
            &["Tab", "q", "w", "f", "p", "g", "j", "l", "u", "y", ";", "[", "]", "\\"],
            &["Caps", "a", "r", "s", "t", "d", "h", "n", "e", "i", "o", "'", "Enter"],
            &["Shift", "z", "x", "c", "v", "b", "k", "m", ",", ".", "/", "Shift"],
            &["Ctrl", "Alt", "Space", "Alt", "Ctrl"],
        ],
        KeyboardLayout::ColemakDh => [
            &["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "=", "Bksp"],
            &["Tab", "q", "w", "f", "p", "b", "j", "l", "u", "y", "'", "[", "]", "\\"],
            &["Caps", "a", "r", "s", "t", "g", "m", "n", "e", "i", "o", ";", "Enter"],
            &["Shift", "z", "x", "c", "d", "v", "k", "h", ",", ".", "/", "Shift"],
            &["Ctrl", "Alt", "Space", "Alt", "Ctrl"],
        ],
        KeyboardLayout::Dvorak => [
            &["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "[", "]", "Bksp"],
            &["Tab", "'", ",", ".", "p", "y", "f", "g", "c", "r", "l", "/", "=", "\\"],
            &["Caps", "a", "o", "e", "u", "i", "d", "h", "t", "n", "s", "-", "Enter"],
            &["Shift", ";", "q", "j", "k", "x", "b", "m", "w", "v", "z", "Shift"],
            &["Ctrl", "Alt", "Space", "Alt", "Ctrl"],
        ],
    }
}

/// Renders the live interactive keyboard & thermal heatmap.
pub fn draw_live_keyboard(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    if area.height < 3 || area.width < 40 {
        return;
    }

    // Identify active and next expected characters
    let active_char = app.target.get(app.typed.len()).copied();
    let next_char = app.target.get(app.typed.len() + 1).copied();

    let (active_base, is_shifted) = match active_char {
        Some(c) => {
            let (b, s) = unshift_char(c);
            (Some(b), s)
        }
        None => (None, false),
    };

    // Determine opposite-hand shift requirement
    let active_finger = active_char.and_then(|c| {
        let prev = if !app.typed.is_empty() {
            app.typed.last().map(|t| t.expected)
        } else {
            None
        };
        app.engine.finger_for_context(c, prev)
    });

    let (need_left_shift, need_right_shift) = if is_shifted {
        match active_finger {
            Some(f) if f.is_left_hand() => (false, true),  // Right shift for left hand key
            Some(f) if f.is_right_hand() => (true, false), // Left shift for right hand key
            _ => (false, true),
        }
    } else {
        (false, false)
    };

    // If height is compact (< 7 rows), render the 1-line or 2-line condensed strip
    if area.height < 7 {
        draw_compact_strip(frame, area, app, theme, active_char, active_finger, is_shifted);
        return;
    }

    let block = theme.block("Keyboard Biofeedback");
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Split horizontally: Left for Keyboard layout, Right for Status panel
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(45), Constraint::Min(25)])
        .split(inner_area);

    // Render Keyboard Rows
    let rows = layout_rows(app.layout);
    let mut keyboard_lines: Vec<Line> = Vec::with_capacity(5);

    for key_row in rows.iter() {
        let mut spans: Vec<Span> = Vec::new();

        for &key_str in *key_row {
            let is_space_key = key_str == "Space";
            let is_shift_key = key_str == "Shift";
            let is_l_shift = is_shift_key && spans.is_empty();
            let is_r_shift = is_shift_key && !spans.is_empty();

            let is_active_key = if is_space_key {
                active_char == Some(' ')
            } else if is_l_shift {
                need_left_shift
            } else if is_r_shift {
                need_right_shift
            } else if key_str.len() == 1 {
                let c = key_str.chars().next().unwrap();
                active_base == Some(c)
            } else {
                false
            };

            let heat = if key_str.len() == 1 {
                app.keyboard_state.heat_for(key_str.chars().next().unwrap())
            } else {
                0.0
            };

            let is_anchor = key_str.len() == 1 && is_home_row_anchor(key_str.chars().next().unwrap(), app.layout);

            // Determine styling based on active state and thermal heat
            let style = if is_active_key {
                Style::default()
                    .fg(Color::Black)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if heat > 0.7 {
                Style::default()
                    .fg(Color::White)
                    .bg(theme.error)
                    .add_modifier(Modifier::BOLD)
            } else if heat > 0.3 {
                Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)
            } else if heat > 0.05 {
                Style::default().fg(theme.brand)
            } else if is_anchor {
                Style::default().fg(theme.title).add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(theme.untyped)
            };

            let display_str = if is_space_key {
                "  [  SPACE  ]  "
            } else if is_shift_key {
                "[Shift]"
            } else {
                key_str
            };

            spans.push(Span::styled(format!(" {display_str}"), style));
        }

        keyboard_lines.push(Line::from(spans));
    }

    let keyboard_para = Paragraph::new(keyboard_lines);
    frame.render_widget(keyboard_para, cols[0]);

    // Render Right-Side Biofeedback Details
    let active_desc = match (active_char, active_finger) {
        (Some(c), Some(f)) => format!("Active: [{}] -> {}", char_display(c), f),
        (Some(c), None) => format!("Active: [{}]", char_display(c)),
        (None, _) => "Active: Round Complete".to_string(),
    };

    let next_desc = match next_char {
        Some(c) => {
            let prev = active_char;
            let f_str = app.engine.finger_for_context(c, prev)
                .map(|f| f.to_string())
                .unwrap_or_else(|| "—".to_string());
            format!("Next:   [{}] -> {}", char_display(c), f_str)
        }
        None => "Next:   —".to_string(),
    };

    let chord_desc = if is_shifted {
        if need_left_shift {
            "Chord:  Left Shift (Right Hand Key)".to_string()
        } else {
            "Chord:  Right Shift (Left Hand Key)".to_string()
        }
    } else {
        "Chord:  None (Unshifted)".to_string()
    };

    let heat_desc = match app.keyboard_state.hottest_key() {
        Some((k, heat)) => {
            let tag = if heat > 0.7 { "Critical Glow" } else if heat > 0.3 { "Hot" } else { "Warm" };
            format!("Mistake Heat: [{k}] {tag} ({:.0}%)", heat * 100.0)
        }
        None => "Mistake Heat: All Keys Cool".to_string(),
    };

    let layout_desc = format!("Layout: {} (Home Anchor)", app.layout.label());

    let side_lines = vec![
        Line::from(Span::styled(active_desc, Style::default().fg(theme.correct).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(next_desc, Style::default().fg(theme.accent))),
        Line::from(Span::styled(chord_desc, Style::default().fg(theme.warning))),
        Line::from(Span::styled(heat_desc, Style::default().fg(theme.error))),
        Line::from(Span::styled(layout_desc, Style::default().fg(theme.muted))),
    ];

    let side_para = Paragraph::new(side_lines);
    frame.render_widget(side_para, cols[1]);
}

fn char_display(c: char) -> String {
    if c == ' ' { "␣".to_string() } else { c.to_string() }
}

fn draw_compact_strip(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    theme: &Theme,
    active_char: Option<char>,
    active_finger: Option<Finger>,
    is_shifted: bool,
) {
    let active_str = match (active_char, active_finger) {
        (Some(c), Some(f)) => format!("KEY: [{}] ({f})", char_display(c)),
        (Some(c), None) => format!("KEY: [{}]", char_display(c)),
        (None, _) => "KEY: —".to_string(),
    };

    let chord_str = if is_shifted { "CHORD: [Shift]" } else { "CHORD: None" };
    let heat_str = match app.keyboard_state.hottest_key() {
        Some((k, h)) => format!("HEAT: [{k}] {:.0}%", h * 100.0),
        None => "HEAT: Cool".to_string(),
    };

    let line = Line::from(vec![
        Span::styled(format!("{active_str}   "), Style::default().fg(theme.correct).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{chord_str}   "), Style::default().fg(theme.warning)),
        Span::styled(format!("LAYOUT: {}   ", app.layout.label()), Style::default().fg(theme.accent)),
        Span::styled(heat_str, Style::default().fg(theme.error)),
    ]);

    let para = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unshift_char() {
        assert_eq!(unshift_char('A'), ('a', true));
        assert_eq!(unshift_char('a'), ('a', false));
        assert_eq!(unshift_char('!'), ('1', true));
        assert_eq!(unshift_char('1'), ('1', false));
        assert_eq!(unshift_char('{'), ('[', true));
        assert_eq!(unshift_char(' '), (' ', false));
    }

    #[test]
    fn test_keyboard_thermal_decay() {
        let mut state = LiveKeyboardState::new();
        assert_eq!(state.heat_for('e'), 0.0);

        state.record_error('E');
        assert!((state.heat_for('e') - 0.4).abs() < 1e-6);

        state.record_error('e');
        assert!((state.heat_for('e') - 0.8).abs() < 1e-6);

        // Decay 2 seconds: lambda = 0.15, decay = 0.30
        state.decay(2.0);
        assert!((state.heat_for('e') - 0.5).abs() < 1e-6);

        state.reset_heat();
        assert_eq!(state.heat_for('e'), 0.0);
    }
}
