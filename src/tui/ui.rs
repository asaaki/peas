use super::app::{App, InputMode};
use super::ui_modals;
use super::ui_views;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    // Full-screen detail view when in DetailView or EditBody mode
    if app.input_mode == InputMode::DetailView || app.input_mode == InputMode::EditBody {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Full detail
                Constraint::Length(1), // Footer (keybindings only)
            ])
            .split(f.area());

        match app.view_mode {
            super::app::ViewMode::Tickets => {
                ui_views::draw_detail_fullscreen(f, app, chunks[0], app.detail_scroll)
            }
            super::app::ViewMode::Memory => {
                ui_views::draw_memory_detail(f, app, chunks[0], app.detail_scroll)
            }
        }
        ui_views::draw_footer(f, app, chunks[1]);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content (tree view or memory list)
            Constraint::Length(1), // Footer (keybindings only)
        ])
        .split(f.area());

    match app.view_mode {
        super::app::ViewMode::Tickets => ui_views::draw_tree(f, app, chunks[0]),
        super::app::ViewMode::Memory => ui_views::draw_memory_list(f, app, chunks[0]),
    }
    ui_views::draw_footer(f, app, chunks[1]);

    if app.show_help {
        ui_views::draw_help_popup(f);
    }

    // Draw modal if active
    match app.input_mode {
        InputMode::StatusModal => ui_modals::draw_status_modal(f, app),
        InputMode::PriorityModal => ui_modals::draw_priority_modal(f, app),
        InputMode::TypeModal => ui_modals::draw_type_modal(f, app),
        InputMode::DeleteConfirm => ui_modals::draw_delete_confirm(f, app),
        InputMode::ParentModal => ui_modals::draw_parent_modal(f, app),
        InputMode::BlockingModal => ui_modals::draw_blocking_modal(f, app),
        InputMode::CreateModal => ui_modals::draw_create_modal(f, app),
        InputMode::MemoryCreateModal => ui_modals::draw_memory_create_modal(f, app),
        InputMode::TagsModal => ui_modals::draw_tags_modal(f, app),
        InputMode::UrlModal => ui_modals::draw_url_modal(f, app),
        _ => {}
    }
}
