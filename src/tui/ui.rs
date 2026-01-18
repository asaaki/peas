use super::app::{App, InputMode};
use crate::model::{Pea, PeaPriority, PeaStatus, PeaType};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
};

/// Returns priority indicator and color for a pea
fn priority_indicator(pea: &Pea) -> Option<(String, Color)> {
    match pea.priority {
        PeaPriority::Critical => Some(("‼".to_string(), Color::Red)), // U+203C double exclamation
        PeaPriority::High => Some(("!".to_string(), Color::LightRed)),
        PeaPriority::Normal => None, // No indicator for normal
        PeaPriority::Low => Some(("↓".to_string(), Color::DarkGray)),
        PeaPriority::Deferred => Some(("⏸".to_string(), Color::DarkGray)),
    }
}

/// Returns status icon and color
fn status_indicator(status: &PeaStatus) -> (&'static str, Color) {
    match status {
        PeaStatus::Draft => ("○", Color::DarkGray),
        PeaStatus::Todo => ("○", Color::Green), // Green for open work
        PeaStatus::InProgress => ("◐", Color::Yellow),
        PeaStatus::Completed => ("●", Color::DarkGray), // Gray for completed (de-emphasized)
        PeaStatus::Scrapped => ("✗", Color::DarkGray),
    }
}

