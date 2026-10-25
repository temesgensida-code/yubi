//! FingerTrack — a TUI typing trainer that tracks per-finger accuracy over
//! time and visualizes it with terminal charts.
//!
//! Architecture:
//! - `finger`      : the Finger enum, QWERTY -> finger keymap, TypingEngine
//! - `generator`   : weak-finger-targeted word drills (used at Beginner level)
//! - `paragraphs`  : finger-tagged paragraph bank (Intermediate/Advanced levels)
//! - `persistence` : JSON load/save of cross-session history + app settings
//! - `app`         : application state machine (Consent / Setup / Typing / Dashboard)
//! - `theme`       : centralized color palette + shared widget helpers
//! - `ui`          : pure rendering functions (ratatui widgets)
//!
//! `main.rs` only wires these together: terminal setup/teardown and the
//! non-blocking input -> update -> draw loop.

mod app;
mod drills;
mod engine;
mod finger;
mod generator;
mod paragraphs;
mod persistence;
mod theme;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, DashboardTab, Mode};

/// How long each poll waits for an input event before giving up control and
/// looping back to redraw. Keeps the UI responsive (redraws ~10x/sec even
/// with no input - which also drives the live WPM sparkline and the error
/// flash fade) without busy-spinning the CPU.
const POLL_TIMEOUT: Duration = Duration::from_millis(100);

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let app = App::new();
    let result = run_app(&mut terminal, app);
    restore_terminal(&mut terminal)?;

    if let Err(err) = &result {
        eprintln!("Error: {err}");
    }
    result
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Mouse capture is on so the Dashboard's finger sidebar can support
    // click-to-select (see `handle_mouse` / `App::handle_sidebar_click`).
    // This does mean the terminal swallows normal text selection while
    // FingerTrack is running; that trade-off only matters in the
    // Dashboard, since Setup/Typing simply ignore mouse events.
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// The core loop: draw the current state, tick time-based effects, then
/// non-blockingly poll for an input event and dispatch it. Because
/// `event::poll` returns promptly even when nothing happened, the terminal
/// stays responsive to resizes and we never block indefinitely on stdin.
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    loop {
        app.tick();
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        if event::poll(POLL_TIMEOUT)? {
            match event::read()? {
                Event::Key(key) => {
                    // On some platforms (notably Windows) key events fire
                    // on both press and release; only act on press to
                    // avoid double-handling.
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    handle_key(&mut app, key);
                }
                Event::Mouse(mouse) => handle_mouse(&mut app, mouse),
                Event::Resize(_, _) => {
                    // Nothing to do explicitly: the next `terminal.draw`
                    // call already re-queries the terminal size via
                    // `frame.area()`, so layouts adapt automatically.
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

/// Routes mouse events. Currently the only interactive mouse target is the
/// Dashboard's finger sidebar (left-click a row to select that finger);
/// every other mode ignores mouse input entirely.
fn handle_mouse(app: &mut App, mouse: MouseEvent) {
    if app.mode != Mode::Dashboard {
        return;
    }
    if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
        app.handle_sidebar_click(mouse.column, mouse.row);
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::F(2) {
        app.cycle_theme();
        return;
    }
    match app.mode {
        Mode::Consent => match key.code {
            KeyCode::Esc => app.should_quit = true,
            KeyCode::Char('y') | KeyCode::Char('Y') => app.set_tracking_consent(true),
            KeyCode::Char('n') | KeyCode::Char('N') => app.set_tracking_consent(false),
            _ => {}
        },
        Mode::Setup => match key.code {
            KeyCode::Esc => app.should_quit = true,
            KeyCode::Up | KeyCode::Down | KeyCode::Tab => app.setup_toggle_focus(),
            KeyCode::Left => app.setup_cycle_left(),
            KeyCode::Right => app.setup_cycle_right(),
            KeyCode::Enter => app.start_new_round(),
            _ => {}
        },
        Mode::Typing => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match key.code {
                    KeyCode::Char('w') | KeyCode::Backspace => {
                        app.handle_delete_word();
                        return;
                    }
                    KeyCode::Char('r') => {
                        app.restart_current_round();
                        return;
                    }
                    _ => {}
                }
            }
            match key.code {
                KeyCode::Esc => app.should_quit = true,
                KeyCode::Backspace => app.handle_backspace(),
                KeyCode::Char(c) => {
                    app.handle_char_input(c);
                }
                _ => {}
            }
        }
        Mode::Debrief => match key.code {
            KeyCode::Char('r') => {
                app.restart_current_round();
                app.mode = Mode::Typing;
            }
            KeyCode::Char('w') => {
                app.training_mode = crate::app::TrainingMode::FocusWeakest;
                app.start_new_round();
            }
            KeyCode::Enter => {
                app.start_new_round();
            }
            KeyCode::Tab => {
                app.mode = Mode::Dashboard;
            }
            KeyCode::Esc => {
                app.mode = Mode::Setup;
            }
            KeyCode::Char('q') => app.should_quit = true,
            _ => {}
        },
        Mode::Dashboard => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('1') => app.set_dashboard_tab(DashboardTab::Trends),
            KeyCode::Char('2') => app.set_dashboard_tab(DashboardTab::KeyboardHeatmap),
            KeyCode::Char('3') => app.set_dashboard_tab(DashboardTab::FingerLoad),
            KeyCode::Char('4') => app.set_dashboard_tab(DashboardTab::LevelComparison),
            KeyCode::Char('5') => app.set_dashboard_tab(DashboardTab::SessionHistory),
            KeyCode::Tab => app.cycle_dashboard_tab(),
            KeyCode::BackTab => app.prev_dashboard_tab(),
            KeyCode::Char('t') | KeyCode::Char('T') => app.cycle_time_filter(),
            KeyCode::Char('l') | KeyCode::Char('L') => app.cycle_level_filter(),
            KeyCode::Char('m') | KeyCode::Char('M') => {
                if app.dashboard_tab == DashboardTab::KeyboardHeatmap {
                    app.cycle_heatmap_mode();
                } else {
                    app.cycle_mode_filter();
                }
            }
            KeyCode::Char('h') | KeyCode::Char('H') => app.cycle_heatmap_mode(),
            KeyCode::Char('f') | KeyCode::Char('F') => app.cycle_time_filter(),
            KeyCode::Up => match app.dashboard_tab {
                DashboardTab::KeyboardHeatmap => app.move_heatmap_selection(-1, 0),
                DashboardTab::SessionHistory => {
                    let count = app.filtered_sessions().len();
                    app.history_prev(count);
                }
                _ => app.select_prev_finger(),
            },
            KeyCode::Down => match app.dashboard_tab {
                DashboardTab::KeyboardHeatmap => app.move_heatmap_selection(1, 0),
                DashboardTab::SessionHistory => {
                    let count = app.filtered_sessions().len();
                    app.history_next(count);
                }
                _ => app.select_next_finger(),
            },
            KeyCode::Left => {
                if app.dashboard_tab == DashboardTab::KeyboardHeatmap {
                    app.move_heatmap_selection(0, -1);
                }
            }
            KeyCode::Right => {
                if app.dashboard_tab == DashboardTab::KeyboardHeatmap {
                    app.move_heatmap_selection(0, 1);
                }
            }
            KeyCode::PageUp => {
                if app.dashboard_tab == DashboardTab::SessionHistory {
                    app.history_page_up(10);
                }
            }
            KeyCode::PageDown => {
                if app.dashboard_tab == DashboardTab::SessionHistory {
                    let count = app.filtered_sessions().len();
                    app.history_page_down(count, 10);
                }
            }
            KeyCode::Home => {
                if app.dashboard_tab == DashboardTab::SessionHistory {
                    app.history_first();
                }
            }
            KeyCode::End => {
                if app.dashboard_tab == DashboardTab::SessionHistory {
                    let count = app.filtered_sessions().len();
                    app.history_last(count);
                }
            }
            // Goes back to Setup rather than immediately starting a new
            // round, so the user can change level/training-mode between
            // rounds (or just hit Enter again to keep the same settings).
            KeyCode::Char('r') => app.mode = Mode::Setup,
            _ => {}
        },
    }
}
