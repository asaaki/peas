use super::app::{App, DetailPane, InputMode};
use super::theme::theme;
use crate::model::{Pea, PeaPriority, PeaStatus, PeaType};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

/// Estimate the number of wrapped lines for a Text widget
fn estimate_wrapped_lines(text: &Text, width: usize) -> u16 {
    if width == 0 {
        return 0;
    }
    let mut total_lines = 0u16;
    for line in &text.lines {
        let line_width: usize = line.spans.iter().map(|s| s.content.len()).sum();
        let wrapped = if line_width == 0 {
            1 // Empty line still takes 1 line
        } else {
            ((line_width + width - 1) / width) as u16 // Ceiling division
        };
        total_lines = total_lines.saturating_add(wrapped);
    }
    total_lines
}

/// Returns priority indicator and color for a pea
fn priority_indicator(pea: &Pea) -> Option<(String, Color)> {
    theme()
        .priority_indicator(&pea.priority)
        .map(|(s, c)| (s.to_string(), c))
}

/// Returns status icon and color
fn status_indicator(status: &PeaStatus) -> (&'static str, Color) {
    theme().status_indicator(status)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    // Full-screen detail view when in DetailView mode
    if app.input_mode == InputMode::DetailView {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Full detail
                Constraint::Length(1), // Footer (keybindings only)
            ])
            .split(f.area());

        draw_detail_fullscreen(f, app, chunks[0], app.detail_scroll);
        draw_footer(f, app, chunks[1]);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content (tree view)
            Constraint::Length(1), // Footer (keybindings only)
        ])
        .split(f.area());

    draw_tree(f, app, chunks[0]);
    draw_footer(f, app, chunks[1]);

    if app.show_help {
        draw_help_popup(f);
    }

    // Draw modal if active
    match app.input_mode {
        InputMode::StatusModal => draw_status_modal(f, app),
        InputMode::PriorityModal => draw_priority_modal(f, app),
        InputMode::TypeModal => draw_type_modal(f, app),
        InputMode::DeleteConfirm => draw_delete_confirm(f, app),
        InputMode::ParentModal => draw_parent_modal(f, app),
        InputMode::BlockingModal => draw_blocking_modal(f, app),
        InputMode::CreateModal => draw_create_modal(f, app),
        _ => {}
    }
}

