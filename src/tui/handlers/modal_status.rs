use crate::tui::app::App;
use crossterm::event::KeyEvent;
use std::io;

use super::modal_enum::handle_enum_modal;

/// Handle StatusModal mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_status_modal(app: &mut App, key: KeyEvent) -> io::Result<bool> {
    let options_count = App::status_options().len();
    handle_enum_modal(app, key, options_count, |app| {
        app.apply_modal_status()
            .map_err(io::Error::other)
    })
}
