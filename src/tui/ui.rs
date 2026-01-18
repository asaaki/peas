use super::app::{App, FilterType, InputMode, ViewMode};
use crate::model::{Pea, PeaPriority, PeaStatus, PeaType};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

/// Returns priority indicator and color for a pea
fn priority_indicator(pea: &Pea) -> Option<(String, Color)> {
    match pea.priority {
        PeaPriority::Critical => Some(("!!".to_string(), Color::Red)),
        PeaPriority::High => Some(("!".to_string(), Color::LightRed)),
        PeaPriority::Normal => None, // No indicator for normal
        PeaPriority::Low => Some(("↓".to_string(), Color::DarkGray)),
        PeaPriority::Deferred => Some(("⏸".to_string(), Color::DarkGray)),
    }
}

/// Returns type indicator character and color
fn type_indicator(pea_type: &PeaType) -> (&'static str, Color) {
    match pea_type {
        PeaType::Milestone => ("M", Color::Magenta),
        PeaType::Epic => ("E", Color::Blue),
        PeaType::Story => ("S", Color::Cyan),
        PeaType::Feature => ("F", Color::Cyan),
        PeaType::Bug => ("B", Color::Red),
        PeaType::Chore => ("C", Color::Yellow),
        PeaType::Research => ("R", Color::LightMagenta),
        PeaType::Task => ("T", Color::White),
    }
}

/// Returns status icon and color
fn status_indicator(status: &PeaStatus) -> (&'static str, Color) {
    match status {
        PeaStatus::Draft => ("○", Color::DarkGray),
        PeaStatus::Todo => ("○", Color::White),
        PeaStatus::InProgress => ("◐", Color::Yellow),
        PeaStatus::Completed => ("●", Color::Green),
        PeaStatus::Scrapped => ("✗", Color::Red),
    }
}