/// Render a markdown line with basic formatting
fn render_markdown_line(line: &str) -> Line<'static> {
    let trimmed = line.trim_start();

    // Headers - keep the # characters for visual distinction
    if trimmed.starts_with("### ") {
        return Line::from(Span::styled(
            trimmed.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("## ") {
        return Line::from(Span::styled(
            trimmed.to_string(),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("# ") {
        return Line::from(Span::styled(
            trimmed.to_string(),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Checkbox items
    if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
        let indent = line.len() - trimmed.len();
        return Line::from(vec![
            Span::raw(" ".repeat(indent)),
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::raw(trimmed[6..].to_string()),
        ]);
    }
    if trimmed.starts_with("- [ ] ") {
        let indent = line.len() - trimmed.len();
        return Line::from(vec![
            Span::raw(" ".repeat(indent)),
            Span::styled("○ ", Style::default().fg(Color::DarkGray)),
            Span::raw(trimmed[6..].to_string()),
        ]);
    }

    // Bullet points
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
        let indent = line.len() - trimmed.len();
        return Line::from(vec![
            Span::raw(" ".repeat(indent)),
            Span::styled("• ", Style::default().fg(Color::Cyan)),
            Span::raw(trimmed[2..].to_string()),
        ]);
    }

    // Numbered lists
    if let Some(rest) = trimmed
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .len()
        .checked_sub(0)
    {
        if rest > 0 && trimmed.chars().nth(rest) == Some('.') {
            let indent = line.len() - trimmed.len();
            let num_end = rest + 1; // include the dot
            if trimmed.len() > num_end && trimmed.chars().nth(num_end) == Some(' ') {
                return Line::from(vec![
                    Span::raw(" ".repeat(indent)),
                    Span::styled(
                        trimmed[..num_end].to_string(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(trimmed[num_end..].to_string()),
                ]);
            }
        }
    }

    // Code blocks (indented by 4+ spaces or starting with ```)
    if trimmed.starts_with("```") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    }
    if line.starts_with("    ") || line.starts_with("\t") {
        return Line::from(Span::styled(
            line.to_string(),
            Style::default().fg(Color::Yellow),
        ));
    }

    // Blockquotes
    if trimmed.starts_with("> ") {
        let indent = line.len() - trimmed.len();
        return Line::from(vec![
            Span::raw(" ".repeat(indent)),
            Span::styled("│ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                trimmed[2..].to_string(),
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]);
    }

    // Default: render inline formatting
    render_inline_markdown(line)
}

/// Render inline markdown formatting (bold, italic, code)
fn render_inline_markdown(line: &str) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Inline code
        if chars[i] == '`' {
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == '`') {
                let code: String = chars[i + 1..i + 1 + end].iter().collect();
                spans.push(Span::styled(code, Style::default().fg(Color::Yellow)));
                i += end + 2;
                continue;
            }
        }

        // Bold (**text**)
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_closing(&chars[i + 2..], "**") {
                let text: String = chars[i + 2..i + 2 + end].iter().collect();
                spans.push(Span::styled(
                    text,
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                i += end + 4;
                continue;
            }
        }

        // Italic (*text* or _text_)
        if chars[i] == '*' || chars[i] == '_' {
            let marker = chars[i];
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == marker) {
                if end > 0 {
                    let text: String = chars[i + 1..i + 1 + end].iter().collect();
                    spans.push(Span::styled(
                        text,
                        Style::default().add_modifier(Modifier::ITALIC),
                    ));
                    i += end + 2;
                    continue;
                }
            }
        }

        // Regular character
        spans.push(Span::raw(chars[i].to_string()));
        i += 1;
    }

    Line::from(spans)
}

/// Find closing marker in a char slice
fn find_closing(chars: &[char], marker: &str) -> Option<usize> {
    let marker_chars: Vec<char> = marker.chars().collect();
    for i in 0..chars.len().saturating_sub(marker_chars.len() - 1) {
        if chars[i..].starts_with(&marker_chars) {
            return Some(i);
        }
    }
    None
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
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::SLOW_BLINK)
            } else {
                Style::default()
            };

            // Multi-select checkbox
            let checkbox = if is_multi_selected { "◆" } else { " " };
            let checkbox_style = Style::default().fg(Color::Cyan);

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
            let id_style = if is_selected {
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };
            let tree_and_id = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
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
        .border_style(Style::default().fg(Color::Gray));
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
                    Span::styled("•", Style::default().fg(Color::White))
                } else {
                    Span::styled("•", Style::default().fg(Color::DarkGray))
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
    match pea_type {
        PeaType::Milestone => Color::Magenta,
        PeaType::Epic => Color::Blue,
        PeaType::Story => Color::Cyan,
        PeaType::Feature => Color::Cyan,
        PeaType::Bug => Color::Red,
        PeaType::Chore => Color::Yellow,
        PeaType::Research => Color::LightMagenta,
        PeaType::Task => Color::White,
    }
}

fn draw_detail_fullscreen(f: &mut Frame, app: &App, area: Rect, detail_scroll: u16) {
    let detail_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Green));

    if let Some(pea) = app.selected_pea() {
        let status_color = match pea.status {
            PeaStatus::Draft => Color::DarkGray,
            PeaStatus::Todo => Color::White,
            PeaStatus::InProgress => Color::Yellow,
            PeaStatus::Completed => Color::Green,
            PeaStatus::Scrapped => Color::Red,
        };

        let priority_color = match pea.priority {
            PeaPriority::Critical => Color::Red,
            PeaPriority::High => Color::LightRed,
            PeaPriority::Normal => Color::White,
            PeaPriority::Low => Color::DarkGray,
            PeaPriority::Deferred => Color::DarkGray,
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled(
                    &pea.id,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(&pea.title, Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Type:     "),
                Span::styled(
                    format!("{}", pea.pea_type),
                    Style::default().fg(Color::Blue),
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
                    Style::default().fg(priority_color),
                ),
            ]),
        ];

        if let Some(ref parent) = pea.parent {
            let parent_title = app
                .all_peas
                .iter()
                .find(|p| p.id == *parent)
                .map(|p| p.title.as_str())
                .unwrap_or("");
            lines.push(Line::from(vec![
                Span::raw("Parent:   "),
                Span::styled(parent, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(parent_title, Style::default().fg(Color::DarkGray)),
            ]));
        }

        // Show blocking tickets
        if !pea.blocking.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("Blocking: "),
                Span::styled(
                    format!("{} tickets", pea.blocking.len()),
                    Style::default().fg(Color::LightRed),
                ),
            ]));
            for id in pea.blocking.iter().take(5) {
                let title = app
                    .all_peas
                    .iter()
                    .find(|p| &p.id == id)
                    .map(|p| format!("{} ({})", id, p.title))
                    .unwrap_or_else(|| id.clone());
                lines.push(Line::from(vec![
                    Span::raw("          "),
                    Span::styled(title, Style::default().fg(Color::DarkGray)),
                ]));
            }
            if pea.blocking.len() > 5 {
                lines.push(Line::from(Span::styled(
                    format!("          ... and {} more", pea.blocking.len() - 5),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        // Find children
        let children: Vec<_> = app
            .all_peas
            .iter()
            .filter(|p| p.parent.as_ref() == Some(&pea.id))
            .collect();
        if !children.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("Children: "),
                Span::styled(
                    format!("{} items", children.len()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        if !pea.tags.is_empty() {
            lines.push(Line::from(vec![
                Span::raw("Tags:     "),
                Span::styled(pea.tags.join(", "), Style::default().fg(Color::Magenta)),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("Created:  "),
            Span::styled(
                pea.created.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("Updated:  "),
            Span::styled(
                pea.updated.format("%Y-%m-%d %H:%M").to_string(),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        // List children if any
        if !children.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Children:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            for child in children.iter().take(15) {
                let (status_icon, status_color) = status_indicator(&child.status);
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {} ", status_icon),
                        Style::default().fg(status_color),
                    ),
                    Span::styled(&child.id, Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::raw(&child.title),
                ]));
            }
            if children.len() > 15 {
                lines.push(Line::from(Span::styled(
                    format!("  ... and {} more", children.len() - 15),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        if !pea.body.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Description:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            for line in pea.body.lines() {
                lines.push(render_markdown_line(line));
            }
        }

        // Add scroll hint
        let content_height = lines.len() as u16;
        let visible_height = area.height.saturating_sub(2);
        let scroll_hint = if content_height > visible_height {
            format!(
                " [↑↓ scroll: {}/{}] ",
                detail_scroll + 1,
                content_height.saturating_sub(visible_height) + 1
            )
        } else {
            String::new()
        };

        let detail_block = Block::default()
            .title(format!(" {} ", pea.id))
            .title_bottom(Line::from(scroll_hint).right_aligned())
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Green));

        let detail = Paragraph::new(Text::from(lines))
            .block(detail_block)
            .wrap(Wrap { trim: true })
            .scroll((detail_scroll, 0));

        f.render_widget(detail, area);
    } else {
        let empty = Paragraph::new("No pea selected")
            .block(detail_block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, area);
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    // Mode indicator
    let mode_indicator = match app.input_mode {
        InputMode::Normal => Span::styled(
            " NORMAL ",
            Style::default().bg(Color::Blue).fg(Color::White),
        ),
        InputMode::Filter => Span::styled(
            " SEARCH ",
            Style::default().bg(Color::Yellow).fg(Color::Black),
        ),
        InputMode::StatusModal => Span::styled(
            " STATUS ",
            Style::default().bg(Color::Green).fg(Color::Black),
        ),
        InputMode::PriorityModal => Span::styled(
            " PRIORITY ",
            Style::default().bg(Color::Red).fg(Color::White),
        ),
        InputMode::TypeModal => Span::styled(
            " TYPE ",
            Style::default().bg(Color::Magenta).fg(Color::White),
        ),
        InputMode::DeleteConfirm => {
            Span::styled(" DELETE ", Style::default().bg(Color::Red).fg(Color::White))
        }
        InputMode::ParentModal => Span::styled(
            " PARENT ",
            Style::default().bg(Color::Blue).fg(Color::White),
        ),
        InputMode::BlockingModal => Span::styled(
            " BLOCKING ",
            Style::default().bg(Color::LightRed).fg(Color::Black),
        ),
        InputMode::DetailView => Span::styled(
            " DETAIL ",
            Style::default().bg(Color::Green).fg(Color::Black),
        ),
        InputMode::CreateModal => Span::styled(
            " CREATE ",
            Style::default().bg(Color::Cyan).fg(Color::Black),
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

    if let Some(ref msg) = app.message {
        footer_spans.push(Span::raw(" "));
        footer_spans.push(Span::styled(
            msg,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));
    }

    footer_spans.push(Span::styled(
        help_text,
        Style::default().fg(Color::DarkGray),
    ));

    let keybindings = Paragraph::new(Line::from(footer_spans));
    f.render_widget(keybindings, area);
}

/// Get color for priority
fn priority_color(priority: &PeaPriority) -> Color {
    match priority {
        PeaPriority::Critical => Color::Red,
        PeaPriority::High => Color::LightRed,
        PeaPriority::Normal => Color::White,
        PeaPriority::Low => Color::DarkGray,
        PeaPriority::Deferred => Color::DarkGray,
    }
}

fn draw_status_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(30, 30, f.area());

    let options = App::status_options();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, status)| {
            let is_selected = idx == app.modal_selection;
            let (icon, color) = status_indicator(status);

            let selection_indicator = if is_selected {
                Span::styled("▌", Style::default().fg(Color::Cyan))
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
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

fn draw_priority_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(30, 30, f.area());

    let options = App::priority_options();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, priority)| {
            let is_selected = idx == app.modal_selection;
            let color = priority_color(priority);

            let selection_indicator = if is_selected {
                Span::styled("▌", Style::default().fg(Color::Cyan))
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
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

fn draw_delete_confirm(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, f.area());

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
        Line::from(Span::styled(pea_info, Style::default().fg(Color::Cyan))),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "y",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/Enter = Yes    "),
            Span::styled(
                "n",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
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
                .border_style(Style::default().fg(Color::Red)),
        )
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn draw_create_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 25, f.area());

    let title_active = app.modal_selection == 0;
    let type_active = app.modal_selection == 1;

    // Build display text for title field
    let title_display = if app.create_title.is_empty() {
        Span::styled("Enter title...", Style::default().fg(Color::DarkGray))
    } else {
        Span::raw(app.create_title.clone())
    };

    let title_style = if title_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };

    let type_style = if type_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };

    let type_color = type_color(&app.create_type);

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if title_active { "▶ " } else { "  " },
                Style::default().fg(Color::Cyan),
            ),
            Span::styled("Title: ", title_style.add_modifier(Modifier::BOLD)),
            title_display,
            if title_active {
                Span::styled("_", Style::default().fg(Color::Cyan))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if type_active { "▶ " } else { "  " },
                Style::default().fg(Color::Cyan),
            ),
            Span::styled("Type:  ", type_style.add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("< {} >", app.create_type),
                Style::default().fg(type_color),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  (use ←/→ to change type)",
            Style::default().fg(Color::DarkGray),
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
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(all_content).block(
        Block::default()
            .title(" Create Ticket ")
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn draw_blocking_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, f.area());

    let items: Vec<ListItem> = app
        .blocking_candidates
        .iter()
        .zip(app.blocking_selected.iter())
        .enumerate()
        .map(|(idx, (pea, &is_checked))| {
            let is_cursor = idx == app.modal_selection;

            // Cursor indicator
            let cursor = if is_cursor {
                Span::styled("▌", Style::default().fg(Color::Cyan))
            } else {
                Span::raw(" ")
            };

            // Checkbox
            let checkbox = if is_checked {
                Span::styled("[x] ", Style::default().fg(Color::Green))
            } else {
                Span::styled("[ ] ", Style::default().fg(Color::DarkGray))
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
                Span::styled(&pea.id, Style::default().fg(Color::Cyan)),
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
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

fn draw_parent_modal(f: &mut Frame, app: &App) {
    // Use a larger area for parent modal since it can have many options
    let area = centered_rect(60, 50, f.area());

    // Build items: first is "(none)", then all candidates
    let mut items: Vec<ListItem> = Vec::new();

    // "(none)" option
    let is_none_selected = app.modal_selection == 0;
    let none_indicator = if is_none_selected {
        Span::styled("▌", Style::default().fg(Color::Cyan))
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
        Span::styled("(none)", none_style.fg(Color::DarkGray)),
    ])));

    // Candidate options
    for (idx, pea) in app.parent_candidates.iter().enumerate() {
        let is_selected = app.modal_selection == idx + 1;
        let selection_indicator = if is_selected {
            Span::styled("▌", Style::default().fg(Color::Cyan))
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
            Span::styled(&pea.id, Style::default().fg(Color::Cyan)),
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
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

fn draw_type_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(30, 35, f.area());

    let options = App::type_options();
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(idx, pea_type)| {
            let is_selected = idx == app.modal_selection;
            let color = type_color(pea_type);

            let selection_indicator = if is_selected {
                Span::styled("▌", Style::default().fg(Color::Cyan))
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
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

fn draw_help_popup(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());

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
            Span::styled("↑/↓     ", Style::default().fg(Color::Cyan)),
            Span::raw("Move up/down"),
        ]),
        Line::from(vec![
            Span::styled("←/→     ", Style::default().fg(Color::Cyan)),
            Span::raw("Prev/next page"),
        ]),
        Line::from(vec![
            Span::styled("g/G     ", Style::default().fg(Color::Cyan)),
            Span::raw("First/last item"),
        ]),
        Line::from(vec![
            Span::styled("Enter   ", Style::default().fg(Color::Cyan)),
            Span::raw("Open detail view"),
        ]),
        Line::from(vec![
            Span::styled("/       ", Style::default().fg(Color::Cyan)),
            Span::raw("Search"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(vec![
            Span::styled("c       ", Style::default().fg(Color::Cyan)),
            Span::raw("Create new ticket"),
        ]),
        Line::from(vec![
            Span::styled("s       ", Style::default().fg(Color::Cyan)),
            Span::raw("Change status"),
        ]),
        Line::from(vec![
            Span::styled("t       ", Style::default().fg(Color::Cyan)),
            Span::raw("Change type"),
        ]),
        Line::from(vec![
            Span::styled("P       ", Style::default().fg(Color::Cyan)),
            Span::raw("Change priority"),
        ]),
        Line::from(vec![
            Span::styled("p       ", Style::default().fg(Color::Cyan)),
            Span::raw("Set parent"),
        ]),
        Line::from(vec![
            Span::styled("b       ", Style::default().fg(Color::Cyan)),
            Span::raw("Set blocking tickets"),
        ]),
        Line::from(vec![
            Span::styled("Space   ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle selection (multi-select)"),
        ]),
        Line::from(vec![
            Span::styled("e       ", Style::default().fg(Color::Cyan)),
            Span::raw("Edit in $EDITOR"),
        ]),
        Line::from(vec![
            Span::styled("d       ", Style::default().fg(Color::Cyan)),
            Span::raw("Delete ticket"),
        ]),
        Line::from(vec![
            Span::styled("y       ", Style::default().fg(Color::Cyan)),
            Span::raw("Copy ID to clipboard"),
        ]),
        Line::from(vec![
            Span::styled("r       ", Style::default().fg(Color::Cyan)),
            Span::raw("Refresh list"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("?       ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("Esc     ", Style::default().fg(Color::Cyan)),
            Span::raw("Close / Cancel"),
        ]),
        Line::from(vec![
            Span::styled("q       ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Yellow)),
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
