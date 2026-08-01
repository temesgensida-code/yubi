//! All ratatui rendering lives here, kept separate from state/event
//! handling so the drawing code stays pure: `draw` only reads `&App`
//! (aside from the two stateful widgets - the finger list - which need
//! `&mut` for their `ListState`).

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Axis, Chart, Dataset, Gauge, GraphType, List, ListItem, Paragraph, Row, Sparkline, Table,
        Tabs, Wrap,
    },
    symbols, Frame,
};

use crate::app::{App, DashboardTab, Mode, SetupField, ERROR_FLASH_DURATION};
use crate::theme::{self, THEME};

const MIN_WIDTH: u16 = 70;
const MIN_HEIGHT: u16 = 20;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
        draw_too_small(frame, size);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(size);

    draw_header(frame, chunks[0], app);
    match app.mode {
        Mode::Consent => draw_consent(frame, chunks[1]),
        Mode::Setup => draw_setup(frame, chunks[1], app),
        Mode::Typing => draw_typing(frame, chunks[1], app),
        Mode::Dashboard => draw_dashboard(frame, chunks[1], app),
    }
    draw_footer(frame, chunks[2], app);
}

fn draw_too_small(frame: &mut Frame, area: Rect) {
    let msg = format!(
        "Terminal too small.\nResize to at least {MIN_WIDTH}x{MIN_HEIGHT} to continue."
    );
    let para = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(THEME.warning));
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

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = match app.mode {
        Mode::Consent => "Yubi — Welcome",
        Mode::Setup => "Yubi — Setup",
        Mode::Typing => "Yubi — Practice Round",
        Mode::Dashboard => "Yubi — Progress Dashboard",
    };
    let subtitle = match app.mode {
        Mode::Consent => "y: yes, save my progress   n: no, don't save   Esc: quit".to_string(),
        Mode::Setup => "↑/↓ or Tab: switch field   ←/→: change value   Enter: start   Esc: quit".to_string(),
        Mode::Typing => format!(
            "{} • {} • {} • Esc to quit • Backspace to correct",
            app.level.label(),
            app.training_mode.label(),
            app.round_length.label()
        ),
        Mode::Dashboard => format!(
            "sessions recorded: {} • ↑/↓ or click: select finger • Tab: switch view • e: export CSV • r: new round • Esc/q to quit",
            app.history.sessions.len()
        ),
    };
    let para = Paragraph::new(subtitle)
        .block(theme::block(title))
        .alignment(Alignment::Left);
    frame.render_widget(para, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let text = app
        .status_message
        .clone()
        .unwrap_or_else(|| "Ready.".to_string());
    let para = Paragraph::new(text).style(Style::default().fg(THEME.muted));
    frame.render_widget(para, area);
}

// ---------------------------------------------------------------------
// Consent screen (first launch only)
// ---------------------------------------------------------------------

fn draw_consent(frame: &mut Frame, area: Rect) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(11),
        ])
        .split(area);

    let logo_text: Vec<Line> = YUBI_ASCII_LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(THEME.brand).add_modifier(Modifier::BOLD))))
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
            "JSON file on this machine, and you can export it to CSV from the",
        ),
        Line::from("dashboard later. If no, nothing is written to disk — rounds still"),
        Line::from("work, but history and CSV export are unavailable."),
        Line::from(""),
        Line::from(Span::styled(
            "y: yes, track and allow CSV export      n: no, don't track",
            Style::default().fg(THEME.accent).add_modifier(Modifier::BOLD),
        )),
    ];
    let para = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(theme::block("Before we start"));
    frame.render_widget(para, block_area);
}

// ---------------------------------------------------------------------
// Setup screen
// ---------------------------------------------------------------------

fn draw_setup(frame: &mut Frame, area: Rect, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(12),
        ])
        .split(area);

    let logo_text: Vec<Line> = YUBI_ASCII_LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(THEME.brand).add_modifier(Modifier::BOLD))))
        .collect();
    let logo_para = Paragraph::new(logo_text).alignment(Alignment::Center);
    frame.render_widget(logo_para, main_layout[0]);

    let block_area = centered_rect(60, 12, main_layout[1]);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Length(4),
        ])
        .split(block_area);

    let level_block = if app.setup_focus == SetupField::Level {
        theme::block_focused("Difficulty (focused)")
    } else {
        theme::block("Difficulty")
    };
    let level_text = format!("◀   {}   ▶", app.level.label());
    let level_para = Paragraph::new(level_text)
        .alignment(Alignment::Center)
        .block(level_block);
    frame.render_widget(level_para, rows[0]);

    let training_block = if app.setup_focus == SetupField::Training {
        theme::block_focused("Training Mode (focused)")
    } else {
        theme::block("Training Mode")
    };
    let training_text = format!("◀   {}   ▶", app.training_mode.label());
    let training_para = Paragraph::new(training_text)
        .alignment(Alignment::Center)
        .block(training_block);
    frame.render_widget(training_para, rows[1]);

    let length_block = if app.setup_focus == SetupField::Length {
        theme::block_focused("Round Length (focused)")
    } else {
        theme::block("Round Length")
    };
    let length_text = format!("◀   {}   ▶", app.round_length.label());
    let length_para = Paragraph::new(length_text)
        .alignment(Alignment::Center)
        .block(length_block);
    frame.render_widget(length_para, rows[2]);
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

