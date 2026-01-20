use super::app::{App, DetailPane, InputMode};
use super::theme::{theme, tui_config};
use super::ui_utils;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, Wrap,
    },
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
                draw_detail_fullscreen(f, app, chunks[0], app.detail_scroll)
            }
            super::app::ViewMode::Memory => {
                draw_memory_detail(f, app, chunks[0], app.detail_scroll)
            }
        }
        draw_footer(f, app, chunks[1]);
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
        super::app::ViewMode::Tickets => draw_tree(f, app, chunks[0]),
        super::app::ViewMode::Memory => draw_memory_list(f, app, chunks[0]),
    }
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
        InputMode::MemoryCreateModal => draw_memory_create_modal(f, app),
        InputMode::TagsModal => draw_tags_modal(f, app),
        InputMode::UrlModal => draw_url_modal(f, app),
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
    let new_page_height = page_height.max(1);

    // Rebuild page table if page_height changed
    if app.page_height != new_page_height {
        app.page_height = new_page_height;
        app.build_page_table();
    }

    // Get current page info from page table
    let current_page_num = app.current_page();
    let page_info = app.page_table.get(current_page_num).cloned();

    // Get the items for current page based on page table
    let (page_items, parent_indices, page_start) = if let Some(info) = page_info {
        let start = info.start_index;
        let end = start + info.item_count;
        let items = &app.tree_nodes[start..end.min(app.tree_nodes.len())];
        (items, info.parent_indices, start)
    } else {
        // Fallback if page table not ready
        (&app.tree_nodes[..0], Vec::new(), 0)
    };

    // Calculate index within page for highlighting
    let index_in_page = app.selected_index.saturating_sub(page_start);

    // Build parent context rows using indices from page table (Layer 2 → Layer 3)
    let mut parent_context_rows: Vec<Row> = Vec::new();
    let has_parent_context = !parent_indices.is_empty();

    if has_parent_context {
        let t = theme();
        let muted_style = Style::default().fg(t.text_muted);

        // Render each parent from the page table (already in top-down order)
        for &parent_index in &parent_indices {
            if let Some(parent_node) = app.tree_nodes.get(parent_index) {
                let pea = &parent_node.pea;
                let (status_icon, _) = ui_utils::status_indicator(&pea.status);

                // Build tree prefix using dotted lines (┊) instead of solid lines (│)
                let mut prefix = String::new();
                for &has_line in &parent_node.parent_lines {
                    if has_line {
                        prefix.push_str("┊  ");
                    } else {
                        prefix.push_str("   ");
                    }
                }
                if parent_node.depth > 0 {
                    if parent_node.is_last {
                        prefix.push_str("╰─ ");
                    } else {
                        prefix.push_str("├─ ");
                    }
                }

                // Priority indicator
                let pri = if let Some((ind, _)) = ui_utils::priority_indicator(pea) {
                    ind
                } else {
                    String::new()
                };

                // Type text
                let type_text = if tui_config().use_type_emojis {
                    format!("{} {}", theme().type_emoji(&pea.pea_type), pea.pea_type)
                } else {
                    format!("{}", pea.pea_type)
                };

                // Create tree+id cell
                let tree_and_id = Line::from(vec![
                    Span::styled(prefix, muted_style),
                    Span::styled(&pea.id, muted_style),
                ]);

                parent_context_rows.push(Row::new(vec![
                    Cell::from(""), // Selection indicator (empty for context rows)
                    Cell::from(""), // Checkbox (empty for context rows)
                    Cell::from(tree_and_id),
                    Cell::from(type_text).style(muted_style),
                    Cell::from(format!("{} {}", status_icon, pea.status)).style(muted_style),
                    Cell::from(pri).style(muted_style),
                    Cell::from(pea.title.as_str()).style(muted_style),
                ]));
            }
        }
    }

    let mut rows: Vec<Row> = parent_context_rows;
    rows.extend(page_items.iter().enumerate().map(|(idx, node)| {
        let pea = &node.pea;
        let is_selected = idx == index_in_page;
        let is_multi_selected = app.is_multi_selected(&pea.id);
        let (status_icon, status_color) = ui_utils::status_indicator(&pea.status);
        let pea_type_color = ui_utils::type_color(&pea.pea_type);

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

        // Selection indicator with pulsing effect
        let sel = if is_selected { theme().row_marker } else { " " };
        let sel_style = if is_selected {
            let elapsed_millis = app.start_time.elapsed().as_millis();
            let pulsing_color = theme().selection_indicator_pulsing_color(elapsed_millis);
            Style::default().fg(pulsing_color)
        } else {
            Style::default()
        };

        // Multi-select checkbox
        let checkbox = if is_multi_selected { "◆" } else { " " };
        let checkbox_style = Style::default().fg(theme().multi_select);

        // Priority indicator
        let pri = if let Some((ind, _)) = ui_utils::priority_indicator(pea) {
            ind
        } else {
            String::new()
        };
        let pri_color = ui_utils::priority_indicator(pea)
            .map(|(_, c)| c)
            .unwrap_or(Color::Reset);

        // Title style and highlighting
        let title_style = if is_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        // Highlight search terms in title
        let title_spans = ui_utils::highlight_search(&pea.title, &app.search_query, title_style);

        // Tree + ID combined in one cell (so tree connects to ID visually)
        // ID is bold and bright green when selected
        let id_style = theme().id_style(is_selected);

        // Highlight search terms in ID
        let id_spans = ui_utils::highlight_search(&pea.id, &app.search_query, id_style);
        let mut tree_id_spans = vec![Span::styled(
            prefix,
            Style::default().fg(theme().tree_lines),
        )];
        tree_id_spans.extend(id_spans);
        let tree_and_id = Line::from(tree_id_spans);

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
        let type_text = if tui_config().use_type_emojis {
            format!("{} {}", theme().type_emoji(&pea.pea_type), pea.pea_type)
        } else {
            format!("{}", pea.pea_type)
        };

        Row::new(vec![
            Cell::from(sel).style(sel_style),
            Cell::from(checkbox).style(checkbox_style),
            Cell::from(tree_and_id),
            Cell::from(type_text).style(type_style),
            Cell::from(format!("{} {}", status_icon, pea.status)).style(status_style),
            Cell::from(pri).style(Style::default().fg(pri_color)),
            Cell::from(Line::from(title_spans)),
        ])
    }));

    // Title shows count, selection count, and current date/time (ISO 8601)
    let selection_count = app.multi_select_count();
    let now = chrono::Local::now();
    let datetime_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let title_left = if selection_count > 0 {
        format!(
            "─🫛 peas ({}) [{} selected] ○",
            app.tree_nodes.len(),
            selection_count
        )
    } else {
        format!("─🫛 peas ({}) ○", app.tree_nodes.len())
    };

    let title_right = format!("○ {} ○─", datetime_str);

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
    // Combine left and right titles with border line spacing
    let terminal_width = area.width.saturating_sub(3); // Account for borders
    let left_len = title_left.chars().count() as u16;
    let right_len = title_right.chars().count() as u16;
    let spacing = terminal_width
        .saturating_sub(left_len)
        .saturating_sub(right_len);
    let combined_title = format!(
        "{}{}{}",
        title_left,
        "─".repeat(spacing as usize),
        title_right
    );

    let block = Block::default()
        .title(combined_title)
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
    // Adjust index to account for parent context row if present
    let adjusted_index = if has_parent_context {
        index_in_page + 1 // Offset by 1 for the parent context row
    } else {
        index_in_page
    };
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(adjusted_index));
    f.render_stateful_widget(table, table_area, &mut table_state);

    // Render page dots inside panel if needed
    if let Some(dots_area) = page_dots_area {
        let dots: Vec<Span> = (0..total_pages)
            .map(|i| {
                if i == current_page {
                    Span::styled("☍︎", Style::default().fg(theme().status_todo))
                } else {
                    Span::styled("☍︎", Style::default().fg(theme().text_muted))
                }
            })
            .collect();
        let dots_line = Line::from(dots);
        let dots_paragraph = Paragraph::new(dots_line);
        f.render_widget(dots_paragraph, dots_area);
    }
}

