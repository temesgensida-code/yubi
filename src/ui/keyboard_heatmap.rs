//! Tab 2: Interactive Virtual Keyboard Heatmap & Key Inspector.

use std::collections::HashMap;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

use crate::app::{App, HeatmapMode, HEATMAP_KEYS};
use crate::theme::Theme;

#[derive(Default, Clone)]
pub struct KeyAggregate {
    pub keystrokes: u64,
    pub errors: u64,
    pub substitutions: HashMap<char, u64>,
}

pub fn aggregate_key_stats(app: &App) -> HashMap<char, KeyAggregate> {
    let mut stats: HashMap<char, KeyAggregate> = HashMap::new();

    // 1. From engine stats in memory
    for fs in app.engine.stats.values() {
        for (&expected, wrongs) in &fs.mistake_matrix {
            let exp_lower = expected.to_ascii_lowercase();
            let mut key_errors = 0;
            for (&actual, &count) in wrongs {
                let act_lower = actual.to_ascii_lowercase();
                key_errors += count;
                *stats
                    .entry(exp_lower)
                    .or_default()
                    .substitutions
                    .entry(act_lower)
                    .or_default() += count;
            }
            let entry = stats.entry(exp_lower).or_default();
            entry.errors += key_errors;
            entry.keystrokes += key_errors;
        }
    }

    // 2. From filtered sessions top_mistakes serialized strings
    for s in app.filtered_sessions() {
        if let Some(ref tm) = s.top_mistakes {
            // format: "e->r:28;e->w:16;..."
            for item in tm.split(';') {
                let parts: Vec<&str> = item.split("->").collect();
                if parts.len() == 2 {
                    let exp_char = parts[0].chars().next().unwrap_or(' ').to_ascii_lowercase();
                    let act_parts: Vec<&str> = parts[1].split(':').collect();
                    if act_parts.len() == 2 {
                        let act_char = act_parts[0].chars().next().unwrap_or(' ').to_ascii_lowercase();
                        let count: u64 = act_parts[1].parse().unwrap_or(0);
                        *stats
                            .entry(exp_char)
                            .or_default()
                            .substitutions
                            .entry(act_char)
                            .or_default() += count;
                    }
                }
            }
        }
    }

    stats
}

pub fn draw_keyboard_heatmap(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let stats = aggregate_key_stats(app);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(48), Constraint::Length(36)])
        .split(area);

    draw_visual_keyboard(frame, chunks[0], app, &stats, theme);
    draw_key_inspector(frame, chunks[1], app, &stats, theme);
}

fn draw_visual_keyboard(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    stats: &HashMap<char, KeyAggregate>,
    theme: &Theme,
) {
    let max_errors = stats.values().map(|s| s.errors).max().unwrap_or(1).max(1);
    let max_keystrokes = stats
        .values()
        .map(|s| s.keystrokes)
        .max()
        .unwrap_or(1)
        .max(1);

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    let row_indents = [0, 2, 4, 6, 12];

    for (r_idx, &row) in HEATMAP_KEYS.iter().enumerate() {
        let mut spans = Vec::new();
        let indent = " ".repeat(row_indents[r_idx]);
        spans.push(Span::raw(indent));

        for &c in row {
            let is_selected = c == app.selected_heatmap_key;
            let agg = stats.get(&c.to_ascii_lowercase()).cloned().unwrap_or_default();

            let (style, cell_text) = if c == ' ' {
                let style = get_key_style(app.heatmap_mode, &agg, max_errors, max_keystrokes, is_selected, theme);
                let label = if is_selected {
                    "[        SPACE        ]"
                } else {
                    "         SPACE         "
                };
                (style, label.to_string())
            } else {
                let style = get_key_style(app.heatmap_mode, &agg, max_errors, max_keystrokes, is_selected, theme);
                let label = if is_selected {
                    format!("[{}]", c.to_ascii_uppercase())
                } else {
                    format!(" {} ", c.to_ascii_uppercase())
                };
                (style, label)
            };

            spans.push(Span::styled(cell_text, style));
            spans.push(Span::raw(" "));
        }

        lines.push(Line::from(spans));
        lines.push(Line::from(""));
    }

    // Legend
    lines.push(Line::from(""));
    let legend_spans = match app.heatmap_mode {
        HeatmapMode::FingerAccuracy => vec![
            Span::raw("Legend: "),
            Span::styled("■ <90% (Poor)  ", Style::default().fg(theme.error)),
            Span::styled("■ 90-95% (Warning)  ", Style::default().fg(theme.warning)),
            Span::styled("■ ≥95% (Good)  ", Style::default().fg(theme.correct)),
            Span::styled("■ No Data", Style::default().fg(theme.muted)),
        ],
        HeatmapMode::KeyErrors => vec![
            Span::raw("Legend: "),
            Span::styled("■ Low Errors  ", Style::default().fg(theme.warning)),
            Span::styled("■ Moderate  ", Style::default().fg(Color::Rgb(255, 140, 0))),
            Span::styled("■ High Errors  ", Style::default().fg(theme.error)),
            Span::styled("■ Clean", Style::default().fg(theme.muted)),
        ],
        HeatmapMode::KeystrokeLoad => vec![
            Span::raw("Legend: "),
            Span::styled("■ Light Load  ", Style::default().fg(theme.muted)),
            Span::styled("■ Moderate  ", Style::default().fg(theme.accent)),
            Span::styled("■ Heavy Load  ", Style::default().fg(Color::Cyan)),
        ],
    };
    lines.push(Line::from(legend_spans));

    let controls = format!(
        "Controls: [↑↓←→] Select Key  •  [h] Heatmap Mode: {}  •  [r] Setup",
        app.heatmap_mode.label()
    );
    lines.push(Line::from(Span::styled(controls, Style::default().fg(theme.brand))));

    let title = format!("Keyboard Heatmap — Mode: {}", app.heatmap_mode.label());
    let para = Paragraph::new(lines).block(theme.block(title));
    frame.render_widget(para, area);
}