fn draw_tree(f: &mut Frame, app: &mut App, area: Rect) {
    // First pass: calculate page height without page dots to determine if we need them
    let base_page_height = area.height.saturating_sub(2) as usize;
    let item_count = app.tree_nodes.len();
    let needs_page_dots = item_count > base_page_height;

    // If we need page dots, reduce available height by 2 (empty line + dots line)
    let page_height = if needs_page_dots {
        area.height.saturating_sub(4) as usize
    } else {
        base_page_height
    };
    app.page_height = page_height.max(1);

    // Get the index within the current page for highlighting
    let index_in_page = app.index_in_page();

    // Only render items for the current page
    let page_items = app.current_page_items();
    let rows: Vec<Row> = page_items
        .iter()
        .enumerate()
        .map(|(idx, node)| {
            let pea = &node.pea;
            let is_selected = idx == index_in_page;
            let is_multi_selected = app.is_multi_selected(&pea.id);
            let (status_icon, status_color) = status_indicator(&pea.status);
            let pea_type_color = type_color(&pea.pea_type);

            // Build the tree prefix with rounded corners
            let mut prefix = String::new();
            for &has_line in &node.parent_lines {
                if has_line {
                    prefix.push_str("│  ");
                } else {
                    prefix.push_str("   ");
                }
            }
            if node.depth > 0 {
                if node.is_last {
                    prefix.push_str("╰─ ");
                } else {
                    prefix.push_str("├─ ");
                }
            }

            // Selection indicator (green, blinking)
            let sel = if is_selected { "▌" } else { " " };
            let sel_style = if is_selected {
                theme().selection_indicator_style()
            } else {
                Style::default()
            };

            // Multi-select checkbox
            let checkbox = if is_multi_selected { "◆" } else { " " };
            let checkbox_style = Style::default().fg(theme().multi_select);

            // Priority indicator
            let pri = if let Some((ind, _)) = priority_indicator(pea) {
                ind
            } else {
                String::new()
            };
            let pri_color = priority_indicator(pea)
                .map(|(_, c)| c)
                .unwrap_or(Color::Reset);

            // Title style
            let title_style = if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Tree + ID combined in one cell (so tree connects to ID visually)
            // ID is bold and bright green when selected
            let id_style = theme().id_style(is_selected);
            let tree_and_id = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme().tree_lines)),
                Span::styled(pea.id.clone(), id_style),
            ]);

            // Type and status styles (bold when selected)
            let type_style = if is_selected {
                Style::default()
                    .fg(pea_type_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(pea_type_color)
            };
            let status_style = if is_selected {
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(status_color)
            };

            // Build cells for each column
            Row::new(vec![
                Cell::from(sel).style(sel_style),
                Cell::from(checkbox).style(checkbox_style),
                Cell::from(tree_and_id),
                Cell::from(format!("{}", pea.pea_type)).style(type_style),
                Cell::from(format!("{} {}", status_icon, pea.status)).style(status_style),
                Cell::from(pri).style(Style::default().fg(pri_color)),
                Cell::from(pea.title.clone()).style(title_style),
            ])
        })
        .collect();

    // Title shows count and selection count if any
    let selection_count = app.multi_select_count();
    let title = if selection_count > 0 {
        format!(
            " peas ({}) [{} selected] ",
            app.tree_nodes.len(),
            selection_count
        )
    } else {
        format!(" peas ({}) ", app.tree_nodes.len())
    };

    // Page dots for bottom of panel (recalculate after page_height is set)
    let total_pages = app.total_pages();
    let current_page = app.current_page();

    // Define column widths:
    // sel(1), checkbox(1), tree+id(20), type(12), status(14), priority(1), title(fill)
    let widths = [
        Constraint::Length(1),  // Selection indicator
        Constraint::Length(1),  // Multi-select checkbox
        Constraint::Length(20), // Tree prefix + ID combined
        Constraint::Length(12), // Type
        Constraint::Length(14), // Status (icon + text)
        Constraint::Length(1),  // Priority (single char)
        Constraint::Fill(1),    // Title (fills remaining space)
    ];

    // Render the outer block first and get inner area
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme().border));
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Split inner area if we need page dots (with empty line above)
    let (table_area, page_dots_area) = if needs_page_dots {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Table rows
                Constraint::Length(2), // Empty line + page dots line
            ])
            .split(inner_area);
        // Offset the dots area by 1 to leave empty line, and 1 space padding on left
        let dots_area = Rect {
            x: chunks[1].x + 1,
            y: chunks[1].y + 1,
            width: chunks[1].width.saturating_sub(1),
            height: 1,
        };
        (chunks[0], Some(dots_area))
    } else {
        (inner_area, None)
    };

    // Table without its own block (we already rendered the outer block)
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .row_highlight_style(Style::default());

    // Use a fresh table state for page-local selection
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(index_in_page));
    f.render_stateful_widget(table, table_area, &mut table_state);

    // Render page dots inside panel if needed
    if let Some(dots_area) = page_dots_area {
        let dots: Vec<Span> = (0..total_pages)
            .map(|i| {
                if i == current_page {
                    Span::styled("•", Style::default().fg(theme().text_highlight))
                } else {
                    Span::styled("•", Style::default().fg(theme().text_muted))
                }
            })
            .collect();
        let dots_line = Line::from(dots);
        let dots_paragraph = Paragraph::new(dots_line);
        f.render_widget(dots_paragraph, dots_area);
    }
}

/// Get color for type (without the indicator character)
fn type_color(pea_type: &PeaType) -> Color {
    theme().type_color(pea_type)
}

