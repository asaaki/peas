use crate::tui::app::App;
use crossterm::event::{Event, KeyCode, KeyEvent};
use std::io;

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
            // Pass all other events to textarea using the proper handle_events function
            if let Some(ref mut textarea) = app.body_textarea {
                let event = Event::Key(key);
                let _ = rat_text::text_area::handle_events(textarea, true, &event);
            }
        }
    }

    Ok(false)
}
