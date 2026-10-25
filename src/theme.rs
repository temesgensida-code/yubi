//! Centralized visual theme engine.
//!
//! Provides multiple curated palettes (Yubi Classic, Catppuccin Mocha, Tokyo Night,
//! Nord, Gruvbox Dark) with full dynamic switching at runtime.

use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeId {
    #[default]
    YubiClassic,
    CatppuccinMocha,
    TokyoNight,
    Nord,
    GruvboxDark,
}

impl ThemeId {
    pub const ALL: [ThemeId; 5] = [
        ThemeId::YubiClassic,
        ThemeId::CatppuccinMocha,
        ThemeId::TokyoNight,
        ThemeId::Nord,
        ThemeId::GruvboxDark,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            ThemeId::YubiClassic => "Yubi Classic (Charcoal & Cyan)",
            ThemeId::CatppuccinMocha => "Catppuccin Mocha",
            ThemeId::TokyoNight => "Tokyo Night",
            ThemeId::Nord => "Nord",
            ThemeId::GruvboxDark => "Gruvbox Dark",
        }
    }

    pub fn next(&self) -> ThemeId {
        let idx = ThemeId::ALL.iter().position(|t| t == self).unwrap_or(0);
        ThemeId::ALL[(idx + 1) % ThemeId::ALL.len()]
    }

    #[allow(dead_code)]
    pub fn prev(&self) -> ThemeId {
        let idx = ThemeId::ALL.iter().position(|t| t == self).unwrap_or(0);
        ThemeId::ALL[(idx + ThemeId::ALL.len() - 1) % ThemeId::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy)]
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
    pub brand: Color,
}

impl Theme {
    pub fn block<'a>(&self, title: impl Into<String>) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(self.border))
            .title(title.into())
            .title_style(Style::default().fg(self.title))
    }

    pub fn block_focused<'a>(&self, title: impl Into<String>) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(self.border_focused))
            .title(title.into())
            .title_style(Style::default().fg(self.title))
    }
}

pub fn get_theme(id: ThemeId) -> Theme {
    match id {
        ThemeId::YubiClassic => Theme {
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
            brand: Color::Rgb(48, 57, 53),
        },
        ThemeId::CatppuccinMocha => Theme {
            border: Color::Rgb(88, 91, 112),         // Surface2
            border_focused: Color::Rgb(203, 166, 247), // Mauve
            title: Color::Rgb(203, 166, 247),
            correct: Color::Rgb(166, 227, 161),       // Green
            error: Color::Rgb(243, 139, 168),         // Red
            error_flash_bg: Color::Rgb(130, 30, 50),
            cursor_bg: Color::Rgb(249, 226, 175),     // Peach
            cursor_fg: Color::Rgb(17, 17, 27),        // Crust
            untyped: Color::Rgb(108, 112, 134),       // Overlay0
            untyped_current_word: Color::Rgb(205, 214, 244), // Text
            accent: Color::Rgb(137, 180, 250),        // Blue
            warning: Color::Rgb(250, 179, 135),       // Peach
            muted: Color::Rgb(108, 112, 134),
            brand: Color::Rgb(49, 50, 68),            // Surface0
        },
        ThemeId::TokyoNight => Theme {
            border: Color::Rgb(65, 72, 104),          // Dark border
            border_focused: Color::Rgb(122, 162, 247), // Soft Blue
            title: Color::Rgb(122, 162, 247),
            correct: Color::Rgb(158, 206, 106),       // Green
            error: Color::Rgb(247, 118, 142),         // Red
            error_flash_bg: Color::Rgb(140, 30, 45),
            cursor_bg: Color::Rgb(224, 175, 104),     // Yellow
            cursor_fg: Color::Rgb(26, 27, 38),
            untyped: Color::Rgb(86, 95, 137),
            untyped_current_word: Color::Rgb(192, 202, 245),
            accent: Color::Rgb(125, 207, 255),        // Cyan
            warning: Color::Rgb(255, 158, 100),       // Orange
            muted: Color::Rgb(86, 95, 137),
            brand: Color::Rgb(36, 40, 59),
        },
        ThemeId::Nord => Theme {
            border: Color::Rgb(76, 86, 106),          // Polar Night 3
            border_focused: Color::Rgb(136, 192, 208), // Frost Blue
            title: Color::Rgb(136, 192, 208),
            correct: Color::Rgb(163, 190, 140),       // Aurora Green
            error: Color::Rgb(191, 97, 106),          // Aurora Red
            error_flash_bg: Color::Rgb(120, 35, 45),
            cursor_bg: Color::Rgb(235, 203, 139),     // Aurora Yellow
            cursor_fg: Color::Rgb(46, 52, 64),
            untyped: Color::Rgb(94, 129, 172),
            untyped_current_word: Color::Rgb(236, 239, 244), // Snow Storm
            accent: Color::Rgb(136, 192, 208),
            warning: Color::Rgb(208, 135, 112),       // Aurora Orange
            muted: Color::Rgb(76, 86, 106),
            brand: Color::Rgb(59, 66, 82),
        },
        ThemeId::GruvboxDark => Theme {
            border: Color::Rgb(102, 92, 84),          // Gray 246
            border_focused: Color::Rgb(250, 189, 47),  // Bright Yellow
            title: Color::Rgb(250, 189, 47),
            correct: Color::Rgb(184, 187, 38),        // Bright Green
            error: Color::Rgb(251, 73, 52),           // Bright Red
            error_flash_bg: Color::Rgb(140, 25, 25),
            cursor_bg: Color::Rgb(254, 128, 25),      // Bright Orange
            cursor_fg: Color::Rgb(40, 40, 40),
            untyped: Color::Rgb(146, 131, 116),
            untyped_current_word: Color::Rgb(235, 219, 178), // Light1
            accent: Color::Rgb(104, 157, 106),        // Aqua
            warning: Color::Rgb(214, 93, 14),         // Orange
            muted: Color::Rgb(124, 111, 100),
            brand: Color::Rgb(60, 56, 54),
        },
    }
}

#[allow(dead_code)]
pub fn block(title: impl Into<String>) -> Block<'static> {
    get_theme(ThemeId::default()).block(title)
}

#[allow(dead_code)]
pub fn block_focused(title: impl Into<String>) -> Block<'static> {
    get_theme(ThemeId::default()).block_focused(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_cycle() {
        let mut current = ThemeId::YubiClassic;
        current = current.next();
        assert_eq!(current, ThemeId::CatppuccinMocha);
        current = current.next();
        assert_eq!(current, ThemeId::TokyoNight);
        current = current.next();
        assert_eq!(current, ThemeId::Nord);
        current = current.next();
        assert_eq!(current, ThemeId::GruvboxDark);
        current = current.next();
        assert_eq!(current, ThemeId::YubiClassic);
    }

    #[test]
    fn test_all_themes_have_valid_palette() {
        for id in ThemeId::ALL {
            let t = get_theme(id);
            assert!(!id.label().is_empty());
            let _b = t.block("Test");
            let _bf = t.block_focused("Test Focused");
        }
    }
}
