//! Tab 5: Detailed scrollable session history table with TableState.

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::app::App;
use crate::theme::Theme;

/// Formats a UNIX timestamp (seconds since 1970-01-01) as `YYYY-MM-DD HH:MM` (UTC)
/// using Howard Hinnant's standard Gregorian civil algorithm without external crates.
pub fn format_unix_timestamp(ts: u64) -> String {
    let secs_of_day = ts % 86400;
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;

    let z = (ts / 86400) as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1024 + doe / 1460 - doe / 36524) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02} {hour:02}:{minute:02}")
}

pub fn draw_history_table(frame: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    let filtered = app.filtered_sessions();
    let total_sessions = filtered.len();

    if total_sessions == 0 {
        let para = Paragraph::new("No session records match the active filter criteria.\nAdjust filters [t], [l], [m] to view your past rounds.")
            .block(theme.block("Session History Table"))
            .wrap(Wrap { trim: true });
        frame.render_widget(para, area);
        return;
    }

    let selected_idx = app.history_table_state.selected().unwrap_or(0).min(total_sessions.saturating_sub(1));

    let header = Row::new(vec![
        Cell::from("#"),
        Cell::from("Date & Time"),
        Cell::from("Level & Mode"),
        Cell::from("Duration"),
        Cell::from("Speed"),
        Cell::from("Accuracy"),
        Cell::from("Strokes / Err"),
        Cell::from("Avg IKI"),
        Cell::from("Consistency"),
    ])
    .style(
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    // Display sessions newest first
    let rows: Vec<Row> = filtered
        .iter()
        .rev()
        .enumerate()
        .map(|(display_idx, s)| {
            let session_num = total_sessions.saturating_sub(display_idx);
            let date_str = format_unix_timestamp(s.timestamp);

            let lvl = s.level.as_deref().unwrap_or("—");
            let mode = s.training_mode.as_deref().unwrap_or("—");
            let level_mode = format!("{lvl} • {mode}");

            let dur_str = match s.session_duration_secs {
                Some(d) => format!("{d:.1}s"),
                None => "—".to_string(),
            };

            let speed_str = format!("{:.1} WPM", s.wpm);

            let acc = s.overall_accuracy.unwrap_or(0.0);
            let acc_str = format!("{acc:.1}%");
            let acc_style = if acc >= 95.0 {
                Style::default().fg(theme.correct)
            } else if acc >= 90.0 {
                Style::default().fg(theme.warning)
            } else {
                Style::default().fg(theme.error)
            };

            let ks = s.total_keystrokes.unwrap_or(0);
            let errs = s.error_count.unwrap_or(0);
            let strokes_str = format!("{ks} / {errs}");

            let iki_str = match s.avg_iki_ms {
                Some(iki) => format!("{iki:.0}ms"),
                None => "—".to_string(),
            };

            let rhythm_str = match s.consistency_pct {
                Some(c) => format!("{c:.0}%"),
                None => "—".to_string(),
            };

            Row::new(vec![
                Cell::from(format!("{session_num}")).style(Style::default().fg(theme.muted)),
                Cell::from(date_str).style(Style::default().fg(Color::White)),
                Cell::from(level_mode).style(Style::default().fg(Color::White)),
                Cell::from(dur_str).style(Style::default().fg(Color::White)),
                Cell::from(speed_str).style(Style::default().fg(theme.correct)),
                Cell::from(acc_str).style(acc_style),
                Cell::from(strokes_str).style(Style::default().fg(Color::White)),
                Cell::from(iki_str).style(Style::default().fg(theme.accent)),
                Cell::from(rhythm_str).style(Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let title = format!(
        "Session History Log (Row {} of {}) — Scroll: [↑/↓] Row  •  [PgUp/PgDn] Page  •  [Home/End] First/Last",
        selected_idx + 1,
        total_sessions
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(18),
            Constraint::Length(26),
            Constraint::Length(11),
            Constraint::Length(13),
            Constraint::Length(11),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(13),
        ],
    )
    .header(header)
    .block(theme.block(title))
    .row_highlight_style(
        Style::default()
            .bg(theme.accent)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("\u{25B6} ");

    frame.render_stateful_widget(table, area, &mut app.history_table_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_unix_timestamp() {
        // 2026-10-25 00:00:00 UTC
        // 1792886400 (approx)
        let ts_epoch = 0; // 1970-01-01 00:00
        assert_eq!(format_unix_timestamp(ts_epoch), "1970-01-01 00:00");

        let ts_one_day = 86400 + 3600 * 14 + 60 * 30; // 1970-01-02 14:30
        assert_eq!(format_unix_timestamp(ts_one_day), "1970-01-02 14:30");
    }
}
