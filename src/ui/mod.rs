//! All ratatui rendering lives here, kept separate from state/event
//! handling so the drawing code stays pure: `draw` only reads `&App`
//! (aside from the two stateful widgets - the finger list - which need
//! `&mut` for their `ListState`).

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph, Sparkline, Tabs, Wrap},
    Frame,
};

use crate::app::{App, DashboardTab, Mode, SetupField, ERROR_FLASH_DURATION};
use crate::theme::Theme;

pub mod debrief;
pub mod finger_load;
pub mod history_table;
pub mod keyboard_heatmap;
pub mod keyboard_live;
pub mod levels;
pub mod metronome;
pub mod trends;

const MIN_WIDTH: u16 = 70;
const MIN_HEIGHT: u16 = 20;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();
    let theme = app.theme();

    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        draw_too_small(frame, size, &theme);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(size);

    draw_header(frame, chunks[0], app, &theme);
    match app.mode {
        Mode::Consent => draw_consent(frame, chunks[1], &theme),
        Mode::Setup => draw_setup(frame, chunks[1], app, &theme),
        Mode::Typing => draw_typing(frame, chunks[1], app, &theme),
        Mode::Debrief => debrief::draw_debrief(frame, chunks[1], app, &theme),
        Mode::Dashboard => draw_dashboard(frame, chunks[1], app, &theme),
    }
    draw_footer(frame, chunks[2], app, &theme);
}

fn draw_too_small(frame: &mut Frame, area: Rect, theme: &Theme) {
    let msg = format!(
        "Terminal too small.\nResize to at least {MIN_WIDTH}x{MIN_HEIGHT} to continue."
    );
    let para = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.warning));
    frame.render_widget(para, area);
}

/// Computes a `Rect` of exactly `height` rows, horizontally centered at
/// `percent_x`% of `area`'s width, and vertically centered within `area`.
/// The standard ratatui "centered box" recipe.
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let height = height.min(area.height);
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}

const YUBI_ASCII_LOGO: &[&str] = &[
    "██╗   ██╗██╗   ██╗██████╗ ██╗",
    "╚██╗ ██╔╝██║   ██║██╔══██╗██║",
    " ╚████╔╝ ██║   ██║██████╔╝██║",
    "  ╚██╔╝  ██║   ██║██╔══██╗██║",
    "   ██║   ╚██████╔╝██████╔╝██║",
    "   ╚═╝    ╚═════╝ ╚═════╝ ╚═╝",
];

fn draw_header(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = match app.mode {
        Mode::Consent => "Yubi — Welcome",
        Mode::Setup => "Yubi — Setup",
        Mode::Typing => "Yubi — Practice Round",
        Mode::Debrief => "Yubi — Instant Replay Debrief",
        Mode::Dashboard => "Yubi — Progress Dashboard",
    };
    let subtitle = match app.mode {
        Mode::Consent => "y: yes, save my progress   n: no, don't save   F2: theme   Esc: quit".to_string(),
        Mode::Setup => "↑/↓ or Tab: switch field   ←/→: change value   Enter: start   F2: theme   Esc: quit".to_string(),
        Mode::Typing => format!(
            "{} • {} • {} • {} • F2: theme • Esc: quit • Backspace/Ctrl+W: correct",
            app.layout.label(),
            app.level.label(),
            app.training_mode.label(),
            app.round_length.label()
        ),
        Mode::Debrief => "r: retry same text • w: drill weak • Enter: next round • Tab: dashboard • Esc: setup".to_string(),
        Mode::Dashboard => format!(
            "Total Sessions: {}  •  F2: Theme ({})  •  r: Setup  •  Esc/q: Quit",
            app.history.sessions.len(),
            app.theme_id.label()
        ),
    };
    let para = Paragraph::new(subtitle)
        .block(theme.block(title))
        .alignment(Alignment::Left);
    frame.render_widget(para, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        match app.mode {
            Mode::Dashboard => match app.dashboard_tab {
                DashboardTab::KeyboardHeatmap => {
                    "[1-5] Tab  •  [t] Time  •  [l] Level  •  [m] Mode  •  [h] Heatmap  •  [↑↓←→] Key  •  [r] Setup  •  [q] Quit".to_string()
                }
                DashboardTab::SessionHistory => {
                    "[1-5] Tab  •  [t] Time  •  [l] Level  •  [m] Mode  •  [↑↓] Row  •  [PgUp/PgDn] Page  •  [Home/End] Edge  •  [r] Setup  •  [q] Quit".to_string()
                }
                _ => {
                    "[1-5] Tab  •  [Tab/Shift+Tab] Next/Prev  •  [t] Time  •  [l] Level  •  [m] Mode  •  [r] Setup  •  [q] Quit".to_string()
                }
            },
            Mode::Typing => {
                "Type text shown above. Ctrl+W delete word, Ctrl+R restart, Esc quit.".to_string()
            }
            Mode::Setup => {
                "↑/↓/Tab: Focus  •  ←/→: Cycle  •  Enter: Start Round  •  F2: Theme  •  Esc: Quit".to_string()
            }
            Mode::Debrief => {
                "r: retry same round  •  w: drill weakest finger  •  Enter: next round  •  Tab: dashboard  •  Esc: setup".to_string()
            }
            Mode::Consent => "Ready.".to_string(),
        }
    };
    let para = Paragraph::new(text).style(Style::default().fg(theme.muted));
    frame.render_widget(para, area);
}