fn draw_detail_fullscreen(f: &mut Frame, app: &mut App, area: Rect, detail_scroll: u16) {
    let detail_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme().border_focused));

    if let Some(pea) = app.selected_pea().cloned() {
        let status_color = theme().status_color(&pea.status);
        let pea_priority_color = priority_color(&pea.priority);

        // Check if we have body content
        let has_body = !pea.body.is_empty();
        let has_relations = !app.relations_items.is_empty();
        let body_content = pea.body.clone();

        // Layout: Top section (metadata + relations) | Bottom section (body)
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_body {
                vec![
                    Constraint::Length(12), // Top section (metadata + relations)
                    Constraint::Min(5),     // Body
                ]
            } else {
                vec![Constraint::Min(0)]
            })
            .split(area);

        let top_area = vertical_chunks[0];
        let body_area = if has_body {
            Some(vertical_chunks[1])
        } else {
            None
        };

        // Split top area horizontally: metadata (left) | relations (right)
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(if has_relations {
                vec![Constraint::Percentage(50), Constraint::Percentage(50)]
            } else {
                vec![Constraint::Percentage(100)]
            })
            .split(top_area);

        let metadata_area = top_chunks[0];
        let relations_area = if has_relations {
            Some(top_chunks[1])
        } else {
            None
        };

        // Build metadata lines
        let t = theme();
        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    &pea.id,
                    Style::default().fg(t.id).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(&pea.title, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Type:     "),
                Span::styled(
                    format!("{}", pea.pea_type),
                    Style::default().fg(type_color(&pea.pea_type)),
                ),
            ]),
            Line::from(vec![
                Span::raw("Status:   "),
                Span::styled(format!("{}", pea.status), Style::default().fg(status_color)),
            ]),
            Line::from(vec![
                Span::raw("Priority: "),
                Span::styled(
                    format!("{}", pea.priority),
                    Style::default().fg(pea_priority_color),
                ),
            ]),
        ];

        if !pea.tags.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("Tags:     "),
                Span::styled(pea.tags.join(", "), Style::default().fg(theme().tags)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("Created:  "),
            Span::styled(
                pea.created.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(theme().timestamp),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("Updated:  "),
            Span::styled(
                pea.updated.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(theme().timestamp),
            ),
        ]));

        // Render metadata section
        let is_metadata_focused = app.detail_pane == DetailPane::Metadata;
        let metadata_block = Block::default()
            .title(format!(" {} ", pea.id))
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(theme().border_style(is_metadata_focused));

        let metadata = Paragraph::new(Text::from(lines))
            .block(metadata_block)
            .wrap(Wrap { trim: true });

        f.render_widget(metadata, metadata_area);

        // Render relationships pane if there are any
        if let Some(rel_area) = relations_area {
            let rel_count = app.relations_items.len();
            let is_focused = app.detail_pane == DetailPane::Relations;
            let relations_block = Block::default()
                .title(format!(" Relationships ({}) ", rel_count))
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(theme().border_style(is_focused));

            let inner = relations_block.inner(rel_area);
            f.render_widget(relations_block, rel_area);

            // Build list items for relationships
            let items: Vec<ListItem> = app
                .relations_items
                .iter()
                .enumerate()
                .map(|(i, (rel_type, id, title))| {
                    let is_selected = i == app.relations_selection;
                    let prefix = super::theme::Theme::relation_prefix(rel_type);
                    let rel_color = theme().relation_color(rel_type);

                    // Selection cursor
                    let cursor = if is_selected {
                        Span::styled("▌ ", theme().selection_indicator_style())
                    } else {
                        Span::raw("  ")
                    };

                    let content = Line::from(vec![
                        cursor,
                        Span::styled(format!("{} ", prefix), Style::default().fg(rel_color)),
                        Span::styled(format!("[{}] ", rel_type), Style::default().fg(rel_color)),
                        Span::styled(id, Style::default().fg(theme().id)),
                        Span::raw(" "),
                        Span::styled(
                            if title.len() > 25 {
                                format!("{}...", &title[..22])
                            } else {
                                title.clone()
                            },
                            Style::default().fg(theme().text_muted),
                        ),
                    ]);

                    if is_selected {
                        ListItem::new(content).style(Style::default().add_modifier(Modifier::BOLD))
                    } else {
                        ListItem::new(content)
                    }
                })
                .collect();

            let list = List::new(items);
            f.render_widget(list, inner);
        }

        // Render body section with tui-markdown
        if let Some(body_rect) = body_area {
            let body_focused = app.detail_pane == DetailPane::Body;
            let body_block = Block::default()
                .title(" Description ")
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(theme().border_style(body_focused));

            let inner = body_block.inner(body_rect);
            f.render_widget(body_block, body_rect);

            // Render markdown using tui-markdown
            let md_text = tui_markdown::from_str(&body_content);

            // Calculate content height for scroll limiting
            let view_height = inner.height as u16;
            let content_lines = estimate_wrapped_lines(&md_text, inner.width as usize);
            let max_scroll = content_lines.saturating_sub(view_height);
            app.set_detail_max_scroll(max_scroll);

            let md_paragraph = Paragraph::new(md_text)
                .wrap(Wrap { trim: false })
                .scroll((detail_scroll, 0));
            f.render_widget(md_paragraph, inner);
        } else {
            // No body, no scrolling needed
            app.set_detail_max_scroll(0);
        }
    } else {
        let empty = Paragraph::new("No pea selected")
            .block(detail_block)
            .style(Style::default().fg(theme().text_muted));
        f.render_widget(empty, area);
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    // Mode indicator - use theme colors
    let t = theme();
    let mode_indicator = match app.input_mode {
        InputMode::Normal => Span::styled(
            " NORMAL ",
            Style::default().bg(t.mode_normal.0).fg(t.mode_normal.1),
        ),
        InputMode::Filter => Span::styled(
            " SEARCH ",
            Style::default().bg(t.mode_search.0).fg(t.mode_search.1),
        ),
        InputMode::StatusModal => Span::styled(
            " STATUS ",
            Style::default().bg(t.mode_status.0).fg(t.mode_status.1),
        ),
        InputMode::PriorityModal => Span::styled(
            " PRIORITY ",
            Style::default().bg(t.mode_priority.0).fg(t.mode_priority.1),
        ),
        InputMode::TypeModal => Span::styled(
            " TYPE ",
            Style::default().bg(t.mode_type.0).fg(t.mode_type.1),
        ),
        InputMode::DeleteConfirm => Span::styled(
            " DELETE ",
            Style::default().bg(t.mode_delete.0).fg(t.mode_delete.1),
        ),
        InputMode::ParentModal => Span::styled(
            " PARENT ",
            Style::default().bg(t.mode_parent.0).fg(t.mode_parent.1),
        ),
        InputMode::BlockingModal => Span::styled(
            " BLOCKING ",
            Style::default().bg(t.mode_blocking.0).fg(t.mode_blocking.1),
        ),
        InputMode::DetailView => Span::styled(
            " DETAIL ",
            Style::default().bg(t.mode_detail.0).fg(t.mode_detail.1),
        ),
        InputMode::CreateModal => Span::styled(
            " CREATE ",
            Style::default().bg(t.mode_create.0).fg(t.mode_create.1),
        ),
    };

    let help_text = match app.input_mode {
        InputMode::Normal => {
            " ↑↓:nav  ←→:page  Space:select  /:search  c:create  s:status  e:edit  ?:help  q:quit "
        }
        InputMode::Filter => " Type to search, Enter/Esc to confirm ",
        InputMode::StatusModal
        | InputMode::PriorityModal
        | InputMode::TypeModal
        | InputMode::ParentModal => " j/k:nav  Enter:select  Esc:cancel ",
        InputMode::BlockingModal => " j/k:nav  Space:toggle  Enter:apply  Esc:cancel ",
        InputMode::DetailView => " j/k:scroll  e:edit  Esc/Enter/q:close ",
        InputMode::CreateModal => " Tab:next field  ←→:change type  Enter:create  Esc:cancel ",
        InputMode::DeleteConfirm => " y/Enter:confirm  n/Esc:cancel ",
    };

    let mut footer_spans = vec![mode_indicator];

    // Show search input when in Filter mode
    if app.input_mode == InputMode::Filter {
        footer_spans.push(Span::raw(" Search: "));
        footer_spans.push(Span::styled(
            &app.search_query,
            Style::default().fg(t.text_highlight),
        ));
        footer_spans.push(Span::styled("_", Style::default().fg(t.modal_cursor)));
        footer_spans.push(Span::raw(" "));
    }

    if let Some(ref msg) = app.message {
        footer_spans.push(Span::raw(" "));
        footer_spans.push(Span::styled(
            msg,
            Style::default().fg(t.message).add_modifier(Modifier::BOLD),
        ));
    }

    footer_spans.push(Span::styled(help_text, Style::default().fg(t.text_muted)));

    let keybindings = Paragraph::new(Line::from(footer_spans));
    f.render_widget(keybindings, area);
}

