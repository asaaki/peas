use crate::tui::app::{App, DetailPane, InputMode};
use cli_clipboard::ClipboardProvider;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

/// Handle DetailView mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_detail_view(
    app: &mut App,
    key: KeyEvent,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.input_mode = InputMode::Normal;
            app.detail_pane = DetailPane::Body;
        }
        KeyCode::Tab => {
            app.toggle_detail_pane();
        }
        KeyCode::Enter => {
            // Open modal for selected metadata property, jump to relation, or open asset
            if app.detail_pane == DetailPane::Metadata {
                match app.metadata_selection {
                    0 => app.open_type_modal(),     // Type
                    1 => app.open_status_modal(),   // Status
                    2 => app.open_priority_modal(), // Priority
                    3 => app.open_tags_modal(),     // Tags
                    _ => {}
                }
            } else if app.detail_pane == DetailPane::Relations && !app.relations_items.is_empty() {
                app.jump_to_relation();
            } else if app.detail_pane == DetailPane::Assets && !app.assets_items.is_empty() {
                let _ = app.open_selected_asset();
            } else {
                app.input_mode = InputMode::Normal;
                app.detail_pane = DetailPane::Body;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => match app.detail_pane {
            DetailPane::Metadata => {
                // Navigate down through metadata properties (type, status, priority, tags)
                if app.metadata_selection < 3 {
                    app.metadata_selection += 1;
                }
            }
            DetailPane::Body => app.scroll_detail_down(),
            DetailPane::Relations => app.relations_next(),
            DetailPane::Assets => app.assets_next(),
        },
        KeyCode::Up | KeyCode::Char('k') => match app.detail_pane {
            DetailPane::Metadata => {
                // Navigate up through metadata properties
                if app.metadata_selection > 0 {
                    app.metadata_selection -= 1;
                }
            }
            DetailPane::Body => app.scroll_detail_up(),
            DetailPane::Relations => app.relations_previous(),
            DetailPane::Assets => app.assets_previous(),
        },
        KeyCode::Char('J') => {
            // Always scroll body
            app.scroll_detail_down();
        }
        KeyCode::Char('K') => {
            // Always scroll body
            app.scroll_detail_up();
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                app.scroll_detail_down();
            }
        }
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.scroll_detail_up();
            }
        }
        KeyCode::Char('e') => {
            // Start inline editing
            app.start_body_edit();
        }
        KeyCode::Char('E') => {
            // External editor (uppercase E)
            if let Some(file_path) = app.selected_pea_file_path() {
                disable_raw_mode()?;
                execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

                let editor = std::env::var("EDITOR")
                    .or_else(|_| std::env::var("VISUAL"))
                    .unwrap_or_else(|_| {
                        if cfg!(windows) {
                            "notepad".to_string()
                        } else {
                            "vi".to_string()
                        }
                    });

                let _ = std::process::Command::new(&editor).arg(&file_path).status();

                enable_raw_mode()?;
                execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                terminal.clear()?;
                let _ = app.refresh();
                app.build_relations(); // Rebuild relations after edit
            }
        }
        // Property editing hotkeys (same as normal mode)
        KeyCode::Char('s') => {
            app.open_status_modal();
        }
        KeyCode::Char('P') => {
            app.open_priority_modal();
        }
        KeyCode::Char('t') => {
            app.open_type_modal();
        }
        KeyCode::Char('p') => {
            app.open_parent_modal();
        }
        KeyCode::Char('b') => {
            app.open_blocking_modal();
        }
        KeyCode::Char('y') => {
            // Copy ticket ID to clipboard
            if let Some(pea) = app.selected_pea() {
                let id = pea.id.clone();
                if let Ok(mut ctx) = cli_clipboard::ClipboardContext::new() {
                    if ctx.set_contents(id.clone()).is_ok() {
                        app.message = Some(format!("Copied: {}", id));
                    } else {
                        app.message = Some("Failed to copy to clipboard".to_string());
                    }
                } else {
                    app.message = Some("Clipboard not available".to_string());
                }
            }
        }
        KeyCode::Char('o') => {
            // Open URL selection modal
            app.open_url_modal();
        }
        _ => {}
    }

    Ok(false)
}
