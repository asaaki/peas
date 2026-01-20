use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use std::io;

/// Handle StatusModal mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_status_modal(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = app.previous_mode;
        }
        KeyCode::Enter => {
            let _ = app.apply_modal_status();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let count = App::status_options().len();
            app.modal_selection = (app.modal_selection + 1) % count;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let count = App::status_options().len();
            app.modal_selection = if app.modal_selection == 0 {
                count - 1
            } else {
                app.modal_selection - 1
            };
        }
        _ => {}
    }

    Ok(false)
}
