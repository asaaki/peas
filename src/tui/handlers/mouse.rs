use crate::tui::app::{App, InputMode};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

/// Handle mouse events
pub fn handle_mouse(app: &mut App, mouse_event: MouseEvent) {
    match mouse_event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Handle click events
            app.handle_mouse_click(mouse_event.column, mouse_event.row);
        }
        MouseEventKind::ScrollDown => {
            if app.input_mode == InputMode::Normal {
                app.next();
            } else if app.input_mode == InputMode::DetailView {
                // Scroll detail view down
                if app.detail_scroll < app.detail_max_scroll {
                    app.detail_scroll += 1;
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if app.input_mode == InputMode::Normal {
                app.previous();
            } else if app.input_mode == InputMode::DetailView {
                // Scroll detail view up
                if app.detail_scroll > 0 {
                    app.detail_scroll -= 1;
                }
            }
        }
        _ => {}
    }
}