// ---------------------------------------------------------------------
// Consent screen (first launch only)
// ---------------------------------------------------------------------

fn draw_consent(frame: &mut Frame, area: Rect, theme: &Theme) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(11),
        ])
        .split(area);

    let logo_text: Vec<Line> = YUBI_ASCII_LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(theme.brand).add_modifier(Modifier::BOLD))))
        .collect();
    let logo_para = Paragraph::new(logo_text).alignment(Alignment::Center);
    frame.render_widget(logo_para, main_layout[0]);

    let block_area = centered_rect(70, 11, main_layout[1]);
    let text = vec![
        Line::from("Track your progress across sessions?"),
        Line::from(""),
        Line::from(
            "If yes, each round's per-finger accuracy and WPM is saved to a small",
        ),
        Line::from(
            "JSON file on this machine to visualize your improvement in the dashboard.",
        ),
        Line::from("If no, nothing is written to disk — rounds still work, but local"),
        Line::from("progress tracking is unavailable."),
        Line::from(""),
        Line::from(Span::styled(
            "y: yes, track progress locally      n: no, don't track",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        )),
    ];
    let para = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(theme.block("Before we start"));
    frame.render_widget(para, block_area);
}

// ---------------------------------------------------------------------
// Setup screen
// ---------------------------------------------------------------------

fn draw_setup(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(12),
        ])
        .split(area);

    let logo_text: Vec<Line> = YUBI_ASCII_LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(theme.brand).add_modifier(Modifier::BOLD))))
        .collect();
    let logo_para = Paragraph::new(logo_text).alignment(Alignment::Center);
    frame.render_widget(logo_para, main_layout[0]);

    let block_area = centered_rect(60, 20, main_layout[1]);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(block_area);

    let level_block = if app.setup_focus == SetupField::Level {
        theme.block_focused("Difficulty (focused)")
    } else {
        theme.block("Difficulty")
    };
    let level_text = format!("◀   {}   ▶", app.level.label());
    let level_para = Paragraph::new(level_text)
        .alignment(Alignment::Center)
        .block(level_block);
    frame.render_widget(level_para, rows[0]);

    let training_block = if app.setup_focus == SetupField::Training {
        theme.block_focused("Training Mode (focused)")
    } else {
        theme.block("Training Mode")
    };
    let training_text = format!("◀   {}   ▶", app.training_mode.label());
    let training_para = Paragraph::new(training_text)
        .alignment(Alignment::Center)
        .block(training_block);
    frame.render_widget(training_para, rows[1]);

    let length_block = if app.setup_focus == SetupField::Length {
        theme.block_focused("Round Length (focused)")
    } else {
        theme.block("Round Length")
    };
    let length_text = format!("◀   {}   ▶", app.round_length.label());
    let length_para = Paragraph::new(length_text)
        .alignment(Alignment::Center)
        .block(length_block);
    frame.render_widget(length_para, rows[2]);

    let layout_block = if app.setup_focus == SetupField::Layout {
        theme.block_focused("Keyboard Layout (focused)")
    } else {
        theme.block("Keyboard Layout")
    };
    let layout_text = format!("◀   {}   ▶", app.layout.label());
    let layout_para = Paragraph::new(layout_text)
        .alignment(Alignment::Center)
        .block(layout_block);
    frame.render_widget(layout_para, rows[3]);

    let thumb_block = if app.setup_focus == SetupField::Thumb {
        theme.block_focused("Spacebar Thumb (focused)")
    } else {
        theme.block("Spacebar Thumb")
    };
    let thumb_text = format!("◀   {}   ▶", app.thumb_preference.label());
    let thumb_para = Paragraph::new(thumb_text)
        .alignment(Alignment::Center)
        .block(thumb_block);
    frame.render_widget(thumb_para, rows[4]);

    let pacer_block = if app.setup_focus == SetupField::Pacer {
        theme.block_focused("Ghost Pacer (focused)")
    } else {
        theme.block("Ghost Pacer")
    };
    let pacer_text = format!("◀   {}   ▶", app.ghost_pacer.mode.label());
    let pacer_para = Paragraph::new(pacer_text)
        .alignment(Alignment::Center)
        .block(pacer_block);
    frame.render_widget(pacer_para, rows[5]);
}

