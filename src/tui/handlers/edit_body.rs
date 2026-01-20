use crate::tui::app::App;
use crossterm::event::{Event, KeyCode, KeyEvent};
use std::io;
use tui_textarea::Input;

/// Handle EditBody mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_edit_body(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc => {
            app.cancel_body_edit();
        }
        KeyCode::Char('s')
            if key
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL) =>
        {
            if let Err(e) = app.save_body_edit() {
                app.message = Some(format!("Save failed: {}", e));
            } else {
                app.message = Some("Saved successfully".to_string());
            }
        }
        _ => {
            // Pass all other events to textarea
            if let Some(ref mut textarea) = app.body_textarea {
                let event = Event::Key(key);
                textarea.input(Input::from(event));
            }
        }
    }

    Ok(false)
}
