//! Tab 1: Continuous WPM and Accuracy Trend visualization with Braille curves.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    symbols,
    text::Span,
    widgets::{Axis, Chart, Dataset, GraphType, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::theme::Theme;

/// Generates a dense, smooth continuous curve from discrete `(x, y)` points.
/// Uses smooth cosine sub-sampling so Ratatui's Braille canvas displays a seamless, connected curve.
pub fn generate_smooth_curve(points: &[(f64, f64)], num_samples: usize) -> Vec<(f64, f64)> {
    if points.is_empty() {
        return Vec::new();
    }
    if points.len() == 1 {
        return vec![points[0]];
    }

    let min_x = points[0].0;
    let max_x = points[points.len() - 1].0;
    let total_span = max_x - min_x;

    if total_span <= f64::EPSILON {
        return points.to_vec();
    }

    let n = num_samples.max(points.len() * 4).min(200);
    let mut curve = Vec::with_capacity(n);

    for i in 0..n {
        let t = i as f64 / (n - 1) as f64;
        let target_x = min_x + t * total_span;

        let mut idx = 0;
        while idx < points.len() - 1 && points[idx + 1].0 < target_x {
            idx += 1;
        }

        if idx >= points.len() - 1 {
            curve.push((target_x, points[points.len() - 1].1));
            continue;
        }

        let p0 = points[idx];
        let p1 = points[idx + 1];
        let dx = p1.0 - p0.0;

        let interpolated_y = if dx <= f64::EPSILON {
            p0.1
        } else {
            let local_t = (target_x - p0.0) / dx;
            // Cosine interpolation for smooth S-curve transitions
            let mu2 = (1.0 - (local_t * std::f64::consts::PI).cos()) / 2.0;
            p0.1 * (1.0 - mu2) + p1.1 * mu2
        };

        curve.push((target_x, interpolated_y));
    }

    curve
}

pub fn draw_trends(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let filtered = app.filtered_sessions();

    if filtered.len() < 2 {
        let msg = if filtered.is_empty() {
            "No sessions match the current filter criteria.\nAdjust filters with [t] (Time), [l] (Level), [m] (Mode)."
        } else {
            "Need at least 2 sessions under current filters to draw progression curves.\nComplete another round or broaden active filters."
        };
        let para = Paragraph::new(msg)
            .block(theme.block("Speed (WPM) & Accuracy Trend Over Time"))
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
        return;
    }

    let wpm_points: Vec<(f64, f64)> = filtered
        .iter()
        .enumerate()
        .map(|(i, s)| (i as f64, s.wpm))
        .collect();

    let acc_points: Vec<(f64, f64)> = filtered
        .iter()
        .enumerate()
        .map(|(i, s)| (i as f64, s.overall_accuracy.unwrap_or(0.0)))
        .collect();

    let max_x = (filtered.len() as f64 - 1.0).max(1.0);
    let peak_wpm = wpm_points.iter().map(|(_, y)| *y).fold(0.0_f64, f64::max);
    let avg_wpm = wpm_points.iter().map(|(_, y)| *y).sum::<f64>() / wpm_points.len() as f64;
    let latest_wpm = wpm_points.last().map(|(_, y)| *y).unwrap_or(0.0);
    let avg_acc = acc_points.iter().map(|(_, y)| *y).sum::<f64>() / acc_points.len() as f64;
    let latest_acc = acc_points.last().map(|(_, y)| *y).unwrap_or(0.0);

    let smooth_wpm = generate_smooth_curve(&wpm_points, 100);
    let smooth_acc = generate_smooth_curve(&acc_points, 100);
    let target_90_data = vec![(0.0, 90.0), (max_x, 90.0)];

    let max_y = (peak_wpm.max(100.0) * 1.1).max(50.0);

    let target_line = Dataset::default()
        .name("Target (90%)")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.muted))
        .data(&target_90_data);

    let wpm_line = Dataset::default()
        .name("Smooth WPM")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.correct))
        .data(&smooth_wpm);

    let acc_line = Dataset::default()
        .name("Accuracy %")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.accent))
        .data(&smooth_acc);

    let wpm_dots = Dataset::default()
        .name("WPM Points")
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Scatter)
        .style(Style::default().fg(theme.brand))
        .data(&wpm_points);

    let acc_dots = Dataset::default()
        .name("Acc Points")
        .marker(symbols::Marker::Dot)
        .graph_type(GraphType::Scatter)
        .style(Style::default().fg(theme.warning))
        .data(&acc_points);

    let x_labels = vec![
        Span::raw("Session #1"),
        Span::raw(format!("Session #{}", (filtered.len() / 2).max(1))),
        Span::raw(format!("Session #{}", filtered.len())),
    ];

    let y_labels = vec![
        Span::raw("0"),
        Span::raw(format!("{:.0}", max_y * 0.33)),
        Span::raw(format!("{:.0}", max_y * 0.66)),
        Span::raw(format!("{max_y:.0}")),
    ];

    let title = format!(
        "Speed (WPM) & Accuracy Trend  •  Avg: {avg_wpm:.1} WPM ({avg_acc:.1}%)  •  Peak: {peak_wpm:.1} WPM  •  Latest: {latest_wpm:.1} WPM ({latest_acc:.1}%)"
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if area.height >= 14 {
            [Constraint::Min(9), Constraint::Length(3)]
        } else {
            [Constraint::Min(0), Constraint::Length(0)]
        })
        .split(area);

    let chart = Chart::new(vec![target_line, wpm_line, acc_line, wpm_dots, acc_dots])
        .block(theme.block(title))
        .x_axis(
            Axis::default()
                .title("Session Sequence")
                .style(Style::default().fg(theme.muted))
                .bounds([0.0, max_x])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("WPM / Acc %")
                .style(Style::default().fg(theme.muted))
                .bounds([0.0, max_y])
                .labels(y_labels),
        );

    frame.render_widget(chart, chunks[0]);

    if chunks.len() > 1 && chunks[1].height >= 3 {
        let first_wpm = wpm_points.first().map(|(_, y)| *y).unwrap_or(0.0);
        let delta_wpm = latest_wpm - first_wpm;
        let delta_sign = if delta_wpm >= 0.0 { "+" } else { "" };
        let met_90_count = acc_points.iter().filter(|(_, y)| *y >= 90.0).count();
        let met_90_pct = (met_90_count as f64 / acc_points.len() as f64) * 100.0;

        let summary_text = format!(
            "Progression:  WPM Growth: {delta_sign}{delta_wpm:.1} WPM  •  Avg Speed: {avg_wpm:.1} WPM  •  Sessions \u{2265}90% Target: {met_90_count}/{} ({met_90_pct:.0}%)  •  Legend: \u{2500}\u{2500} WPM (Green)   \u{2500}\u{2500} Acc % (Cyan)   - - Target 90% (Gray)",
            acc_points.len()
        );
        let summary_para = Paragraph::new(summary_text)
            .alignment(Alignment::Center)
            .block(theme.block("Trend Summary"));
        frame.render_widget(summary_para, chunks[1]);
    }
}