// ---------------------------------------------------------------------
// Typing view
// ---------------------------------------------------------------------

/// Finds the `[start, end)` char-index bounds of the word containing
/// `cursor`, used to dim everything except the word currently being typed.
fn current_word_bounds(target: &[char], cursor: usize) -> (usize, usize) {
    if target.is_empty() {
        return (0, 0);
    }
    let idx = cursor.min(target.len() - 1);
    let mut start = idx;
    while start > 0 && target[start - 1] != ' ' {
        start -= 1;
    }
    let mut end = idx;
    while end < target.len() && target[end] != ' ' {
        end += 1;
    }
    (start, end)
}

fn draw_typing(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let show_full_keyboard = area.height >= 26 && area.width >= 75;
    let min_height: u16 = if show_full_keyboard { 24 } else { 18 };
    let content_height = area
        .height
        .saturating_sub(1)
        .min(32)
        .max(min_height.min(area.height.saturating_sub(1)));
    let centered = centered_rect(80, content_height, area);

    if show_full_keyboard {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // next-finger hint
                Constraint::Min(3),    // target text
                Constraint::Length(3), // progress gauge
                Constraint::Length(1), // cadence metronome
                Constraint::Length(3), // live stats line
                Constraint::Length(8), // live interactive keyboard & heatmap
            ])
            .split(centered);

        draw_next_finger_hint(frame, rows[0], app, theme);
        draw_target_text(frame, rows[1], app, theme);
        draw_progress_gauge(frame, rows[2], app, theme);
        metronome::draw_cadence_metronome(frame, rows[3], app, theme);
        draw_live_stats(frame, rows[4], app, theme);
        keyboard_live::draw_live_keyboard(frame, rows[5], app, theme);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // next-finger hint
                Constraint::Min(3),    // target text
                Constraint::Length(3), // progress gauge
                Constraint::Length(1), // cadence metronome
                Constraint::Length(3), // live stats line
                Constraint::Length(2), // compact keyboard strip
            ])
            .split(centered);

        draw_next_finger_hint(frame, rows[0], app, theme);
        draw_target_text(frame, rows[1], app, theme);
        draw_progress_gauge(frame, rows[2], app, theme);
        metronome::draw_cadence_metronome(frame, rows[3], app, theme);
        draw_live_stats(frame, rows[4], app, theme);
        keyboard_live::draw_live_keyboard(frame, rows[5], app, theme);
    }
}

fn cursor_line_and_total_lines(target: &[char], cursor: usize, width: usize) -> (u16, u16) {
    if target.is_empty() || width == 0 {
        return (0, 1);
    }
    let mut line: u16 = 0;
    let mut col: usize = 0;
    let mut cursor_line: u16 = 0;

    let mut i = 0;
    while i < target.len() {
        if i == cursor {
            cursor_line = line;
        }
        if target[i] == ' ' {
            col += 1;
            if col >= width {
                line += 1;
                col = 0;
            }
            i += 1;
        } else {
            let mut w_end = i;
            while w_end < target.len() && target[w_end] != ' ' {
                w_end += 1;
            }
            let word_len = w_end - i;
            if col + word_len > width && col > 0 {
                line += 1;
                col = 0;
            }
            while i < w_end {
                if i == cursor {
                    cursor_line = line;
                }
                col += 1;
                i += 1;
            }
        }
    }
    if cursor >= target.len() {
        cursor_line = line;
    }
    (cursor_line, line + 1)
}

fn draw_next_finger_hint(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let prev_char = if !app.typed.is_empty() {
        app.typed.last().map(|t| t.expected)
    } else {
        None
    };
    let text = match app.target.get(app.typed.len()) {
        Some(c) => match app.engine.finger_for_context(*c, prev_char) {
            Some(f) => format!("Next finger: {f}"),
            None => "Next finger: —".to_string(),
        },
        None => "Round complete!".to_string(),
    };
    let para = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD));
    frame.render_widget(para, area);
}

