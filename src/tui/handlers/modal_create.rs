use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use std::io;

/// Handle CreateModal mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_create_modal(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = crate::tui::app::InputMode::Normal;
        }
        KeyCode::Enter => {
            let _ = app.create_from_modal();
        }
        KeyCode::Tab => {
            // Toggle between title (0) and type (1) fields
            app.modal_selection = (app.modal_selection + 1) % 2;
        }
        KeyCode::BackTab => {
            app.modal_selection = if app.modal_selection == 0 { 1 } else { 0 };
        }
        KeyCode::Char(c) => {
            if app.modal_selection == 0 {
                // Title field - add character
                app.create_title.push(c);
            } else {
                // Type field - cycle through types with space
                // (handled below)
            }
        }
        KeyCode::Backspace => {
            if app.modal_selection == 0 {
                app.create_title.pop();
            }
        }
        KeyCode::Left | KeyCode::Right => {
            if app.modal_selection == 1 {
                // Cycle type
                let types = App::type_options();
                let current_idx = types
                    .iter()
                    .position(|t| *t == app.create_type)
                    .unwrap_or(0);
                let new_idx = if key.code == KeyCode::Right {
                    (current_idx + 1) % types.len()
                } else if current_idx == 0 {
                    types.len() - 1
                } else {
                    current_idx - 1
                };
                app.create_type = types[new_idx];
            }
        }
        _ => {}
    }

    Ok(false)
}