fn get_key_style(
    mode: HeatmapMode,
    agg: &KeyAggregate,
    max_errors: u64,
    max_keystrokes: u64,
    is_selected: bool,
    theme: &Theme,
) -> Style {
    if is_selected {
        return Style::default()
            .bg(theme.accent)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD);
    }

    match mode {
        HeatmapMode::FingerAccuracy => {
            if agg.keystrokes == 0 && agg.errors == 0 {
                Style::default().fg(theme.muted)
            } else {
                let acc = if agg.keystrokes > 0 {
                    ((agg.keystrokes.saturating_sub(agg.errors)) as f64 / agg.keystrokes as f64) * 100.0
                } else {
                    100.0 - (agg.errors as f64 * 5.0).min(50.0)
                };
                if acc >= 95.0 {
                    Style::default().fg(theme.correct)
                } else if acc >= 90.0 {
                    Style::default().fg(theme.warning)
                } else {
                    Style::default().fg(theme.error).add_modifier(Modifier::BOLD)
                }
            }
        }
        HeatmapMode::KeyErrors => {
            if agg.errors == 0 {
                Style::default().fg(theme.muted)
            } else {
                let ratio = agg.errors as f64 / max_errors as f64;
                if ratio < 0.33 {
                    Style::default().fg(theme.warning)
                } else if ratio < 0.66 {
                    Style::default().fg(Color::Rgb(255, 140, 0))
                } else {
                    Style::default().fg(theme.error).add_modifier(Modifier::BOLD)
                }
            }
        }
        HeatmapMode::KeystrokeLoad => {
            if agg.keystrokes == 0 {
                Style::default().fg(theme.muted)
            } else {
                let ratio = agg.keystrokes as f64 / max_keystrokes as f64;
                if ratio < 0.33 {
                    Style::default().fg(theme.muted)
                } else if ratio < 0.66 {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                }
            }
        }
    }
}

fn draw_key_inspector(
    frame: &mut Frame,
    area: Rect,
    app: &App,
    stats: &HashMap<char, KeyAggregate>,
    theme: &Theme,
) {
    let key = app.selected_heatmap_key;
    let key_label = if key == ' ' { "SPACE".to_string() } else { key.to_string().to_uppercase() };
    let agg = stats.get(&key.to_ascii_lowercase()).cloned().unwrap_or_default();

    let finger = app.engine.keymap.get(&key.to_ascii_lowercase()).copied();
    let finger_name = finger
        .map(|f| f.to_string())
        .unwrap_or_else(|| "Unassigned".to_string());

    let acc_pct = if agg.keystrokes > 0 {
        ((agg.keystrokes.saturating_sub(agg.errors)) as f64 / agg.keystrokes as f64) * 100.0
    } else if agg.errors > 0 {
        0.0
    } else {
        100.0
    };

    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::raw("Assigned Finger: "),
        Span::styled(finger_name, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("Key Accuracy:   "),
        Span::styled(
            format!("{acc_pct:.1}%"),
            Style::default().fg(if acc_pct >= 95.0 {
                theme.correct
            } else if acc_pct >= 90.0 {
                theme.warning
            } else {
                theme.error
            }),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Total Strokes:  "),
        Span::styled(format!("{}", agg.keystrokes), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("Total Errors:   "),
        Span::styled(
            format!("{}", agg.errors),
            Style::default().fg(if agg.errors > 0 { theme.error } else { theme.correct }),
        ),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Top Mistake Substitutions:",
        Style::default().fg(theme.brand).add_modifier(Modifier::BOLD),
    )));

    let mut subs: Vec<(char, u64)> = agg.substitutions.into_iter().collect();
    subs.sort_by_key(|a| std::cmp::Reverse(a.1));

    if subs.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  No mistakes recorded on this key!",
            Style::default().fg(theme.correct),
        )));
    } else {
        let total_subs: u64 = subs.iter().map(|s| s.1).sum();
        for (idx, (actual, count)) in subs.iter().take(5).enumerate() {
            let pct = if total_subs > 0 {
                (*count as f64 / total_subs as f64) * 100.0
            } else {
                0.0
            };
            let act_display = if *actual == ' ' { "SPACE".to_string() } else { actual.to_string().to_uppercase() };
            lines.push(Line::from(vec![
                Span::raw(format!("  {}. Typed '", idx + 1)),
                Span::styled(act_display, Style::default().fg(theme.error)),
                Span::raw(format!("' instead: {count}x ({pct:.1}%)")),
            ]));
        }
    }

    let title = format!("Key Inspector: '{key_label}'");
    let para = Paragraph::new(lines)
        .block(theme.block(title))
        .wrap(Wrap { trim: true });
    frame.render_widget(para, area);
}