/// Get color for type (without the indicator character)

fn draw_memory_list(f: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::{
        layout::Constraint,
        style::{Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Cell, Row, Table, TableState},
    };

    let t = theme();
    let title = format!("─🧠 Memory ({}) ○", app.filtered_memories.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(t.border));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let rows: Vec<Row> = app
        .filtered_memories
        .iter()
        .enumerate()
        .map(|(idx, memory)| {
            let is_selected = idx == app.selected_index;

            // Pulsing row marker (column 1)
            let marker = if is_selected { t.row_marker } else { " " };
            let marker_style = if is_selected {
                let elapsed_millis = app.start_time.elapsed().as_millis();
                let pulsing_color = t.selection_indicator_pulsing_color(elapsed_millis);
                Style::default().fg(pulsing_color)
            } else {
                Style::default()
            };
            let marker_cell = Cell::from(Span::styled(marker, marker_style));

            // Memory key styled as ID (column 2)
            let key_style = if is_selected {
                Style::default()
                    .fg(t.id_selected)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(t.id)
            };
            let key_cell = Cell::from(Span::styled(&memory.key, key_style));

            // Tags (column 3)
            let mut tag_spans = vec![];
            if !memory.tags.is_empty() {
                for (i, tag) in memory.tags.iter().enumerate() {
                    if i > 0 {
                        tag_spans.push(Span::raw(" "));
                    }
                    tag_spans.push(Span::styled(
                        format!("#{}", tag),
                        Style::default().fg(t.tags),
                    ));
                }
            }
            let tags_cell = Cell::from(Line::from(tag_spans));

            // Timestamp (column 4)
            let time_str = memory.updated.format("%Y-%m-%d %H:%M").to_string();
            let time_cell = Cell::from(Span::styled(time_str, Style::default().fg(t.text_muted)));

            Row::new(vec![marker_cell, key_cell, tags_cell, time_cell])
        })
        .collect();

    let widths = [
        Constraint::Length(2),      // Marker
        Constraint::Percentage(40), // Key
        Constraint::Percentage(40), // Tags
        Constraint::Percentage(20), // Timestamp
    ];

    let table = Table::new(rows, widths)
        .column_spacing(1)
        .row_highlight_style(Style::default());

    // Use a fresh table state for selection
    let mut table_state = TableState::default();
    table_state.select(Some(app.selected_index));

    f.render_stateful_widget(table, inner_area, &mut table_state);
}

