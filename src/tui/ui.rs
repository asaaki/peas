use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::model::{PeaPriority, PeaStatus, PeaType};

use super::app::{App, FilterType, InputMode};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
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

    let mut header_spans = vec![Span::styled(
        " peas ",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )];
    header_spans.push(Span::raw(" | "));
    header_spans.extend(filter_spans);

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

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_list(f, app, chunks[0]);
    draw_detail(f, app, chunks[1]);
}

fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_peas
        .iter()
        .enumerate()
        .map(|(i, pea)| {
            let status_icon = match pea.status {
                PeaStatus::Draft => "○",
                PeaStatus::Todo => "○",
                PeaStatus::InProgress => "◐",
                PeaStatus::Completed => "●",
                PeaStatus::Scrapped => "✗",
            };

            let status_color = match pea.status {
                PeaStatus::Draft => Color::DarkGray,
                PeaStatus::Todo => Color::White,
                PeaStatus::InProgress => Color::Yellow,
                PeaStatus::Completed => Color::Green,
                PeaStatus::Scrapped => Color::Red,
            };

            let type_indicator = match pea.pea_type {
                PeaType::Milestone => "M",
                PeaType::Epic => "E",
                PeaType::Feature => "F",
                PeaType::Bug => "B",
                PeaType::Task => "T",
            };

            let type_color = match pea.pea_type {
                PeaType::Milestone => Color::Magenta,
                PeaType::Epic => Color::Blue,
                PeaType::Feature => Color::Cyan,
                PeaType::Bug => Color::Red,
                PeaType::Task => Color::White,
            };

            let style = if i == app.selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled(
                    format!("{} ", status_icon),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    format!("[{}] ", type_indicator),
                    Style::default().fg(type_color),
                ),
                Span::styled(&pea.id, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::raw(&pea.title),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(" Peas ({}) ", app.filtered_peas.len());
    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, area);
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let detail_block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
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
            lines.push(Line::from(vec![
                Span::raw("Parent:   "),
                Span::styled(parent, Style::default().fg(Color::Cyan)),
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

        if !pea.body.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Description:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )));
            for line in pea.body.lines().take(15) {
                lines.push(Line::from(line.to_string()));
            }
        }

        let detail = Paragraph::new(Text::from(lines))
            .block(detail_block)
            .wrap(Wrap { trim: true });

        f.render_widget(detail, area);
    } else {
        let empty = Paragraph::new("No pea selected")
            .block(detail_block)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(empty, area);
    }
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
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
            " j/k:nav  Tab:filter  /:search  s:start  d:done  Space:toggle  ?:help  q:quit "
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

    let footer =
        Paragraph::new(Line::from(footer_spans)).block(Block::default().borders(Borders::TOP));

    f.render_widget(footer, area);
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