fn draw_typing(frame: &mut Frame, area: Rect, app: &App) {
    let desired_height: u16 = 15; // hint(1) + text(4) + gauge(3) + sparkline(4) + stats(3)
    let content_height = desired_height.min(area.height.saturating_sub(1)).max(10);
    let centered = centered_rect(80, content_height, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // next-finger hint
            Constraint::Min(3),    // target text (flexes with leftover space)
            Constraint::Length(3), // progress gauge
            Constraint::Length(4), // live wpm sparkline
            Constraint::Length(3), // live stats line
        ])
        .split(centered);

    draw_next_finger_hint(frame, rows[0], app);
    draw_target_text(frame, rows[1], app);
    draw_progress_gauge(frame, rows[2], app);
    draw_wpm_sparkline(frame, rows[3], app);
    draw_live_stats(frame, rows[4], app);
}

fn draw_next_finger_hint(frame: &mut Frame, area: Rect, app: &App) {
    let text = match app.target.get(app.typed.len()) {
        Some(c) => match app.engine.finger_for(*c) {
            Some(f) => format!("Next finger: {f}"),
            None => "Next finger: —".to_string(),
        },
        None => "Round complete!".to_string(),
    };
    let para = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(THEME.accent).add_modifier(Modifier::BOLD));
    frame.render_widget(para, area);
}

fn draw_target_text(frame: &mut Frame, area: Rect, app: &App) {
    let (word_start, word_end) = current_word_bounds(&app.target, app.typed.len());

    let mut spans: Vec<Span> = Vec::with_capacity(app.target.len());
    for (i, expected) in app.target.iter().enumerate() {
        let in_current_word = i >= word_start && i < word_end;
        let style = if let Some(typed) = app.typed.get(i) {
            if typed.is_correct() {
                Style::default().fg(THEME.correct)
            } else if typed.at.elapsed() < ERROR_FLASH_DURATION {
                Style::default()
                    .fg(Color::White)
                    .bg(THEME.error_flash_bg)
                    .add_modifier(Modifier::UNDERLINED | Modifier::BOLD)
            } else {
                Style::default()
                    .fg(THEME.error)
                    .add_modifier(Modifier::UNDERLINED)
            }
        } else if i == app.typed.len() {
            Style::default().fg(THEME.cursor_fg).bg(THEME.cursor_bg)
        } else if in_current_word {
            Style::default().fg(THEME.untyped_current_word)
        } else {
            Style::default().fg(THEME.untyped)
        };
        spans.push(Span::styled(expected.to_string(), style));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .block(theme::block("Type this"))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_progress_gauge(frame: &mut Frame, area: Rect, app: &App) {
    let pct = if app.target.is_empty() {
        0
    } else {
        ((app.typed.len() as f64 / app.target.len() as f64) * 100.0).round() as u16
    };
    let gauge = Gauge::default()
        .block(theme::block("Progress"))
        .gauge_style(Style::default().fg(THEME.accent))
        .percent(pct.min(100));
    frame.render_widget(gauge, area);
}

fn draw_wpm_sparkline(frame: &mut Frame, area: Rect, app: &App) {
    let data: Vec<u64> = app.wpm_samples.iter().copied().collect();
    let spark = Sparkline::default()
        .block(theme::block("WPM (live)"))
        .data(&data)
        .style(Style::default().fg(THEME.accent));
    frame.render_widget(spark, area);
}

fn draw_live_stats(frame: &mut Frame, area: Rect, app: &App) {
    let elapsed = app.session_start.elapsed().as_secs_f64();
    let typed_chars = app.typed.len();
    let errors = app.typed.iter().filter(|t| !t.is_correct()).count();
    let wpm = if elapsed > 0.0 {
        (typed_chars as f64 / 5.0) / (elapsed / 60.0)
    } else {
        0.0
    };
    let stats_line = format!(
        "chars: {}/{}   errors: {}   live wpm: {:.1}   elapsed: {:.0}s",
        typed_chars,
        app.target.len(),
        errors,
        wpm,
        elapsed
    );
    let stats = Paragraph::new(stats_line)
        .alignment(Alignment::Center)
        .block(theme::block("Stats"));
    frame.render_widget(stats, area);
}

// ---------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------

fn draw_dashboard(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    draw_summary_header(frame, rows[0], app);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(32), Constraint::Min(20)])
        .split(rows[1]);

    draw_finger_sidebar(frame, cols[0], app);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(cols[1]);

    draw_tabs(frame, right[0], app);
    match app.dashboard_tab {
        DashboardTab::Accuracy => draw_accuracy_chart(frame, right[1], app),
        DashboardTab::Wpm => draw_wpm_chart(frame, right[1], app),
        DashboardTab::Mistakes => draw_mistakes_table(frame, right[1], app),
    }
}