/// Render a markdown line with basic formatting
fn render_markdown_line(line: &str) -> Line<'static> {
    let trimmed = line.trim_start();

    // Headers
    if trimmed.starts_with("### ") {
        return Line::from(Span::styled(
            trimmed[4..].to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("## ") {
        return Line::from(Span::styled(
            trimmed[3..].to_string(),
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if trimmed.starts_with("# ") {
        return Line::from(Span::styled(
            trimmed[2..].to_string(),
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header (smaller now)
            Constraint::Min(0),    // Main content
            Constraint::Length(4), // Footer (2 rows: filters + keybindings)
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    if app.show_help {
        draw_help_popup(f);
    }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let mut header_spans = vec![Span::styled(
        " peas ",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )];

    // Show view mode indicator
    let view_mode_label = match app.view_mode {
        ViewMode::List => "List",
        ViewMode::Tree => "Tree",
    };
    header_spans.push(Span::raw(" | "));
    header_spans.push(Span::styled(
        format!(" {} ", view_mode_label),
        Style::default().fg(Color::Cyan),
    ));

    if !app.search_query.is_empty() {
        header_spans.push(Span::raw(" | "));
        header_spans.push(Span::styled(
            format!("/{}", app.search_query),
            Style::default().fg(Color::Yellow),
        ));
    }

    let header =
        Paragraph::new(Line::from(header_spans)).block(Block::default().borders(Borders::BOTTOM));

    f.render_widget(header, area);
}

fn draw_main(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    match app.view_mode {
        ViewMode::List => draw_list(f, app, chunks[0]),
        ViewMode::Tree => draw_tree(f, app, chunks[0]),
    }
    draw_detail(f, app, chunks[1], app.detail_scroll);
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    // Calculate available width for title (area width minus borders and prefix)
    // Prefix is: "▌○ [T] !! peas-xxxxx " = ~23 chars (with priority and selection bar)
    let prefix_len = 23;
    let available_width = area.width.saturating_sub(2 + prefix_len) as usize; // 2 for borders
    let selected_idx = app.list_state.selected().unwrap_or(usize::MAX);

    let items: Vec<ListItem> = app
        .filtered_peas
        .iter()
        .enumerate()
        .map(|(idx, pea)| {
            let is_selected = idx == selected_idx;
            let (status_icon, status_color) = status_indicator(&pea.status);
            let (type_ind, type_color) = type_indicator(&pea.pea_type);

            // Selection indicator: bar for selected, space otherwise
            let selection_indicator = if is_selected {
                Span::styled("▌", Style::default().fg(Color::Cyan))
            } else {
                Span::raw(" ")
            };

            // Truncate title with ellipsis if too long
            let title = if pea.title.len() > available_width && available_width > 3 {
                format!("{}...", &pea.title[..available_width - 3])
            } else {
                pea.title.clone()
            };

            let mut spans = vec![
                selection_indicator,
                Span::styled(
                    format!("{} ", status_icon),
                    Style::default().fg(status_color),
                ),
                Span::styled(format!("[{}]", type_ind), Style::default().fg(type_color)),
            ];

            // Add priority indicator if not normal
            if let Some((pri_ind, pri_color)) = priority_indicator(pea) {
                spans.push(Span::styled(
                    format!("{}", pri_ind),
                    Style::default().fg(pri_color),
                ));
            }

            spans.push(Span::raw(" "));
            spans.push(Span::styled(&pea.id, Style::default().fg(Color::Cyan)));
            spans.push(Span::raw(" "));

            // Make selected row bold
            if is_selected {
                spans.push(Span::styled(
                    title,
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw(title));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = format!(" Peas ({}) ", app.filtered_peas.len());
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_tree(f: &mut Frame, app: &mut App, area: Rect) {
    let selected_idx = app.list_state.selected().unwrap_or(usize::MAX);

    // Column widths for beans-style layout
    // Format: ▌prefix+id        type        status        priority+title
    let id_col_width = 18; // peas-xxxxx with some tree prefix space
    let type_col_width = 12;
    let status_col_width = 14;

    let items: Vec<ListItem> = app
        .tree_nodes
        .iter()
        .enumerate()
        .map(|(idx, node)| {
            let pea = &node.pea;
            let is_selected = idx == selected_idx;
            let (_, status_color) = status_indicator(&pea.status);

            // Selection indicator: bar for selected, space otherwise
            let selection_indicator = if is_selected {
                Span::styled("▌", Style::default().fg(Color::Cyan))
            } else {
                Span::raw(" ")
            };

            // Build the tree prefix with ASCII art
            let mut prefix = String::new();
            for &has_line in &node.parent_lines {
                if has_line {
                    prefix.push_str("│ ");
                } else {
                    prefix.push_str("  ");
                }
            }

            // Add the branch connector for this node
            if node.depth > 0 {
                if node.is_last {
                    prefix.push_str("└─");
                } else {
                    prefix.push_str("├─");
                }
            }

            // Column 1: tree prefix + ID (padded to fixed width)
            let prefix_and_id = format!("{}{}", prefix, pea.id);
            let col1 = format!("{:<width$}", prefix_and_id, width = id_col_width);

            // Column 2: type (padded)
            let type_str = format!("{}", pea.pea_type);
            let col2 = format!("{:<width$}", type_str, width = type_col_width);

            // Column 3: status (padded)
            let status_str = format!("{}", pea.status);
            let col3 = format!("{:<width$}", status_str, width = status_col_width);

            // Calculate available width for title
            let fixed_cols = id_col_width + type_col_width + status_col_width + 5; // 5 for spacing and borders
            let available_width = area.width.saturating_sub(fixed_cols as u16) as usize;

            let title = if pea.title.len() > available_width && available_width > 3 {
                format!("{}...", &pea.title[..available_width - 3])
            } else {
                pea.title.clone()
            };

            // Build spans with columnar layout
            let mut spans = vec![
                selection_indicator,
                Span::styled(col1, Style::default().fg(Color::Cyan)),
                Span::styled(col2, Style::default().fg(type_color(&pea.pea_type))),
                Span::styled(col3, Style::default().fg(status_color)),
            ];

            // Add priority indicator if not normal
            if let Some((pri_ind, pri_color)) = priority_indicator(pea) {
                spans.push(Span::styled(
                    format!("{} ", pri_ind),
                    Style::default().fg(pri_color),
                ));
            }

            // Make selected row bold
            if is_selected {
                spans.push(Span::styled(
                    title,
                    Style::default().add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::raw(title));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = format!(" Tree ({}) ", app.tree_nodes.len());
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_stateful_widget(list, area, &mut app.list_state);
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

fn draw_detail(f: &mut Frame, app: &App, area: Rect, detail_scroll: u16) {
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
            // Find parent title
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
            for child in children.iter().take(10) {
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
            if children.len() > 10 {
                lines.push(Line::from(Span::styled(
                    format!("  ... and {} more", children.len() - 10),
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
            // Render markdown with basic formatting
            for line in pea.body.lines() {
                lines.push(render_markdown_line(line));
            }
        }

        // Add scroll hint if content is scrollable
        let content_height = lines.len() as u16;
        let visible_height = area.height.saturating_sub(2); // account for borders
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
            .title(" Details ")
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
    let footer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Filter tabs
            Constraint::Length(1), // Separator
            Constraint::Length(1), // Keybindings
        ])
        .split(area);

    // Draw filter tabs
    let filter_spans: Vec<Span> = [
        FilterType::All,
        FilterType::Open,
        FilterType::InProgress,
        FilterType::Completed,
        FilterType::Milestones,
        FilterType::Epics,
        FilterType::Tasks,
    ]
    .iter()
    .map(|ft| {
        let style = if *ft == app.filter {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        Span::styled(format!(" {} ", ft.label()), style)
    })
    .collect();

    let mut filter_line = vec![Span::styled(
        " Filter: ",
        Style::default().fg(Color::DarkGray),
    )];
    filter_line.extend(filter_spans);

    let filter_bar = Paragraph::new(Line::from(filter_line));
    f.render_widget(filter_bar, footer_chunks[0]);

    // Draw separator
    let separator = Paragraph::new(Line::from("─".repeat(area.width as usize)))
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(separator, footer_chunks[1]);

    // Draw keybindings row
    let mode_indicator = match app.input_mode {
        InputMode::Normal => Span::styled(
            " NORMAL ",
            Style::default().bg(Color::Blue).fg(Color::White),
        ),
        InputMode::Filter => Span::styled(
            " SEARCH ",
            Style::default().bg(Color::Yellow).fg(Color::Black),
        ),
    };

    let help_text = match app.input_mode {
        InputMode::Normal => {
            " j/k:nav  J/K:scroll  Tab:filter  /:search  v:view  s:start  d:done  ?:help  q:quit "
        }
        InputMode::Filter => " Type to search, Enter/Esc to confirm ",
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
    f.render_widget(keybindings, footer_chunks[2]);
}

fn draw_help_popup(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());

    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("j/↓     ", Style::default().fg(Color::Cyan)),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("k/↑     ", Style::default().fg(Color::Cyan)),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("J       ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll details down"),
        ]),
        Line::from(vec![
            Span::styled("K       ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll details up"),
        ]),
        Line::from(vec![
            Span::styled("Tab     ", Style::default().fg(Color::Cyan)),
            Span::raw("Next filter"),
        ]),
        Line::from(vec![
            Span::styled("S-Tab   ", Style::default().fg(Color::Cyan)),
            Span::raw("Previous filter"),
        ]),
        Line::from(vec![
            Span::styled("/       ", Style::default().fg(Color::Cyan)),
            Span::raw("Search"),
        ]),
        Line::from(vec![
            Span::styled("t/l     ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle tree/list view"),
        ]),
        Line::from(vec![
            Span::styled("Space   ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle status (todo -> in-progress -> completed)"),
        ]),
        Line::from(vec![
            Span::styled("s       ", Style::default().fg(Color::Cyan)),
            Span::raw("Start (set to in-progress)"),
        ]),
        Line::from(vec![
            Span::styled("d       ", Style::default().fg(Color::Cyan)),
            Span::raw("Done (set to completed)"),
        ]),
        Line::from(vec![
            Span::styled("r       ", Style::default().fg(Color::Cyan)),
            Span::raw("Refresh list"),
        ]),
        Line::from(vec![
            Span::styled("?       ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("Esc     ", Style::default().fg(Color::Cyan)),
            Span::raw("Clear search / Close help"),
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
