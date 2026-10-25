//! Cadence & Rhythm Metronome widget visualizing typing consistency and tempo.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::theme::Theme;

/// Cadence consistency state classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CadenceState {
    Ready,
    FlowState,
    Staccato,
    Hesitant,
}

impl CadenceState {
    pub fn from_consistency(pct: f64, num_typed: usize) -> Self {
        if num_typed < 3 {
            CadenceState::Ready
        } else if pct >= 90.0 {
            CadenceState::FlowState
        } else if pct >= 75.0 {
            CadenceState::Staccato
        } else {
            CadenceState::Hesitant
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            CadenceState::Ready => "READY (Establishing Rhythm)",
            CadenceState::FlowState => "FLOW STATE (Steady Rhythm)",
            CadenceState::Staccato => "STACCATO (Minor Hesitations)",
            CadenceState::Hesitant => "HESITANT (Uneven Cadence)",
        }
    }
}

/// Renders the real-time cadence consistency gauge.
pub fn draw_cadence_metronome(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    if area.height == 0 || area.width < 20 {
        return;
    }

    let num_typed = app.typed.len();
    let consistency = app.latency_engine.cadence.consistency_pct();
    let state = CadenceState::from_consistency(consistency, num_typed);

    let (state_color, state_label) = match state {
        CadenceState::Ready => (theme.muted, state.label()),
        CadenceState::FlowState => (theme.correct, state.label()),
        CadenceState::Staccato => (theme.warning, state.label()),
        CadenceState::Hesitant => (theme.error, state.label()),
    };

    let bar_width = 24usize;
    let (filled_count, has_marker, empty_count) = if state == CadenceState::Ready {
        (0, false, bar_width)
    } else {
        let k = ((consistency / 100.0) * bar_width as f64).round() as usize;
        let clamped = k.min(bar_width);
        if clamped == 0 {
            (0, false, bar_width)
        } else if clamped == bar_width {
            (bar_width - 1, true, 0)
        } else {
            (clamped - 1, true, bar_width - clamped)
        }
    };

    let filled_str: String = "█".repeat(filled_count);
    let marker_str = if has_marker { "◆" } else { "" };
    let empty_str: String = "░".repeat(empty_count);

    let pct_str = if state == CadenceState::Ready {
        "--%".to_string()
    } else {
        format!("{:.0}%", consistency)
    };

    let line = Line::from(vec![
        Span::styled(
            "CADENCE: ",
            Style::default().fg(theme.title).add_modifier(Modifier::BOLD),
        ),
        Span::styled("[", Style::default().fg(theme.border)),
        Span::styled(
            filled_str,
            Style::default().fg(state_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            marker_str,
            Style::default().fg(state_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(empty_str, Style::default().fg(theme.muted)),
        Span::styled("] ", Style::default().fg(theme.border)),
        Span::styled(
            format!("{pct_str} "),
            Style::default().fg(state_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(state_label, Style::default().fg(state_color)),
    ]);

    let para = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cadence_state_classification() {
        assert_eq!(CadenceState::from_consistency(98.0, 1), CadenceState::Ready);
        assert_eq!(CadenceState::from_consistency(92.0, 5), CadenceState::FlowState);
        assert_eq!(CadenceState::from_consistency(89.9, 10), CadenceState::Staccato);
        assert_eq!(CadenceState::from_consistency(75.0, 10), CadenceState::Staccato);
        assert_eq!(CadenceState::from_consistency(74.9, 10), CadenceState::Hesitant);
        assert_eq!(CadenceState::from_consistency(40.0, 15), CadenceState::Hesitant);
    }
}