fn draw_memory_detail(f: &mut Frame, app: &mut App, area: Rect, detail_scroll: u16) {
    use ratatui::{
        layout::{Constraint, Layout},
        style::{Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph, Wrap},
    };

    if app.selected_index >= app.filtered_memories.len() {
        return;
    }

    let t = theme();
    let memory = &app.filtered_memories[app.selected_index];

    // Title with memory key styled like an ID
    let title = format!(" {} ", memory.key);
    let detail_block = Block::default()
        .title(title)
        .title_style(
            Style::default()
                .fg(t.id_selected)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(t.border_focused));

    let inner = detail_block.inner(area);
    f.render_widget(detail_block, area);

    // Split into metadata section and content section
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Metadata (padding + tags + timestamps)
            Constraint::Min(0),    // Content
        ])
        .split(inner);

    // Metadata section with improved styling and padding
    let mut metadata_lines = vec![];

    // Top padding line
    metadata_lines.push(Line::from(""));

    // Tags line
    if !memory.tags.is_empty() {
        let mut tag_spans = vec![Span::raw("  ")]; // Left padding
        for (i, tag) in memory.tags.iter().enumerate() {
            if i > 0 {
                tag_spans.push(Span::raw("  "));
            }
            tag_spans.push(Span::styled(
                format!("#{}", tag),
                Style::default().fg(t.tags),
            ));
        }
        metadata_lines.push(Line::from(tag_spans));
    }

    // Timestamps line
    let time_line = Line::from(vec![
        Span::raw("  "), // Left padding
        Span::styled("Created ", Style::default().fg(t.text_muted)),
        Span::styled(
            memory.created.format("%Y-%m-%d %H:%M").to_string(),
            Style::default().fg(t.timestamp),
        ),
        Span::styled("  •  ", Style::default().fg(t.text_muted)),
        Span::styled("Updated ", Style::default().fg(t.text_muted)),
        Span::styled(
            memory.updated.format("%Y-%m-%d %H:%M").to_string(),
            Style::default().fg(t.timestamp),
        ),
    ]);
    metadata_lines.push(time_line);

    // Empty separator line
    metadata_lines.push(Line::from(""));

    let metadata = Paragraph::new(metadata_lines);
    f.render_widget(metadata, chunks[0]);

    // Content section with proper styling and padding
    let content_lines: Vec<Line> = memory
        .content
        .lines()
        .map(|line| {
            // Add left padding to each line
            Line::from(vec![
                Span::raw("  "),
                Span::styled(line, Style::default().fg(t.text)),
            ])
        })
        .collect();

    let content_height = chunks[1].height as usize;
    let total_lines = content_lines.len();

    // Update max scroll
    let _max_scroll = total_lines.saturating_sub(content_height) as u16;
    // Note: We can't update app here, so scrolling logic needs to be handled in event loop

    let content = Paragraph::new(content_lines)
        .scroll((detail_scroll, 0))
        .wrap(Wrap { trim: false });

    f.render_widget(content, chunks[1]);
}