/// Get color for priority
fn priority_color(priority: &PeaPriority) -> Color {
    theme().priority_color(priority)
}

fn draw_status_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(30, 30, f.area());
    let t = theme();

    let options = App::status_options();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, status)| {
            let is_selected = idx == app.modal_selection;
            let (icon, color) = status_indicator(status);

            let selection_indicator = if is_selected {
                Span::styled("▌", Style::default().fg(t.modal_cursor))
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

fn draw_priority_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(30, 30, f.area());
    let t = theme();

    let options = App::priority_options();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, priority)| {
            let is_selected = idx == app.modal_selection;
            let color = priority_color(priority);

            let selection_indicator = if is_selected {
                Span::styled("▌", Style::default().fg(t.modal_cursor))
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

fn draw_delete_confirm(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());
    let t = theme();

    let pea_info = if let Some(pea) = app.selected_pea() {
        format!("{} - {}", pea.id, pea.title)
    } else {
        "No ticket selected".to_string()
    };

    let content = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Are you sure you want to delete this ticket?",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(pea_info, Style::default().fg(t.id))),
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

fn draw_create_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 25, f.area());
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

    let pea_type_color = type_color(&app.create_type);

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
                format!("< {} >", app.create_type),
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

fn draw_blocking_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, f.area());
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
                Span::styled("▌", Style::default().fg(t.modal_cursor))
            } else {
                Span::raw(" ")
            };

            // Checkbox
            let checkbox = if is_checked {
                Span::styled("[x] ", Style::default().fg(t.checkbox_checked))
            } else {
                Span::styled("[ ] ", Style::default().fg(t.checkbox_unchecked))
            };

            let (status_icon, status_color) = status_indicator(&pea.status);

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