fn draw_summary_header(frame: &mut Frame, area: Rect, app: &App) {
    let sessions = app.history.sessions.len();
    let best_wpm = app.history.best_wpm();
    let streak = app.history.current_streak(90.0);
    let text = format!(
        "Sessions: {sessions}   Best WPM: {best_wpm:.1}   Streak (\u{2265}90% avg accuracy): {streak}"
    );
    let para = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(theme::block("Summary"));
    frame.render_widget(para, area);
}

const SPARK_BLOCKS: [char; 8] = ['\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];

/// Renders a compact inline sparkline as plain Unicode block characters
/// (not the `Sparkline` widget, which can't be embedded inside a `List`
/// row) so the finger sidebar can show a trend next to each entry.
fn text_sparkline(values: &[f64], min: f64, max: f64) -> String {
    if values.is_empty() {
        return "\u{00B7}\u{00B7}\u{00B7}\u{00B7}\u{00B7}\u{00B7}".to_string();
    }
    values
        .iter()
        .map(|v| {
            let clamped = v.clamp(min, max);
            let ratio = if (max - min).abs() < f64::EPSILON {
                0.0
            } else {
                (clamped - min) / (max - min)
            };
            let idx = ((ratio * (SPARK_BLOCKS.len() - 1) as f64).round() as usize)
                .min(SPARK_BLOCKS.len() - 1);
            SPARK_BLOCKS[idx]
        })
        .collect()
}

fn draw_finger_sidebar(frame: &mut Frame, area: Rect, app: &mut App) {
    app.sidebar_area = area;
    let order = app.finger_display_order();
    let items: Vec<ListItem> = order
        .iter()
        .map(|f| {
            let stats = app.engine.stats.get(f);
            let acc = stats.map(|s| s.accuracy_pct()).unwrap_or(100.0);
            let ks = stats.map(|s| s.total_keystrokes).unwrap_or(0);

            let series = app.history.accuracy_series(f.as_key());
            let recent: Vec<f64> = {
                let mut v: Vec<f64> = series.iter().rev().take(10).map(|(_, acc)| *acc).collect();
                v.reverse();
                v
            };
            let spark = text_sparkline(&recent, 50.0, 100.0);

            let is_weak = ks > 0 && acc < 85.0;
            let marker = if is_weak { "\u{26A0} " } else { "  " };
            let label = format!("{marker}{:<12} {}  {:>5.1}%", f.to_string(), spark, acc);
            let style = if is_weak {
                Style::default().fg(THEME.warning)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Span::styled(label, style))
        })
        .collect();

    let list = List::new(items)
        .block(theme::block("Fingers (worst first)"))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(THEME.accent)
                .fg(Color::Black),
        )
        .highlight_symbol("\u{27A4} ");

    frame.render_stateful_widget(list, area, &mut app.finger_list_state);
}

fn draw_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = DashboardTab::ALL
        .iter()
        .map(|t| Line::from(t.label()))
        .collect();
    let idx = DashboardTab::ALL
        .iter()
        .position(|t| *t == app.dashboard_tab)
        .unwrap_or(0);
    let tabs = Tabs::new(titles)
        .block(theme::block("View (Tab to switch)"))
        .select(idx)
        .highlight_style(
            Style::default()
                .fg(THEME.accent)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}

