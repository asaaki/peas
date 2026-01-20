use crate::tui::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};
use std::io;

/// Handle MemoryCreateModal mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_memory_create_modal(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Enter => {
            let _ = app.create_memory_from_modal();
        }
        KeyCode::Tab => {
            // Cycle between key (0), tags (1), and content (2) fields
            app.memory_modal_selection = (app.memory_modal_selection + 1) % 3;
        }
        KeyCode::BackTab => {
            app.memory_modal_selection = if app.memory_modal_selection == 0 {
                2
            } else {
                app.memory_modal_selection - 1
            };
        }
        KeyCode::Char(c) => match app.memory_modal_selection {
            0 => app.memory_create_key.push(c),
            1 => app.memory_create_tags.push(c),
            2 => app.memory_create_content.push(c),
            _ => {}
        },
        KeyCode::Backspace => match app.memory_modal_selection {
            0 => {
                app.memory_create_key.pop();
            }
            1 => {
                app.memory_create_tags.pop();
            }
            2 => {
                app.memory_create_content.pop();
            }
            _ => {}
        },
        _ => {}
    }

    Ok(false)
}