fn draw_detail_fullscreen(f: &mut Frame, app: &mut App, area: Rect, detail_scroll: u16) {
    let detail_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme().border_focused));

    if let Some(pea) = app.selected_pea().cloned() {
        let status_color = theme().status_color(&pea.status);
        let pea_priority_color = ui_utils::priority_color(&pea.priority);

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

        // Build metadata as table with proper column alignment
        let t = theme();
        let is_metadata_focused = app.detail_pane == DetailPane::Metadata;

        // Helper to create row marker for editable rows
        let row_marker = |index: usize| -> String {
            if is_metadata_focused && app.metadata_selection == index {
                theme().row_marker.to_string()
            } else {
                " ".to_string()
            }
        };

        // Compute pulsing style for row markers
        let elapsed_millis = app.start_time.elapsed().as_millis();
        let pulsing_color = theme().selection_indicator_pulsing_color(elapsed_millis);
        let pulsing_style = Style::default().fg(pulsing_color);

        // Build property values
        let type_text = if tui_config().use_type_emojis {
            format!("{} {}", theme().type_emoji(&pea.pea_type), pea.pea_type)
        } else {
            format!("{}", pea.pea_type)
        };

        let tags_display = if pea.tags.is_empty() {
            "(none)".to_string()
        } else {
            pea.tags.join(", ")
        };

        // Build table rows
        let metadata_rows = vec![
            // Title row
            Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from(Span::styled(
                    format!("{} {}", pea.id, pea.title),
                    Style::default().fg(t.id).add_modifier(Modifier::BOLD),
                )),
            ]),
            // Empty row
            Row::new(vec![Cell::from(""), Cell::from(""), Cell::from("")]),
            // Type
            Row::new(vec![
                Cell::from(Span::styled(row_marker(0), pulsing_style)),
                Cell::from("Type:"),
                Cell::from(Span::styled(
                    type_text,
                    Style::default().fg(ui_utils::type_color(&pea.pea_type)),
                )),
            ]),
            // Status
            Row::new(vec![
                Cell::from(Span::styled(row_marker(1), pulsing_style)),
                Cell::from("Status:"),
                Cell::from(Span::styled(
                    format!("{}", pea.status),
                    Style::default().fg(status_color),
                )),
            ]),
            // Priority
            Row::new(vec![
                Cell::from(Span::styled(row_marker(2), pulsing_style)),
                Cell::from("Priority:"),
                Cell::from(Span::styled(
                    format!("{}", pea.priority),
                    Style::default().fg(pea_priority_color),
                )),
            ]),
            // Tags
            Row::new(vec![
                Cell::from(Span::styled(row_marker(3), pulsing_style)),
                Cell::from("Tags:"),
                Cell::from(Span::styled(
                    tags_display,
                    Style::default().fg(theme().tags),
                )),
            ]),
            // Empty row
            Row::new(vec![Cell::from(""), Cell::from(""), Cell::from("")]),
            // Created
            Row::new(vec![
                Cell::from(""),
                Cell::from("Created:"),
                Cell::from(Span::styled(
                    pea.created.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(theme().timestamp),
                )),
            ]),
            // Updated
            Row::new(vec![
                Cell::from(""),
                Cell::from("Updated:"),
                Cell::from(Span::styled(
                    pea.updated.format("%Y-%m-%d %H:%M").to_string(),
                    Style::default().fg(theme().timestamp),
                )),
            ]),
        ];

        let widths = [
            Constraint::Length(2),  // Row marker column
            Constraint::Length(10), // Label column
            Constraint::Min(0),     // Value column
        ];

        // Render metadata section as table
        let metadata_block = Block::default()
            .title(format!(" {} ", pea.id))
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(theme().border_style(is_metadata_focused));

        let metadata_table = Table::new(metadata_rows, widths)
            .block(metadata_block)
            .column_spacing(1);

        f.render_widget(metadata_table, metadata_area);

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
                .map(|(i, (rel_type, id, title, pea_type))| {
                    let is_selected = i == app.relations_selection;
                    let prefix = super::theme::Theme::relation_prefix(rel_type);
                    let rel_color = theme().relation_color(rel_type);
                    let type_color = ui_utils::type_color(pea_type);

                    // Selection cursor with pulsing effect (only show when pane is focused)
                    let cursor = if is_selected && is_focused {
                        Span::styled(format!("{} ", theme().row_marker), pulsing_style)
                    } else {
                        Span::raw("  ")
                    };

                    let type_text = if tui_config().use_type_emojis {
                        format!("{} {} ", theme().type_emoji(pea_type), pea_type)
                    } else {
                        format!("{} ", pea_type)
                    };

                    let content = Line::from(vec![
                        cursor,
                        Span::styled(format!("{} ", prefix), Style::default().fg(rel_color)),
                        Span::styled(format!("[{}] ", rel_type), Style::default().fg(rel_color)),
                        Span::styled(type_text, Style::default().fg(type_color)),
                        Span::styled(id, Style::default().fg(theme().id)),
                        Span::raw(" "),
                        Span::styled(
                            if title.len() > 20 {
                                format!("{}...", &title[..17])
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

        // Render body section with tui-markdown or TextArea (if editing)
        if let Some(body_rect) = body_area {
            let body_focused = app.detail_pane == DetailPane::Body;

            let title = if app.input_mode == InputMode::EditBody {
                " Description [EDITING - Ctrl+S to save, Esc to cancel] "
            } else {
                " Description "
            };

            let body_block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(if app.input_mode == InputMode::EditBody {
                    Style::default().fg(theme().text_highlight) // Yellow/bright to indicate edit mode
                } else {
                    theme().border_style(body_focused)
                });

            let inner = body_block.inner(body_rect);
            f.render_widget(body_block, body_rect);

            // Render textarea if in edit mode, otherwise render markdown
            if app.input_mode == InputMode::EditBody {
                if let Some(textarea) = app.body_textarea.as_ref() {
                    f.render_widget(textarea, inner);
                }
                // No scrolling in edit mode (textarea handles its own scrolling)
                app.set_detail_max_scroll(0);
            } else {
                // Render markdown using tui-markdown
                let md_text_core = tui_markdown::from_str(&body_content);

                // Convert from ratatui_core::Text to ratatui::Text by extracting lines
                let lines: Vec<Line> = md_text_core
                    .lines
                    .into_iter()
                    .map(|line_core| {
                        let spans: Vec<Span> = line_core
                            .spans
                            .into_iter()
                            .map(|span_core| {
                                Span::styled(
                                    span_core.content,
                                    ui_utils::convert_style(span_core.style),
                                )
                            })
                            .collect();
                        Line::from(spans)
                    })
                    .collect();
                let md_text = Text::from(lines);

                // Calculate content height for scroll limiting
                let view_height = inner.height as u16;
                let content_lines =
                    ui_utils::estimate_wrapped_lines(&md_text, inner.width as usize);
                let max_scroll = content_lines.saturating_sub(view_height);
                app.set_detail_max_scroll(max_scroll);

                let md_paragraph = Paragraph::new(md_text)
                    .wrap(Wrap { trim: false })
                    .scroll((detail_scroll, 0));
                f.render_widget(md_paragraph, inner);

                // Render scrollbar if content is scrollable
                if max_scroll > 0 {
                    let mut scrollbar_state =
                        ScrollbarState::new(max_scroll as usize).position(detail_scroll as usize);

                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("↑"))
                        .end_symbol(Some("↓"))
                        .track_symbol(Some("│"))
                        .thumb_symbol("█")
                        .style(Style::default().fg(theme().border));

                    f.render_stateful_widget(scrollbar, body_rect, &mut scrollbar_state);
                }
            }
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
        InputMode::MemoryCreateModal => Span::styled(
            " CREATE MEMORY ",
            Style::default().bg(t.mode_create.0).fg(t.mode_create.1),
        ),
        InputMode::EditBody => Span::styled(
            " EDIT ",
            Style::default().bg(t.text_highlight).fg(Color::Black),
        ),
        InputMode::TagsModal => Span::styled(
            " TAGS ",
            Style::default().bg(t.mode_parent.0).fg(t.mode_parent.1),
        ),
        InputMode::UrlModal => Span::styled(
            " URL ",
            Style::default().bg(t.mode_parent.0).fg(t.mode_parent.1),
        ),
    };

    let help_text = match app.input_mode {
        InputMode::Normal => match app.view_mode {
            super::app::ViewMode::Tickets => {
                " ↑↓:nav  ←→:page  Space:select  /:search  Tab:memory  c:create  s:status  e:edit  ?:help  q:quit "
            }
            super::app::ViewMode::Memory => " ↑↓:nav  Tab:tickets  n:new  ?:help  q:quit ",
        },
        InputMode::Filter => " Type to search, Enter/Esc to confirm ",
        InputMode::StatusModal
        | InputMode::PriorityModal
        | InputMode::TypeModal
        | InputMode::ParentModal => " ↓/↑:nav  Enter:select  Esc:cancel ",
        InputMode::BlockingModal => " ↓/↑:nav  Space:toggle  Enter:apply  Esc:cancel ",
        InputMode::DetailView => match app.view_mode {
            super::app::ViewMode::Tickets => {
                " ↓/↑:scroll  e:edit  o:open-url  s:status  P:priority  t:type  p:parent  b:blocking  y:copy-id  Esc/q:close "
            }
            super::app::ViewMode::Memory => " ↓/↑:scroll  Esc/q:close ",
        },
        InputMode::CreateModal => " Tab:next field  ←→:change type  Enter:create  Esc:cancel ",
        InputMode::MemoryCreateModal => " Tab:next field  Enter:create  Esc:cancel ",
        InputMode::DeleteConfirm => " y/Enter:confirm  n/Esc:cancel ",
        InputMode::EditBody => " Ctrl+S:save  Esc:cancel ",
        InputMode::TagsModal => " Type comma-separated tags  Enter:save  Esc:cancel ",
        InputMode::UrlModal => " ↓/↑:navigate  Enter:open  Esc:cancel ",
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

fn draw_status_modal(f: &mut Frame, app: &App) {
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

fn draw_priority_modal(f: &mut Frame, app: &App) {
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

fn draw_delete_confirm(f: &mut Frame, app: &App) {
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

fn draw_tags_modal(f: &mut Frame, app: &App) {
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

fn draw_url_modal(f: &mut Frame, app: &App) {
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

fn draw_create_modal(f: &mut Frame, app: &App) {
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

fn draw_memory_create_modal(f: &mut Frame, app: &App) {
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

fn draw_blocking_modal(f: &mut Frame, app: &App) {
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

fn draw_parent_modal(f: &mut Frame, app: &App) {
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

fn draw_type_modal(f: &mut Frame, app: &App) {
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

fn draw_help_popup(f: &mut Frame) {
    let area = ui_utils::centered_rect(60, 70, f.area());
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
        Line::from(vec![
            Span::styled("Tab     ", key_style),
            Span::raw("Switch between Tickets/Memory"),
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
