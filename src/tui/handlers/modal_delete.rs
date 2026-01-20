use crate::tui::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};
use std::io;

/// Handle DeleteConfirm mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_delete_confirm(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
            let _ = app.delete_selected();
        }
        _ => {}
    }

    Ok(false)
}
