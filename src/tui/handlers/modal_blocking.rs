use crate::tui::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};
use std::io;

/// Handle BlockingModal mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_blocking_modal(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = app.previous_mode;
        }
        KeyCode::Enter => {
            let _ = app.apply_modal_blocking();
        }
        KeyCode::Char(' ') => {
            app.toggle_blocking_selection();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let count = app.blocking_candidates.len();
            if count > 0 {
                app.modal_selection = (app.modal_selection + 1) % count;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let count = app.blocking_candidates.len();
            if count > 0 {
                app.modal_selection = if app.modal_selection == 0 {
                    count - 1
                } else {
                    app.modal_selection - 1
                };
            }
        }
        _ => {}
    }

    Ok(false)
}
