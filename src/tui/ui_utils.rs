use crate::model::{Pea, PeaPriority, PeaStatus, PeaType};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Text},
};
use ratatui_core;

use super::theme::theme;

/// Convert ratatui_core::Color to ratatui::Color
pub fn convert_color(core_color: ratatui_core::style::Color) -> Color {
    match core_color {
        ratatui_core::style::Color::Reset => Color::Reset,
        ratatui_core::style::Color::Black => Color::Black,
        ratatui_core::style::Color::Red => Color::Red,
        ratatui_core::style::Color::Green => Color::Green,
        ratatui_core::style::Color::Yellow => Color::Yellow,
        ratatui_core::style::Color::Blue => Color::Blue,
        ratatui_core::style::Color::Magenta => Color::Magenta,
        ratatui_core::style::Color::Cyan => Color::Cyan,
        ratatui_core::style::Color::Gray => Color::Gray,
        ratatui_core::style::Color::DarkGray => Color::DarkGray,
        ratatui_core::style::Color::LightRed => Color::LightRed,
        ratatui_core::style::Color::LightGreen => Color::LightGreen,
        ratatui_core::style::Color::LightYellow => Color::LightYellow,
        ratatui_core::style::Color::LightBlue => Color::LightBlue,
        ratatui_core::style::Color::LightMagenta => Color::LightMagenta,
        ratatui_core::style::Color::LightCyan => Color::LightCyan,
        ratatui_core::style::Color::White => Color::White,
        ratatui_core::style::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
        ratatui_core::style::Color::Indexed(i) => Color::Indexed(i),
    }
}

/// Convert ratatui_core::Modifier to ratatui::Modifier
pub fn convert_modifier(core_mod: ratatui_core::style::Modifier) -> Modifier {
    let mut modifier = Modifier::empty();
    if core_mod.contains(ratatui_core::style::Modifier::BOLD) {
        modifier |= Modifier::BOLD;
    }
    if core_mod.contains(ratatui_core::style::Modifier::DIM) {
        modifier |= Modifier::DIM;
    }
    if core_mod.contains(ratatui_core::style::Modifier::ITALIC) {
        modifier |= Modifier::ITALIC;
    }
    if core_mod.contains(ratatui_core::style::Modifier::UNDERLINED) {
        modifier |= Modifier::UNDERLINED;
    }
    if core_mod.contains(ratatui_core::style::Modifier::SLOW_BLINK) {
        modifier |= Modifier::SLOW_BLINK;
    }
    if core_mod.contains(ratatui_core::style::Modifier::RAPID_BLINK) {
        modifier |= Modifier::RAPID_BLINK;
    }
    if core_mod.contains(ratatui_core::style::Modifier::REVERSED) {
        modifier |= Modifier::REVERSED;
    }
    if core_mod.contains(ratatui_core::style::Modifier::HIDDEN) {
        modifier |= Modifier::HIDDEN;
    }
    if core_mod.contains(ratatui_core::style::Modifier::CROSSED_OUT) {
        modifier |= Modifier::CROSSED_OUT;
    }
    modifier
}

/// Convert ratatui_core::Style to ratatui::Style
pub fn convert_style(core_style: ratatui_core::style::Style) -> Style {
    let mut style = Style::default();
    if let Some(fg) = core_style.fg {
        style = style.fg(convert_color(fg));
    }
    if let Some(bg) = core_style.bg {
        style = style.bg(convert_color(bg));
    }
    style = style.add_modifier(convert_modifier(core_style.add_modifier));
    style = style.remove_modifier(convert_modifier(core_style.sub_modifier));
    style
}

/// Estimate the number of wrapped lines for a Text widget
pub fn estimate_wrapped_lines(text: &Text, width: usize) -> u16 {
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

/// Highlight search term in text by splitting into spans
pub fn highlight_search<'a>(text: &str, query: &str, base_style: Style) -> Vec<Span<'a>> {
    if query.is_empty() {
        return vec![Span::styled(text.to_string(), base_style)];
    }

    let t = theme();
    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    let mut spans = Vec::new();
    let mut last_end = 0;

    for (idx, _) in lower_text.match_indices(&lower_query) {
        // Add text before match
        if idx > last_end {
            spans.push(Span::styled(text[last_end..idx].to_string(), base_style));
        }
        // Add highlighted match
        spans.push(Span::styled(
            text[idx..idx + query.len()].to_string(),
            base_style
                .fg(t.modal_border_create) // Blue highlight
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));
        last_end = idx + query.len();
    }

    // Add remaining text
    if last_end < text.len() {
        spans.push(Span::styled(text[last_end..].to_string(), base_style));
    }

    spans
}

/// Returns priority indicator and color for a pea
pub fn priority_indicator(pea: &Pea) -> Option<(String, Color)> {
    theme()
        .priority_indicator(&pea.priority)
        .map(|(s, c)| (s.to_string(), c))
}

/// Returns status icon and color
pub fn status_indicator(status: &PeaStatus) -> (&'static str, Color) {
    theme().status_indicator(status)
}

/// Returns priority color
pub fn priority_color(priority: &PeaPriority) -> Color {
    theme().priority_color(priority)
}

/// Returns type color
pub fn type_color(pea_type: &PeaType) -> Color {
    theme().type_color(pea_type)
}

/// Calculate a centered rectangle within a parent rectangle
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