fn draw_parent_modal(f: &mut Frame, app: &App) {
    // Use a larger area for parent modal since it can have many options
    let area = centered_rect(60, 50, f.area());
    let t = theme();

    // Build items: first is "(none)", then all candidates
    let mut items: Vec<ListItem> = Vec::new();

    // "(none)" option
    let is_none_selected = app.modal_selection == 0;
    let none_indicator = if is_none_selected {
        Span::styled("▌", Style::default().fg(t.modal_cursor))
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
            Span::styled("▌", Style::default().fg(t.modal_cursor))
        } else {
            Span::raw(" ")
        };

        let style = if is_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let type_col = type_color(&pea.pea_type);

        // Truncate title if too long
        let max_title_len = 35;
        let title = if pea.title.len() > max_title_len {
            format!("{}...", &pea.title[..max_title_len - 3])
        } else {
            pea.title.clone()
        };

        items.push(ListItem::new(Line::from(vec![
            selection_indicator,
            Span::styled(&pea.id, Style::default().fg(t.id)),
            Span::raw(" "),
            Span::styled(format!("[{}]", pea.pea_type), Style::default().fg(type_col)),
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

fn draw_type_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(30, 35, f.area());
    let t = theme();

    let options = App::type_options();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, pea_type)| {
            let is_selected = idx == app.modal_selection;
            let color = type_color(pea_type);

            let selection_indicator = if is_selected {
                Span::styled("▌", Style::default().fg(t.modal_cursor))
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
                Span::styled(format!("{}", pea_type), style.fg(color)),
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

fn draw_help_popup(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    let t = theme();
    let key_style = Style::default().fg(t.help_key);

    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(vec![
            Span::styled("↑/↓     ", key_style),
            Span::raw("Move up/down"),
        ]),
        Line::from(vec![
            Span::styled("←/→     ", key_style),
            Span::raw("Prev/next page"),
        ]),
        Line::from(vec![
            Span::styled("g/G     ", key_style),
            Span::raw("First/last item"),
        ]),
        Line::from(vec![
            Span::styled("Enter   ", key_style),
            Span::raw("Open detail view"),
        ]),
        Line::from(vec![
            Span::styled("/       ", key_style),
            Span::raw("Search"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(vec![
            Span::styled("c       ", key_style),
            Span::raw("Create new ticket"),
        ]),
        Line::from(vec![
            Span::styled("s       ", key_style),
            Span::raw("Change status"),
        ]),
        Line::from(vec![
            Span::styled("t       ", key_style),
            Span::raw("Change type"),
        ]),
        Line::from(vec![
            Span::styled("P       ", key_style),
            Span::raw("Change priority"),
        ]),
        Line::from(vec![
            Span::styled("p       ", key_style),
            Span::raw("Set parent"),
        ]),
        Line::from(vec![
            Span::styled("b       ", key_style),
            Span::raw("Set blocking tickets"),
        ]),
        Line::from(vec![
            Span::styled("Space   ", key_style),
            Span::raw("Toggle selection (multi-select)"),
        ]),
        Line::from(vec![
            Span::styled("e       ", key_style),
            Span::raw("Edit in $EDITOR"),
        ]),
        Line::from(vec![
            Span::styled("d       ", key_style),
            Span::raw("Delete ticket"),
        ]),
        Line::from(vec![
            Span::styled("y       ", key_style),
            Span::raw("Copy ID to clipboard"),
        ]),
        Line::from(vec![
            Span::styled("r       ", key_style),
            Span::raw("Refresh list"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("?       ", key_style),
            Span::raw("Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("Esc     ", key_style),
            Span::raw("Close / Cancel"),
        ]),
        Line::from(vec![Span::styled("q       ", key_style), Span::raw("Quit")]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(t.help_border)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(Clear, area);
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
