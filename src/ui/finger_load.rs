//! Tab 3: Biomechanical Finger Workload, Speed & Precision Balance (Milestones 4.2 & 4.4).

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::App;
use crate::finger::Finger;
use crate::theme::Theme;

pub fn draw_finger_load(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(14), Constraint::Length(4)])
        .split(area);

    draw_balance_table(frame, chunks[0], app, theme);
    draw_diagnostic_card(frame, chunks[1], app, theme);
}

fn progress_bar(ratio: f64, width: usize) -> String {
    let clamped = ratio.clamp(0.0, 1.0);
    let filled = (clamped * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

fn draw_balance_table(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let fingers = [
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

    let filtered = app.filtered_sessions();

    // Aggregate keystrokes and errors per finger
    let mut finger_ks: [u64; 10] = [0; 10];
    let mut finger_errs: [u64; 10] = [0; 10];

    if filtered.is_empty() {
        for (i, f) in fingers.iter().enumerate() {
            if let Some(stats) = app.engine.stats.get(f) {
                finger_ks[i] = stats.total_keystrokes;
                finger_errs[i] = stats.total_errors;
            }
        }
    } else {
        for s in &filtered {
            for (i, f) in fingers.iter().enumerate() {
                let key = f.as_key();
                finger_ks[i] += s.finger_keystrokes.get(key).copied().unwrap_or(0);
                finger_errs[i] += s.finger_errors.get(key).copied().unwrap_or(0);
            }
        }
    }

    let total_ks: u64 = finger_ks.iter().sum::<u64>().max(1);

    // Baseline latency estimation or from latency engine
    let base_iki = app.latency_engine.avg_iki_ms().max(120.0);
    let latency_weights = [1.35, 1.15, 1.0, 0.95, 0.85, 0.85, 0.95, 1.0, 1.2, 1.45];

    let header = Row::new(vec![
        Cell::from("Finger"),
        Cell::from("Workload Share (% Total Strokes)"),
        Cell::from("Latency Index (Est. IKI)"),
        Cell::from("Accuracy Precision (%)"),
    ])
    .style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = fingers
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let ks = finger_ks[i];
            let errs = finger_errs[i];
            let workload_pct = (ks as f64 / total_ks as f64) * 100.0;
            let workload_bar = progress_bar(workload_pct / 30.0, 14); // 30% max bar scale

            let latency_ms = base_iki * latency_weights[i];
            let latency_bar = progress_bar(latency_ms / 250.0, 12);

            let acc_pct = if ks > 0 {
                ((ks.saturating_sub(errs)) as f64 / ks as f64) * 100.0
            } else {
                100.0
            };
            let acc_bar = progress_bar(acc_pct / 100.0, 12);
            let flag = if ks > 0 && acc_pct < 90.0 { " !" } else { "" };

            let acc_style = if acc_pct >= 95.0 {
                Style::default().fg(theme.correct)
            } else if acc_pct >= 90.0 {
                Style::default().fg(theme.warning)
            } else {
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD)
            };

            Row::new(vec![
                Cell::from(f.to_string()).style(Style::default().fg(Color::White)),
                Cell::from(format!("{workload_bar} {workload_pct:>4.1}% ({ks})")),
                Cell::from(format!("{latency_bar} {latency_ms:>3.0}ms")),
                Cell::from(format!("{acc_bar} {acc_pct:>5.1}% ({errs} errs){flag}")).style(acc_style),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(32),
            Constraint::Length(24),
            Constraint::Length(28),
        ],
    )
    .header(header)
    .block(theme.block("Finger Biomechanical Workload, Speed & Precision Balance"));

    frame.render_widget(table, area);
}

fn draw_diagnostic_card(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let filtered = app.filtered_sessions();

    let fingers = [
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

    let mut finger_ks: [u64; 10] = [0; 10];
    let mut finger_errs: [u64; 10] = [0; 10];

    if filtered.is_empty() {
        for (i, f) in fingers.iter().enumerate() {
            if let Some(stats) = app.engine.stats.get(f) {
                finger_ks[i] = stats.total_keystrokes;
                finger_errs[i] = stats.total_errors;
            }
        }
    } else {
        for s in &filtered {
            for (i, f) in fingers.iter().enumerate() {
                let key = f.as_key();
                finger_ks[i] += s.finger_keystrokes.get(key).copied().unwrap_or(0);
                finger_errs[i] += s.finger_errors.get(key).copied().unwrap_or(0);
            }
        }
    }

    let left_ks: u64 = finger_ks[0..5].iter().sum();
    let right_ks: u64 = finger_ks[5..10].iter().sum();
    let total_ks = (left_ks + right_ks).max(1);

    let left_pct = (left_ks as f64 / total_ks as f64) * 100.0;
    let right_pct = (right_ks as f64 / total_ks as f64) * 100.0;

    let symmetry_eval = if (left_pct - right_pct).abs() < 10.0 {
        "Optimal Touch-Typing Balance"
    } else if left_pct > right_pct {
        "Left Hand Dominant"
    } else {
        "Right Hand Dominant"
    };

    // Find finger with lowest accuracy that has at least some keystrokes
    let mut worst_finger = Finger::RightPinky;
    let mut worst_acc = 100.0;
    let mut max_errs = 0;

    for (i, &f) in fingers.iter().enumerate() {
        let ks = finger_ks[i];
        let errs = finger_errs[i];
        if ks > 0 {
            let acc = ((ks.saturating_sub(errs)) as f64 / ks as f64) * 100.0;
            if acc < worst_acc || (acc == worst_acc && errs > max_errs) {
                worst_acc = acc;
                worst_finger = f;
                max_errs = errs;
            }
        }
    }

    let diag_text = format!(
        "Workload Symmetry: Left Hand: {:.1}% vs. Right Hand: {:.1}% ({symmetry_eval})\nBiomechanical Diagnostic: {} exhibits lowest accuracy ({:.1}%, {} errors). Recommendation: Target {} drills in Setup [t].",
        left_pct, right_pct, worst_finger, worst_acc, max_errs, worst_finger
    );

    let para = Paragraph::new(diag_text)
        .block(theme.block("Ergonomic Diagnostic & Touch Symmetry"))
        .wrap(Wrap { trim: true });

    frame.render_widget(para, area);
}
