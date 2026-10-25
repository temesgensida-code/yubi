//! Tab 4: Performance breakdown by Difficulty Level and Training Mode.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::App;
use crate::persistence::SessionSnapshot;
use crate::theme::Theme;

#[derive(Default)]
struct CategoryMetrics {
    sessions: usize,
    sum_wpm: f64,
    peak_wpm: f64,
    sum_acc: f64,
    keystrokes: u64,
    errors: u64,
}

impl CategoryMetrics {
    fn add(&mut self, s: &SessionSnapshot) {
        self.sessions += 1;
        self.sum_wpm += s.wpm;
        if s.wpm > self.peak_wpm {
            self.peak_wpm = s.wpm;
        }
        self.sum_acc += s.overall_accuracy.unwrap_or(0.0);
        self.keystrokes += s.total_keystrokes.unwrap_or(0);
        self.errors += s.error_count.unwrap_or(0);
    }

    fn avg_wpm(&self) -> f64 {
        if self.sessions > 0 {
            self.sum_wpm / self.sessions as f64
        } else {
            0.0
        }
    }

    fn avg_acc(&self) -> f64 {
        if self.sessions > 0 {
            self.sum_acc / self.sessions as f64
        } else {
            0.0
        }
    }

    fn error_rate(&self) -> f64 {
        if self.keystrokes > 0 {
            (self.errors as f64 / self.keystrokes as f64) * 100.0
        } else {
            0.0
        }
    }
}

pub fn draw_levels(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let filtered = app.filtered_sessions();

    if filtered.is_empty() {
        let para = Paragraph::new("No session records found matching the active filters.\nAdjust filters [t], [l], [m] to inspect historical performance.")
            .block(theme.block("Level & Mode Performance Breakdown"))
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);

    draw_level_table(frame, chunks[0], &filtered, theme);
    draw_mode_table(frame, chunks[1], &filtered, theme);
}

fn draw_level_table(frame: &mut Frame, area: Rect, sessions: &[&SessionSnapshot], theme: &Theme) {
    let mut beg = CategoryMetrics::default();
    let mut int = CategoryMetrics::default();
    let mut adv = CategoryMetrics::default();

    for s in sessions {
        match s.level.as_deref() {
            Some("Beginner") => beg.add(s),
            Some("Intermediate") => int.add(s),
            Some("Advanced") => adv.add(s),
            _ => {}
        }
    }

    let header = Row::new(vec![
        Cell::from("Difficulty Level"),
        Cell::from("Sessions"),
        Cell::from("Avg Speed"),
        Cell::from("Peak Speed"),
        Cell::from("Avg Accuracy"),
        Cell::from("Keystrokes"),
        Cell::from("Error Rate"),
    ])
    .style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let levels = [
        ("Beginner (Short Words)", &beg),
        ("Intermediate (Short Paragraphs)", &int),
        ("Advanced (Full Paragraphs)", &adv),
    ];

    let rows: Vec<Row> = levels
        .iter()
        .map(|(label, m)| {
            let acc_style = if m.avg_acc() >= 95.0 {
                Style::default().fg(theme.correct)
            } else if m.avg_acc() >= 90.0 {
                Style::default().fg(theme.warning)
            } else {
                Style::default().fg(theme.error)
            };

            Row::new(vec![
                Cell::from(*label).style(Style::default().fg(Color::White)),
                Cell::from(format!("{}", m.sessions)),
                Cell::from(format!("{:.1} WPM", m.avg_wpm())),
                Cell::from(format!("{:.1} WPM", m.peak_wpm)),
                Cell::from(format!("{:.1}%", m.avg_acc())).style(acc_style),
                Cell::from(format!("{}", m.keystrokes)),
                Cell::from(format!("{:.1}%", m.error_rate())),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(34),
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Length(16),
            Constraint::Length(14),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(theme.block("Performance Breakdown by Difficulty Level"));

    frame.render_widget(table, area);
}

fn draw_mode_table(frame: &mut Frame, area: Rect, sessions: &[&SessionSnapshot], theme: &Theme) {
    let mut targeted = CategoryMetrics::default();
    let mut random = CategoryMetrics::default();

    for s in sessions {
        match s.training_mode.as_deref() {
            Some("Targeted") => targeted.add(s),
            Some("Random") => random.add(s),
            _ => {}
        }
    }

    let header = Row::new(vec![
        Cell::from("Training Mode"),
        Cell::from("Sessions"),
        Cell::from("Avg Speed"),
        Cell::from("Peak Speed"),
        Cell::from("Avg Accuracy"),
        Cell::from("Keystrokes"),
        Cell::from("Error Rate"),
    ])
    .style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let modes = [
        ("Targeted (Focus Weakest Finger)", &targeted),
        ("Random (Standard Distribution)", &random),
    ];

    let rows: Vec<Row> = modes
        .iter()
        .map(|(label, m)| {
            let acc_style = if m.avg_acc() >= 95.0 {
                Style::default().fg(theme.correct)
            } else if m.avg_acc() >= 90.0 {
                Style::default().fg(theme.warning)
            } else {
                Style::default().fg(theme.error)
            };

            Row::new(vec![
                Cell::from(*label).style(Style::default().fg(Color::White)),
                Cell::from(format!("{}", m.sessions)),
                Cell::from(format!("{:.1} WPM", m.avg_wpm())),
                Cell::from(format!("{:.1} WPM", m.peak_wpm)),
                Cell::from(format!("{:.1}%", m.avg_acc())).style(acc_style),
                Cell::from(format!("{}", m.keystrokes)),
                Cell::from(format!("{:.1}%", m.error_rate())),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(34),
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Length(16),
            Constraint::Length(14),
            Constraint::Length(14),
        ],
    )
    .header(header)
    .block(theme.block("Performance Breakdown by Training Mode"));

    frame.render_widget(table, area);
}
