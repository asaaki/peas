use super::app::App;
use super::theme::{theme, tui_config};
use super::ui_utils;
use ratatui::{
    Frame,
    style::{Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub fn draw_status_modal(f: &mut Frame, app: &App) {
    let area = ui_utils::centered_rect(30, 30, f.area());
    let t = theme();

    let options = App::status_options();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, status)| {
            let is_selected = idx == app.modal_selection;
            let (icon, color) = ui_utils::status_indicator(status);

            let selection_indicator = if is_selected {
                Span::styled(theme().row_marker, Style::default().fg(t.modal_cursor))
            } else {
                Span::raw(" ")
            };

            let style = if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                selection_indicator,
                Span::styled(format!("{} ", icon), Style::default().fg(color)),
                Span::styled(format!("{}", status), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Status ")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(t.modal_border)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

pub fn draw_priority_modal(f: &mut Frame, app: &App) {
    let area = ui_utils::centered_rect(30, 30, f.area());
    let t = theme();

    let options = App::priority_options();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, priority)| {
            let is_selected = idx == app.modal_selection;
            let color = ui_utils::priority_color(priority);

            let selection_indicator = if is_selected {
                Span::styled(theme().row_marker, Style::default().fg(t.modal_cursor))
            } else {
                Span::raw(" ")
            };

            let style = if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                selection_indicator,
                Span::styled(format!("{}", priority), style.fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Priority ")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(t.modal_border)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

pub fn draw_delete_confirm(f: &mut Frame, app: &App) {
    let area = ui_utils::centered_rect(50, 20, f.area());
    let t = theme();

    let (question, item_info) = match app.view_mode {
        super::app::ViewMode::Tickets => {
            let pea_info = if let Some(pea) = app.selected_pea() {
                format!("{} - {}", pea.id, pea.title)
            } else {
                "No ticket selected".to_string()
            };
            ("Are you sure you want to delete this ticket?", pea_info)
        }
        super::app::ViewMode::Memory => {
            let memory_info = if let Some(memory) = app.filtered_memories.get(app.selected_index) {
                memory.key.clone()
            } else {
                "No memory selected".to_string()
            };
            ("Are you sure you want to delete this memory?", memory_info)
        }
    };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            question,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(item_info, Style::default().fg(t.id))),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "y",
                Style::default()
                    .fg(t.checkbox_checked)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/Enter = Yes    "),
            Span::styled(
                "n",
                Style::default()
                    .fg(t.modal_border_delete)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/Esc = No"),
        ]),
    ];

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Delete Confirmation ")
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(t.modal_border_delete)),
        )
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

pub fn draw_tags_modal(f: &mut Frame, app: &App) {
    let area = ui_utils::centered_rect(60, 20, f.area());
    let t = theme();

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Tags: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&app.tags_input),
            Span::styled("_", Style::default().fg(t.modal_cursor)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Enter comma-separated tags (e.g., bug, ui, performance)",
            Style::default().fg(t.text_muted),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  Press Enter to save, Esc to cancel",
            Style::default().fg(t.text_muted),
        )),
    ];

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .title(" Edit Tags ")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(t.modal_border)),
    );

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

pub fn draw_url_modal(f: &mut Frame, app: &App) {
    let area = ui_utils::centered_rect(80, 60, f.area());
    let t = theme();

    // Compute pulsing color for row marker
    let elapsed_millis = app.start_time.elapsed().as_millis();
    let pulsing_color = theme().selection_indicator_pulsing_color(elapsed_millis);
    let pulsing_style = Style::default().fg(pulsing_color);

    let mut content = vec![
        Line::from(""),
        Line::from(Span::styled(
            " URLs found in ticket body:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Add each URL as a selectable item
    for (i, url) in app.url_candidates.iter().enumerate() {
        let is_selected = i == app.modal_selection;
        let marker = if is_selected {
            Span::styled(format!(" {} ", theme().row_marker), pulsing_style)
        } else {
            Span::raw("   ")
        };

        // Truncate long URLs for display
        let display_url = if url.len() > 70 {
            format!("{}...", &url[..67])
        } else {
            url.clone()
        };

        let url_style = if is_selected {
            Style::default()
                .fg(t.text_highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text)
        };

        content.push(Line::from(vec![
            marker,
            Span::styled(display_url, url_style),
        ]));
    }

    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        "  ↓/↑: Navigate  Enter: Open URL  Esc: Cancel",
        Style::default().fg(t.text_muted),
    )));

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .title(format!(
                " Open URL ({}/{}) ",
                app.modal_selection + 1,
                app.url_candidates.len()
            ))
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(t.modal_border)),
    );

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

