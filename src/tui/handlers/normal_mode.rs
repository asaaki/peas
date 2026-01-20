use crate::tui::app::{App, InputMode, ViewMode};
use cli_clipboard::ClipboardProvider;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

/// Handle Normal mode key events
/// Returns Ok(true) if the application should quit, Ok(false) otherwise
pub fn handle_normal_mode(
    app: &mut App,
    key: KeyEvent,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> io::Result<bool> {
    match key.code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('?') => app.show_help = !app.show_help,
        KeyCode::Tab => {
            app.switch_view();
        }
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
            } else if !app.multi_selected.is_empty() {
                app.clear_multi_select();
            } else if !app.search_query.is_empty() {
                app.search_query.clear();
                app.apply_filter();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Up | KeyCode::Char('k') => app.previous(),
        KeyCode::Right | KeyCode::PageDown | KeyCode::Char('J') => app.next_page(),
        KeyCode::Left | KeyCode::PageUp | KeyCode::Char('K') => app.previous_page(),
        KeyCode::Home | KeyCode::Char('g') => app.first(),
        KeyCode::End | KeyCode::Char('G') => app.last(),
        KeyCode::Char('/') => {
            app.input_mode = InputMode::Filter;
        }
        KeyCode::Enter => {
            match app.view_mode {
                ViewMode::Tickets => {
                    // Open full-screen detail view for tickets
                    if app.selected_pea().is_some() {
                        app.detail_scroll = 0;
                        app.build_relations();
                        app.input_mode = InputMode::DetailView;
                    }
                }
                ViewMode::Memory => {
                    // Open memory detail view
                    if app.selected_index < app.filtered_memories.len() {
                        app.detail_scroll = 0;
                        app.input_mode = InputMode::DetailView;
                    }
                }
            }
        }
        KeyCode::Char(' ') => {
            app.toggle_multi_select();
        }
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
        KeyCode::Char('c') => {
            app.open_create_modal();
        }
        KeyCode::Char('n') => {
            match app.view_mode {
                ViewMode::Memory => {
                    app.open_memory_create_modal();
                }
                ViewMode::Tickets => {
                    // 'n' is not used in Tickets view
                }
            }
        }
        KeyCode::Char('d') => {
            app.open_delete_confirm();
        }
        KeyCode::Char('r') => {
            let _ = app.refresh();
            app.message = Some("Refreshed".to_string());
        }
        KeyCode::Char('y') => {
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
        KeyCode::Char('e') => {
            if let Some(file_path) = app.selected_pea_file_path() {
                // Leave alternate screen temporarily
                disable_raw_mode()?;
                execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

                // Get editor from environment
                let editor = std::env::var("EDITOR")
                    .or_else(|_| std::env::var("VISUAL"))
                    .unwrap_or_else(|_| {
                        if cfg!(windows) {
                            "notepad".to_string()
                        } else {
                            "vi".to_string()
                        }
                    });

                // Spawn editor and wait
                let status = std::process::Command::new(&editor).arg(&file_path).status();

                // Re-enter alternate screen
                enable_raw_mode()?;
                execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                terminal.clear()?;

                // Refresh and show result
                let _ = app.refresh();
                match status {
                    Ok(s) if s.success() => {
                        app.message = Some("Editor closed".to_string());
                    }
                    Ok(_) => {
                        app.message = Some("Editor exited with error".to_string());
                    }
                    Err(e) => {
                        app.message = Some(format!("Failed to open editor: {}", e));
                    }
                }
            }
        }
        KeyCode::Char('u') => {
            let _ = app.undo();
        }
        _ => {}
    }

    Ok(false)
}