fn draw_target_text(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let (word_start, word_end) = current_word_bounds(&app.target, app.typed.len());

    let ghost_idx = if app.ghost_pacer.is_active() {
        let elapsed = app.session_start.elapsed().as_secs_f64();
        Some(app.ghost_pacer.ghost_char_index(elapsed, app.target.len()))
    } else {
        None
    };

    let slice_start = if app.round_length.is_timed_sprint() && app.typed.len() > 50 {
        let mut s = app.typed.len().saturating_sub(35);
        while s > 0 && app.target.get(s) != Some(&' ') {
            s -= 1;
        }
        if app.target.get(s) == Some(&' ') { s + 1 } else { s }
    } else {
        0
    };

    let mut spans: Vec<Span> = Vec::with_capacity(app.target.len() - slice_start);
    for (i, expected) in app.target.iter().enumerate().skip(slice_start) {
        let in_current_word = i >= word_start && i < word_end;
        let is_ghost = ghost_idx == Some(i);

        let style = if let Some(typed) = app.typed.get(i) {
            if typed.is_correct() {
                if is_ghost {
                    Style::default()
                        .fg(theme.correct)
                        .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
                } else {
                    Style::default().fg(theme.correct)
                }
            } else if typed.at.elapsed() < ERROR_FLASH_DURATION {
                Style::default()
                    .fg(Color::White)
                    .bg(theme.error_flash_bg)
                    .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
            } else {
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::UNDERLINED)
            }
        } else if i == app.typed.len() {
            if is_ghost {
                Style::default()
                    .fg(theme.cursor_fg)
                    .bg(theme.cursor_bg)
                    .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
            } else {
                Style::default().fg(theme.cursor_fg).bg(theme.cursor_bg)
            }
        } else if is_ghost {
            Style::default()
                .fg(theme.brand)
                .bg(theme.cursor_bg)
                .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
        } else if in_current_word {
            Style::default().fg(theme.untyped_current_word)
        } else {
            Style::default().fg(theme.untyped)
        };
        spans.push(Span::styled(expected.to_string(), style));
    }

    let pacer_badge = if app.ghost_pacer.is_active() {
        format!(" [Pacer: {}]", app.ghost_pacer.mode.label())
    } else {
        String::new()
    };
    let mode_badge = if app.round_length.is_timed_sprint() {
        format!(" [{}]", app.round_length.label())
    } else {
        String::new()
    };
    let title = format!("Type this{mode_badge}{pacer_badge}");

    let inner_width = area.width.saturating_sub(2).max(10) as usize;
    let inner_height = area.height.saturating_sub(2);

    let (cursor_line, total_lines) = cursor_line_and_total_lines(&app.target, app.typed.len(), inner_width);

    let scroll_y = if total_lines > inner_height && inner_height > 0 {
        let half = inner_height / 2;
        let ideal = cursor_line.saturating_sub(half);
        let max_scroll = total_lines.saturating_sub(inner_height);
        ideal.min(max_scroll)
    } else {
        0
    };

    let mut paragraph = Paragraph::new(Line::from(spans))
        .block(theme.block(&title))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    if scroll_y > 0 {
        paragraph = paragraph.scroll((scroll_y, 0));
    }
    frame.render_widget(paragraph, area);
}

fn draw_progress_gauge(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    if let Some(limit) = app.round_length.timed_duration_secs() {
        let elapsed = app.session_start.elapsed().as_secs_f64();
        let remaining = (limit - elapsed).max(0.0);
        let pct = ((elapsed / limit) * 100.0).clamp(0.0, 100.0).round() as u16;
        let title = format!("Time Remaining: {:.1}s / {:.0}s", remaining, limit);
        let gauge = Gauge::default()
            .block(theme.block(&title))
            .gauge_style(Style::default().fg(theme.accent))
            .percent(pct.min(100));
        frame.render_widget(gauge, area);
    } else {
        let pct = if app.target.is_empty() {
            0
        } else {
            ((app.typed.len() as f64 / app.target.len() as f64) * 100.0).round() as u16
        };
        let gauge = Gauge::default()
            .block(theme.block("Progress"))
            .gauge_style(Style::default().fg(theme.accent))
            .percent(pct.min(100));
        frame.render_widget(gauge, area);
    }
}

#[allow(dead_code)]
fn draw_wpm_sparkline(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let data: Vec<u64> = app.wpm_samples.iter().copied().collect();
    let spark = Sparkline::default()
        .block(theme.block("WPM (live)"))
        .data(&data)
        .style(Style::default().fg(theme.accent));
    frame.render_widget(spark, area);
}