pub fn draw_create_modal(f: &mut Frame, app: &App) {
    let area = ui_utils::centered_rect(50, 25, f.area());
    let t = theme();

    let title_active = app.modal_selection == 0;
    let type_active = app.modal_selection == 1;

    // Build display text for title field
    let title_display = if app.create_title.is_empty() {
        Span::styled("Enter title...", Style::default().fg(t.text_muted))
    } else {
        Span::raw(app.create_title.clone())
    };

    let title_style = if title_active {
        Style::default().fg(t.modal_cursor)
    } else {
        Style::default().fg(t.text)
    };

    let type_style = if type_active {
        Style::default().fg(t.modal_cursor)
    } else {
        Style::default().fg(t.text)
    };

    let pea_type_color = ui_utils::type_color(&app.create_type);

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if title_active { "▶ " } else { "  " },
                Style::default().fg(t.modal_cursor),
            ),
            Span::styled("Title: ", title_style.add_modifier(Modifier::BOLD)),
            title_display,
            if title_active {
                Span::styled("_", Style::default().fg(t.modal_cursor))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if type_active { "▶ " } else { "  " },
                Style::default().fg(t.modal_cursor),
            ),
            Span::styled("Type:  ", type_style.add_modifier(Modifier::BOLD)),
            Span::styled(
                if tui_config().use_type_emojis {
                    format!(
                        "< {} {} >",
                        theme().type_emoji(&app.create_type),
                        app.create_type
                    )
                } else {
                    format!("< {} >", app.create_type)
                },
                Style::default().fg(pea_type_color),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  (use ←/→ to change type)",
            Style::default().fg(t.text_muted),
        )),
    ];

    // Show parent info if current selection would become parent
    let parent_info = app.selected_pea().and_then(|p| {
        if matches!(
            p.pea_type,
            crate::model::PeaType::Milestone
                | crate::model::PeaType::Epic
                | crate::model::PeaType::Story
                | crate::model::PeaType::Feature
        ) {
            Some(format!("  Parent: {} ({})", p.id, p.title))
        } else {
            None
        }
    });

    let mut all_content = content;
    if let Some(info) = parent_info {
        all_content.push(Line::from(""));
        all_content.push(Line::from(Span::styled(
            info,
            Style::default().fg(t.text_muted),
        )));
    }

    let paragraph = Paragraph::new(all_content).block(
        Block::default()
            .title(" Create Ticket ")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(t.modal_border_create)),
    );

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

pub fn draw_memory_create_modal(f: &mut Frame, app: &App) {
    let area = ui_utils::centered_rect(60, 40, f.area());
    let t = theme();

    let key_active = app.memory_modal_selection == 0;
    let tags_active = app.memory_modal_selection == 1;
    let content_active = app.memory_modal_selection == 2;

    // Build display text for key field
    let key_display = if app.memory_create_key.is_empty() {
        Span::styled("Enter key...", Style::default().fg(t.text_muted))
    } else {
        Span::raw(app.memory_create_key.clone())
    };

    // Build display text for tags field
    let tags_display = if app.memory_create_tags.is_empty() {
        Span::styled("tag1, tag2, ...", Style::default().fg(t.text_muted))
    } else {
        Span::raw(app.memory_create_tags.clone())
    };

    // Build display text for content field
    let content_display = if app.memory_create_content.is_empty() {
        Span::styled("Enter content...", Style::default().fg(t.text_muted))
    } else {
        Span::raw(app.memory_create_content.clone())
    };

    let key_style = if key_active {
        Style::default().fg(t.modal_cursor)
    } else {
        Style::default().fg(t.text)
    };

    let tags_style = if tags_active {
        Style::default().fg(t.modal_cursor)
    } else {
        Style::default().fg(t.text)
    };

    let content_style = if content_active {
        Style::default().fg(t.modal_cursor)
    } else {
        Style::default().fg(t.text)
    };

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if key_active { "▶ " } else { "  " },
                Style::default().fg(t.modal_cursor),
            ),
            Span::styled("Key:     ", key_style.add_modifier(Modifier::BOLD)),
            key_display,
            if key_active {
                Span::styled("_", Style::default().fg(t.modal_cursor))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if tags_active { "▶ " } else { "  " },
                Style::default().fg(t.modal_cursor),
            ),
            Span::styled("Tags:    ", tags_style.add_modifier(Modifier::BOLD)),
            tags_display,
            if tags_active {
                Span::styled("_", Style::default().fg(t.modal_cursor))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if content_active { "▶ " } else { "  " },
                Style::default().fg(t.modal_cursor),
            ),
            Span::styled("Content: ", content_style.add_modifier(Modifier::BOLD)),
            content_display,
            if content_active {
                Span::styled("_", Style::default().fg(t.modal_cursor))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  (Tab to switch fields, Enter to create)",
            Style::default().fg(t.text_muted),
        )),
    ];

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .title(" Create Memory ")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(t.modal_border_create)),
    );

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

