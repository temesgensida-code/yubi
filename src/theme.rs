//! Centralized visual theme.
//!
//! Every color used anywhere in `ui.rs` is defined once here. Restyling the
//! whole app (dark/light theme, different accent color, etc.) means editing
//! this file only, instead of hunting for scattered `Color::X` literals.

use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders};

pub struct Theme {
    pub border: Color,
    pub border_focused: Color,
    pub title: Color,
    pub correct: Color,
    pub error: Color,
    pub error_flash_bg: Color,
    pub cursor_bg: Color,
    pub cursor_fg: Color,
    pub untyped: Color,
    pub untyped_current_word: Color,
    pub accent: Color,
    pub warning: Color,
    pub muted: Color,
}

pub const THEME: Theme = Theme {
    border: Color::DarkGray,
    border_focused: Color::Cyan,
    title: Color::Cyan,
    correct: Color::Green,
    error: Color::Red,
    error_flash_bg: Color::Rgb(120, 20, 20),
    cursor_bg: Color::Yellow,
    cursor_fg: Color::Black,
    untyped: Color::DarkGray,
    untyped_current_word: Color::Gray,
    accent: Color::Cyan,
    warning: Color::LightRed,
    muted: Color::DarkGray,
};

/// A `Block` with rounded borders and a title, styled from [`THEME`].
/// The single helper every panel in `ui.rs` should go through, so border
/// style/type stays consistent everywhere.
pub fn block(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.border))
        .title(title.into())
}

/// Same as [`block`] but with an accent-colored border, for panels that
/// should visually stand out (e.g. the focused field on the setup screen).
pub fn block_focused(title: impl Into<String>) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(THEME.border_focused))
        .title(title.into())
}
