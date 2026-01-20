use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use std::io;

/// Handle TagsModal mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_tags_modal(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = app.previous_mode;
        }
        KeyCode::Enter => {
            if let Err(e) = app.apply_tags_modal() {
                app.message = Some(format!("Failed to update tags: {}", e));
            }
        }
        KeyCode::Char(c) => {
            app.tags_input.push(c);
        }
        KeyCode::Backspace => {
            app.tags_input.pop();
        }
        _ => {}
    }

    Ok(false)
}
