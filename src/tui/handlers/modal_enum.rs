use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use std::io;

/// Generic handler for enum selection modals (Status, Priority, Type)
///
/// This eliminates code duplication by providing a common implementation
/// for all enum-based modal selection interfaces.
pub fn handle_enum_modal<F>(
    app: &mut App,
    key: KeyEvent,
    options_count: usize,
    apply_fn: F,
) -> io::Result<bool>
where
    F: FnOnce(&mut App) -> io::Result<()>,
{
    match key.code {
        KeyCode::Esc => {
            app.input_mode = app.previous_mode;
        }
        KeyCode::Enter => {
            let _ = apply_fn(app);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.modal_selection = (app.modal_selection + 1) % options_count;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.modal_selection = if app.modal_selection == 0 {
                options_count - 1
            } else {
                app.modal_selection - 1
            };
        }
        _ => {}
    }

    Ok(false)
}