/// Line chart of historical accuracy for the selected finger, drawn with
/// Braille markers so the trend looks smooth even in a small pane.
fn draw_accuracy_chart(frame: &mut Frame, area: Rect, app: &App) {
    let finger = app.selected_finger();
    let series = app.history.accuracy_series(finger.as_key());
    let title = format!("{finger} \u{2014} Accuracy Trend");

    if series.len() < 2 {
        render_needs_more_data(frame, area, &title, app.history.sessions.is_empty());
        return;
    }

    let max_x = (series.len() as f64 - 1.0).max(1.0);
    let dataset = Dataset::default()
        .name(finger.to_string())
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(THEME.accent))
        .data(&series);

    let x_labels = vec![
        Span::raw("0"),
        Span::raw(format!("{:.0}", max_x / 2.0)),
        Span::raw(format!("{max_x:.0}")),
    ];
    let y_labels = vec![Span::raw("50%"), Span::raw("75%"), Span::raw("100%")];

    let chart = Chart::new(vec![dataset])
        .block(theme::block(title))
        .x_axis(
            Axis::default()
                .title("Session #")
                .style(Style::default().fg(THEME.muted))
                .bounds([0.0, max_x])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("Accuracy")
                .style(Style::default().fg(THEME.muted))
                .bounds([50.0, 100.0])
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}

/// Line chart of WPM across every recorded session (not per-finger - WPM
/// is a whole-round metric).
fn draw_wpm_chart(frame: &mut Frame, area: Rect, app: &App) {
    let series = app.history.wpm_series();
    let title = "Words Per Minute \u{2014} Trend".to_string();

    if series.len() < 2 {
        render_needs_more_data(frame, area, &title, app.history.sessions.is_empty());
        return;
    }

    let max_x = (series.len() as f64 - 1.0).max(1.0);
    let max_y = series
        .iter()
        .map(|(_, y)| *y)
        .fold(0.0_f64, f64::max)
        .max(20.0)
        * 1.2;

    let dataset = Dataset::default()
        .name("WPM")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(THEME.correct))
        .data(&series);

    let x_labels = vec![
        Span::raw("0"),
        Span::raw(format!("{:.0}", max_x / 2.0)),
        Span::raw(format!("{max_x:.0}")),
    ];
    let y_labels = vec![
        Span::raw("0"),
        Span::raw(format!("{:.0}", max_y / 2.0)),
        Span::raw(format!("{max_y:.0}")),
    ];

    let chart = Chart::new(vec![dataset])
        .block(theme::block(title))
        .x_axis(
            Axis::default()
                .title("Session #")
                .style(Style::default().fg(THEME.muted))
                .bounds([0.0, max_x])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("WPM")
                .style(Style::default().fg(THEME.muted))
                .bounds([0.0, max_y])
                .labels(y_labels),
        );

    frame.render_widget(chart, area);
}

fn render_needs_more_data(frame: &mut Frame, area: Rect, title: &str, no_sessions_yet: bool) {
    let msg = if no_sessions_yet {
        "No sessions recorded yet. Finish a practice round to see history here."
    } else {
        "Need at least 2 sessions with data here to draw a trend line."
    };
    let para = Paragraph::new(msg)
        .block(theme::block(title.to_string()))
        .wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}

/// Table of the selected finger's most common mistakes: what was expected
/// vs. what was actually typed, and how many times. Pulled straight from
/// `FingerStats::mistake_matrix`, which was already being recorded but
/// never surfaced anywhere in the UI before.
fn draw_mistakes_table(frame: &mut Frame, area: Rect, app: &App) {
    let finger = app.selected_finger();
    let stats = app.engine.stats.get(&finger);

    let mut rows: Vec<(char, char, u64)> = Vec::new();
    if let Some(s) = stats {
        for (expected, wrongs) in &s.mistake_matrix {
            for (actual, count) in wrongs {
                rows.push((*expected, *actual, *count));
            }
        }
    }
    rows.sort_by(|a, b| b.2.cmp(&a.2));
    rows.truncate(10);

    let title = format!("{finger} \u{2014} Mistake Matrix");

    if rows.is_empty() {
        let para = Paragraph::new(format!("No mistakes recorded yet for {finger}. Nice."))
            .block(theme::block(title))
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
        return;
    }

    let header = Row::new(vec!["Expected", "Typed instead", "Count"])
        .style(Style::default().add_modifier(Modifier::BOLD));
    let table_rows: Vec<Row> = rows
        .iter()
        .map(|(expected, actual, count)| {
            Row::new(vec![expected.to_string(), actual.to_string(), count.to_string()])
        })
        .collect();

    let table = Table::new(
        table_rows,
        [
            Constraint::Length(12),
            Constraint::Length(16),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(theme::block(title));

    frame.render_widget(table, area);
}
