//! Post-round debrief and instant replay screen.
//!
//! Provides immediate biomechanical feedback after a practice round:
//! - Scorecard (Speed, Raw Speed, Accuracy, Consistency, Duration)
//! - Braille velocity timeline with mistake scatter markers
//! - Slowest bigrams with anatomical finger guidance
//! - Mistake substitution matrix
//! - Muscle-memory retry shortcuts ('r', 'w', Enter, Tab, Esc)

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::App;
use crate::theme::Theme;

pub fn draw_debrief(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let Some(summary) = &app.last_round_summary else {
        let msg = Paragraph::new("No session summary available. Press [Enter] for Setup.")
            .alignment(Alignment::Center)
            .block(theme.block("Debrief"));
        frame.render_widget(msg, area);
        return;
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Scorecard
            Constraint::Min(10),   // Velocity Timeline + Choke Points
            Constraint::Length(3), // Action Hotkeys
        ])
        .split(area);

    // 1. Scorecard Header
    let flow_tag = if summary.consistency_pct >= 90.0 {
        "Flow State"
    } else if summary.consistency_pct >= 75.0 {
        "Staccato"
    } else {
        "Hesitant"
    };

    let scorecard_text = format!(
        "SPEED: {:.1} WPM   RAW: {:.1} WPM   ACCURACY: {:.1}%   CONSISTENCY: {:.0}% ({flow_tag})   TIME: {:.1}s   KEYS: {} ({} errs)",
        summary.net_wpm,
        summary.raw_wpm,
        summary.accuracy_pct,
        summary.consistency_pct,
        summary.duration_secs,
        summary.total_keystrokes,
        summary.errors
    );
    let scorecard = Paragraph::new(scorecard_text)
        .alignment(Alignment::Center)
        .block(theme.block_focused("Round Completed! (Practice Debrief)"));
    frame.render_widget(scorecard, rows[0]);

    // 2. Main Middle Area: Chart on Left (55%), Choke points on Right (45%)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(rows[1]);

    // Left: Velocity Timeline Chart
    let max_t = summary.duration_secs.max(5.0);
    let peak_wpm = summary
        .velocity_timeline
        .iter()
        .map(|p| p.1)
        .fold(0.0f64, f64::max);
    let max_y = (peak_wpm * 1.15).max(80.0);

    let x_labels = vec![
        Span::raw("0s"),
        Span::raw(format!("{:.0}s", max_t / 2.0)),
        Span::raw(format!("{:.0}s", max_t)),
    ];
    let y_labels = vec![
        Span::raw("0"),
        Span::raw(format!("{:.0}", max_y / 2.0)),
        Span::raw(format!("{:.0}", max_y)),
    ];

    let datasets = vec![
        Dataset::default()
            .name("Velocity (WPM)")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.correct))
            .data(&summary.velocity_timeline),
        Dataset::default()
            .name("Mistakes (■)")
            .marker(symbols::Marker::Block)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(theme.error))
            .data(&summary.error_points),
    ];

    let chart = Chart::new(datasets)
        .block(theme.block("Keystroke Velocity Curve & Error Plot"))
        .x_axis(
            Axis::default()
                .title("Time (s)")
                .style(Style::default().fg(theme.muted))
                .bounds([0.0, max_t])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("WPM")
                .style(Style::default().fg(theme.muted))
                .bounds([0.0, max_y])
                .labels(y_labels),
        );
    frame.render_widget(chart, cols[0]);

    // Right: Slowest Bigrams & Mistake Substitutions
    let right_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(cols[1]);

    // Slowest Bigrams
    let mut bigram_lines: Vec<Line> = Vec::new();
    if summary.slowest_bigrams.is_empty() {
        bigram_lines.push(Line::from(Span::styled(
            "No hesitation transitions recorded (smooth cadence).",
            Style::default().fg(theme.muted),
        )));
    } else {
        for (i, ((c1, c2), avg_ms)) in summary.slowest_bigrams.iter().take(5).enumerate() {
            let f1 = app
                .engine
                .finger_for(*c1)
                .map(|f| f.to_string())
                .unwrap_or_else(|| "—".to_string());
            let f2 = app
                .engine
                .finger_for(*c2)
                .map(|f| f.to_string())
                .unwrap_or_else(|| "—".to_string());
            let text = format!("{}. '{}' ➔ '{}' ({} ➔ {}): {:.0}ms", i + 1, c1, c2, f1, f2, avg_ms);
            let style = if *avg_ms > 250.0 {
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD)
            } else if *avg_ms > 180.0 {
                Style::default().fg(theme.warning)
            } else {
                Style::default().fg(theme.brand)
            };
            bigram_lines.push(Line::from(Span::styled(text, style)));
        }
    }
    let bigram_para = Paragraph::new(bigram_lines).block(theme.block("Slowest Transitions & Hesitations"));
    frame.render_widget(bigram_para, right_rows[0]);

    // Mistake Substitutions
    let mut mistake_lines: Vec<Line> = Vec::new();
    if summary.mistake_counts.is_empty() {
        mistake_lines.push(Line::from(Span::styled(
            "100% Precision! Zero mistakes made.",
            Style::default().fg(theme.correct).add_modifier(Modifier::BOLD),
        )));
    } else {
        let mut spans: Vec<Span> = Vec::new();
        for ((exp, act), cnt) in summary.mistake_counts.iter().take(8) {
            let exp_str = if *exp == ' ' { "␣" } else { &exp.to_string() };
            let act_str = if *act == ' ' { "␣" } else { &act.to_string() };
            spans.push(Span::styled(
                format!(" '{exp_str}'➔'{act_str}' ({}x) ", cnt),
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
            ));
        }
        mistake_lines.push(Line::from(spans));
    }
    let mistake_para = Paragraph::new(mistake_lines).block(theme.block("Mistake Substitutions"));
    frame.render_widget(mistake_para, right_rows[1]);

    // 3. Bottom Action Bar
    let action_spans = vec![
        Span::styled("Actions: ", Style::default().fg(theme.title).add_modifier(Modifier::BOLD)),
        Span::styled("[r] ", Style::default().fg(theme.correct).add_modifier(Modifier::BOLD)),
        Span::styled("Retry same text   ", Style::default().fg(theme.untyped)),
        Span::styled("[w] ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
        Span::styled("Drill weak   ", Style::default().fg(theme.untyped)),
        Span::styled("[Enter] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Next round   ", Style::default().fg(theme.untyped)),
        Span::styled("[Tab] ", Style::default().fg(theme.brand).add_modifier(Modifier::BOLD)),
        Span::styled("Dashboard   ", Style::default().fg(theme.untyped)),
        Span::styled("[Esc] ", Style::default().fg(theme.muted).add_modifier(Modifier::BOLD)),
        Span::styled("Setup", Style::default().fg(theme.untyped)),
    ];
    let action_para = Paragraph::new(Line::from(action_spans))
        .alignment(Alignment::Center)
        .block(theme.block("Controls"));
    frame.render_widget(action_para, rows[2]);
}