pub fn draw_blocking_modal(f: &mut Frame, app: &App) {
    let area = ui_utils::centered_rect(60, 50, f.area());
    let t = theme();

    let items: Vec<ListItem> = app
        .blocking_candidates
        .iter()
        .zip(app.blocking_selected.iter())
        .enumerate()
        .map(|(idx, (pea, &is_checked))| {
            let is_cursor = idx == app.modal_selection;

            // Cursor indicator
            let cursor = if is_cursor {
                Span::styled(theme().row_marker, Style::default().fg(t.modal_cursor))
            } else {
                Span::raw(" ")
            };

            // Checkbox
            let checkbox = if is_checked {
                Span::styled("[x] ", Style::default().fg(t.checkbox_checked))
            } else {
                Span::styled("[ ] ", Style::default().fg(t.checkbox_unchecked))
            };

            let (status_icon, status_color) = ui_utils::status_indicator(&pea.status);

            let style = if is_cursor {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Truncate title if too long
            let max_title_len = 30;
            let title = if pea.title.len() > max_title_len {
                format!("{}...", &pea.title[..max_title_len - 3])
            } else {
                pea.title.clone()
            };

            ListItem::new(Line::from(vec![
                cursor,
                checkbox,
                Span::styled(
                    format!("{} ", status_icon),
                    Style::default().fg(status_color),
                ),
                Span::styled(&pea.id, Style::default().fg(t.id)),
                Span::raw(" "),
                Span::styled(title, style),
            ]))
        })
        .collect();

    let selected_count = app.blocking_selected.iter().filter(|&&s| s).count();
    let title = format!(" Select Blocking Tickets ({} selected) ", selected_count);

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(t.modal_border)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

pub fn draw_parent_modal(f: &mut Frame, app: &App) {
    // Use a larger area for parent modal since it can have many options
    let area = ui_utils::centered_rect(60, 50, f.area());
    let t = theme();

    // Build items: first is "(none)", then all candidates
    let mut items: Vec<ListItem> = Vec::new();

    // "(none)" option
    let is_none_selected = app.modal_selection == 0;
    let none_indicator = if is_none_selected {
        Span::styled(theme().row_marker, Style::default().fg(t.modal_cursor))
    } else {
        Span::raw(" ")
    };
    let none_style = if is_none_selected {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    items.push(ListItem::new(Line::from(vec![
        none_indicator,
        Span::styled("(none)", none_style.fg(t.text_muted)),
    ])));

    // Candidate options
    for (idx, pea) in app.parent_candidates.iter().enumerate() {
        let is_selected = app.modal_selection == idx + 1;
        let selection_indicator = if is_selected {
            Span::styled(theme().row_marker, Style::default().fg(t.modal_cursor))
        } else {
            Span::raw(" ")
        };

        let style = if is_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let type_col = ui_utils::type_color(&pea.pea_type);

        // Truncate title if too long
        let max_title_len = 35;
        let title = if pea.title.len() > max_title_len {
            format!("{}...", &pea.title[..max_title_len - 3])
        } else {
            pea.title.clone()
        };

        let type_text = if tui_config().use_type_emojis {
            format!("[{} {}]", theme().type_emoji(&pea.pea_type), pea.pea_type)
        } else {
            format!("[{}]", pea.pea_type)
        };

        items.push(ListItem::new(Line::from(vec![
            selection_indicator,
            Span::styled(&pea.id, Style::default().fg(t.id)),
            Span::raw(" "),
            Span::styled(type_text, Style::default().fg(type_col)),
            Span::raw(" "),
            Span::styled(title, style),
        ])));
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Select Parent ")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(t.modal_border)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

pub fn draw_type_modal(f: &mut Frame, app: &App) {
    let area = ui_utils::centered_rect(30, 35, f.area());
    let t = theme();

    let options = App::type_options();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, pea_type)| {
            let is_selected = idx == app.modal_selection;
            let color = ui_utils::type_color(pea_type);

            let selection_indicator = if is_selected {
                Span::styled(theme().row_marker, Style::default().fg(t.modal_cursor))
            } else {
                Span::raw(" ")
            };

            let style = if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let type_text = if tui_config().use_type_emojis {
                format!("{} {}", theme().type_emoji(pea_type), pea_type)
            } else {
                format!("{}", pea_type)
            };

            ListItem::new(Line::from(vec![
                selection_indicator,
                Span::styled(type_text, style.fg(color)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Type ")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(t.modal_border)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}
