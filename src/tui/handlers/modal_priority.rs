use crate::tui::app::App;
use crossterm::event::KeyEvent;
use std::io;

use super::modal_enum::handle_enum_modal;

/// Handle PriorityModal mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_priority_modal(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    let options_count = App::priority_options().len();
    handle_enum_modal(app, key, options_count, |app| {
        app.apply_modal_priority()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    })
}