fn draw_live_stats(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let elapsed = app.session_start.elapsed().as_secs_f64();
    let typed_chars = app.typed.len();
    let errors = app.typed.iter().filter(|t| !t.is_correct()).count();
    let wpm = if elapsed > 0.0 {
        (typed_chars as f64 / 5.0) / (elapsed / 60.0)
    } else {
        0.0
    };

    let pacer_badge = if app.ghost_pacer.is_active() {
        let delta = app.ghost_pacer.delta_wpm(wpm);
        let ghost_idx = app.ghost_pacer.ghost_char_index(elapsed, app.target.len());
        let delta_c = app.ghost_pacer.delta_chars(typed_chars, ghost_idx);
        if delta >= 0.0 {
            format!("   pace: ▲ +{:.1} WPM ({:+} chars)", delta, delta_c)
        } else {
            format!("   pace: ▼ -{:.1} WPM ({:+} chars)", delta.abs(), delta_c)
        }
    } else {
        String::new()
    };

    let stats_line = format!(
        "chars: {}/{}   errors: {}   live wpm: {:.1}{pacer_badge}   elapsed: {:.0}s",
        typed_chars,
        app.target.len(),
        errors,
        wpm,
        elapsed
    );
    let stats = Paragraph::new(stats_line)
        .alignment(Alignment::Center)
        .block(theme.block("Stats"));
    frame.render_widget(stats, area);
}

// ---------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------

fn draw_dashboard(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Summary header & Filter bar
            Constraint::Length(3), // Navigation Tabs
            Constraint::Min(0),    // Active Tab Content
        ])
        .split(area);

    draw_dashboard_header(frame, rows[0], app, theme);
    draw_dashboard_tabs(frame, rows[1], app, theme);

    match app.dashboard_tab {
        DashboardTab::Trends => trends::draw_trends(frame, rows[2], app, theme),
        DashboardTab::KeyboardHeatmap => {
            keyboard_heatmap::draw_keyboard_heatmap(frame, rows[2], app, theme)
        }
        DashboardTab::FingerLoad => finger_load::draw_finger_load(frame, rows[2], app, theme),
        DashboardTab::LevelComparison => levels::draw_levels(frame, rows[2], app, theme),
        DashboardTab::SessionHistory => {
            history_table::draw_history_table(frame, rows[2], app, theme)
        }
    }
}

fn draw_dashboard_header(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let sum = app.filtered_summary();
    let line1 = format!(
        "Sessions: {} • Avg: {:.1} WPM • Peak: {:.1} WPM • Acc: {:.1}% • Strokes: {} ({} errs) • Streak (\u{2265}90%): {}",
        sum.count, sum.avg_wpm, sum.peak_wpm, sum.avg_accuracy, sum.total_keystrokes, sum.total_errors, sum.streak_90
    );
    let line2 = format!(
        "Filters [f]: Time: {} [t]  |  Level: {} [l]  |  Mode: {} [m]   (Showing {} of {} sessions)",
        app.time_filter.label(),
        app.level_filter.label(),
        app.mode_filter.label(),
        sum.count,
        sum.total_count
    );

    let text = vec![
        Line::from(Span::styled(line1, Style::default().fg(Color::White))),
        Line::from(Span::styled(line2, Style::default().fg(theme.accent))),
    ];

    let para = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(theme.block("Yubi — Progress Dashboard"));
    frame.render_widget(para, area);
}

fn draw_dashboard_tabs(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let titles: Vec<Line> = DashboardTab::ALL
        .iter()
        .map(|t| Line::from(t.label()))
        .collect();
    let idx = app.dashboard_tab.index();
    let tabs = Tabs::new(titles)
        .block(theme.block("Navigation (1-5 direct or Tab / Shift+Tab to switch)"))
        .select(idx)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_line_and_total_lines_calculation() {
        let text = "one two three four five six seven eight nine ten";
        let target: Vec<char> = text.chars().collect();
        let width = 15; // "one two three" is 13 chars, fits line 0. "four five six" is 13 chars, fits line 1.
        let (cur_line_0, total_lines) = cursor_line_and_total_lines(&target, 0, width);
        assert_eq!(cur_line_0, 0);
        assert!(total_lines >= 3);

        // Near the end
        let (cur_line_end, _) = cursor_line_and_total_lines(&target, target.len(), width);
        assert!(cur_line_end >= 2);
    }
}
