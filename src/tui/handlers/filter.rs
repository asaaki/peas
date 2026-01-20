use crate::tui::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};
use std::io;

/// Handle Filter mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_filter_mode(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Enter | KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.apply_filter();
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.apply_filter();
        }
        _ => {}
    }

    Ok(false)
}
