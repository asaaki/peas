//! Central theme configuration for the TUI.
//!
//! All colors and styles are defined here to maintain consistency
//! and enable future theming capabilities.

use ratatui::style::{Color, Modifier, Style};

use crate::model::{PeaPriority, PeaStatus, PeaType};

/// Theme configuration for the TUI
#[derive(Debug, Clone)]
pub struct Theme {
    // General UI
    pub border: Color,
    pub border_focused: Color,
    pub selection_indicator: Color,
    pub cursor_blink: bool,

    // Text
    pub text: Color,
    pub text_muted: Color,
    pub text_highlight: Color,

    // Status colors
    pub status_draft: Color,
    pub status_todo: Color,
    pub status_in_progress: Color,
    pub status_completed: Color,
    pub status_scrapped: Color,

    // Priority colors
    pub priority_critical: Color,
    pub priority_high: Color,
    pub priority_normal: Color,
    pub priority_low: Color,
    pub priority_deferred: Color,

    // Type colors
    pub type_milestone: Color,
    pub type_epic: Color,
    pub type_story: Color,
    pub type_feature: Color,
    pub type_bug: Color,
    pub type_chore: Color,
    pub type_research: Color,
    pub type_task: Color,

    // Relation colors
    pub relation_parent: Color,
    pub relation_blocks: Color,
    pub relation_child: Color,

    // ID colors
    pub id: Color,
    pub id_selected: Color,

    // Tags
    pub tags: Color,

    // Timestamps
    pub timestamp: Color,

    // Modal colors
    pub modal_border: Color,
    pub modal_border_delete: Color,
    pub modal_border_create: Color,

    // Footer/Mode colors
    pub mode_normal: (Color, Color), // (bg, fg)
    pub mode_search: (Color, Color),
    pub mode_status: (Color, Color),
    pub mode_priority: (Color, Color),
    pub mode_type: (Color, Color),
    pub mode_delete: (Color, Color),
    pub mode_parent: (Color, Color),
    pub mode_blocking: (Color, Color),
    pub mode_detail: (Color, Color),
    pub mode_create: (Color, Color),

    // Checkbox colors
    pub checkbox_checked: Color,
    pub checkbox_unchecked: Color,

    // Multi-select
    pub multi_select: Color,

    // Tree lines
    pub tree_lines: Color,

    // Message
    pub message: Color,

    // Modal UI elements
    pub modal_cursor: Color,
    pub modal_highlight_bg: Color,

    // Help popup
    pub help_key: Color,
    pub help_border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // General UI
            border: Color::Gray,
            border_focused: Color::Green,
            selection_indicator: Color::Green,
            cursor_blink: true,

            // Text
            text: Color::White,
            text_muted: Color::DarkGray,
            text_highlight: Color::White,

            // Status colors
            status_draft: Color::DarkGray,
            status_todo: Color::Green,
            status_in_progress: Color::Yellow,
            status_completed: Color::DarkGray,
            status_scrapped: Color::DarkGray,

            // Priority colors
            priority_critical: Color::Red,
            priority_high: Color::LightRed,
            priority_normal: Color::White,
            priority_low: Color::DarkGray,
            priority_deferred: Color::DarkGray,

            // Type colors
            type_milestone: Color::Magenta,
            type_epic: Color::Blue,
            type_story: Color::Cyan,
            type_feature: Color::Cyan,
            type_bug: Color::Red,
            type_chore: Color::Yellow,
            type_research: Color::LightMagenta,
            type_task: Color::White,

            // Relation colors
            relation_parent: Color::Yellow,
            relation_blocks: Color::LightRed,
            relation_child: Color::Cyan,

            // ID colors
            id: Color::Green,
            id_selected: Color::LightGreen,

            // Tags
            tags: Color::Magenta,

            // Timestamps
            timestamp: Color::DarkGray,

            // Modal colors
            modal_border: Color::Yellow,
            modal_border_delete: Color::Red,
            modal_border_create: Color::Cyan,

            // Footer/Mode colors (bg, fg)
            mode_normal: (Color::Blue, Color::White),
            mode_search: (Color::Yellow, Color::Black),
            mode_status: (Color::Green, Color::Black),
            mode_priority: (Color::Red, Color::White),
            mode_type: (Color::Magenta, Color::White),
            mode_delete: (Color::Red, Color::White),
            mode_parent: (Color::Blue, Color::White),
            mode_blocking: (Color::LightRed, Color::Black),
            mode_detail: (Color::Green, Color::Black),
            mode_create: (Color::Cyan, Color::Black),

            // Checkbox colors
            checkbox_checked: Color::Green,
            checkbox_unchecked: Color::DarkGray,

            // Multi-select
            multi_select: Color::Cyan,

            // Tree lines
            tree_lines: Color::DarkGray,

            // Message
            message: Color::Green,

            // Modal UI elements
            modal_cursor: Color::Cyan,
            modal_highlight_bg: Color::DarkGray,

            // Help popup
            help_key: Color::Cyan,
            help_border: Color::Yellow,
        }
    }
}

impl Theme {
    /// Get status color
    pub fn status_color(&self, status: &PeaStatus) -> Color {
        match status {
            PeaStatus::Draft => self.status_draft,
            PeaStatus::Todo => self.status_todo,
            PeaStatus::InProgress => self.status_in_progress,
            PeaStatus::Completed => self.status_completed,
            PeaStatus::Scrapped => self.status_scrapped,
        }
    }

    /// Get priority color
    pub fn priority_color(&self, priority: &PeaPriority) -> Color {
        match priority {
            PeaPriority::Critical => self.priority_critical,
            PeaPriority::High => self.priority_high,
            PeaPriority::Normal => self.priority_normal,
            PeaPriority::Low => self.priority_low,
            PeaPriority::Deferred => self.priority_deferred,
        }
    }

    /// Get type color
    pub fn type_color(&self, pea_type: &PeaType) -> Color {
        match pea_type {
            PeaType::Milestone => self.type_milestone,
            PeaType::Epic => self.type_epic,
            PeaType::Story => self.type_story,
            PeaType::Feature => self.type_feature,
            PeaType::Bug => self.type_bug,
            PeaType::Chore => self.type_chore,
            PeaType::Research => self.type_research,
            PeaType::Task => self.type_task,
        }
    }

    /// Get status indicator (icon, color)
    pub fn status_indicator(&self, status: &PeaStatus) -> (&'static str, Color) {
        let icon = match status {
            PeaStatus::Draft => "○",
            PeaStatus::Todo => "○",
            PeaStatus::InProgress => "◐",
            PeaStatus::Completed => "●",
            PeaStatus::Scrapped => "✗",
        };
        (icon, self.status_color(status))
    }

    /// Get priority indicator (icon, color) - returns None for normal priority
    pub fn priority_indicator(&self, priority: &PeaPriority) -> Option<(&'static str, Color)> {
        match priority {
            PeaPriority::Critical => Some(("‼", self.priority_critical)),
            PeaPriority::High => Some(("!", self.priority_high)),
            PeaPriority::Normal => None,
            PeaPriority::Low => Some(("↓", self.priority_low)),
            PeaPriority::Deferred => Some(("⏸", self.priority_deferred)),
        }
    }

    /// Get relation color by type string
    pub fn relation_color(&self, rel_type: &str) -> Color {
        match rel_type {
            "Parent" => self.relation_parent,
            "Blocks" => self.relation_blocks,
            "Child" => self.relation_child,
            _ => self.text,
        }
    }

    /// Get relation prefix by type string
    pub fn relation_prefix(rel_type: &str) -> &'static str {
        match rel_type {
            "Parent" => "↑",
            "Blocks" => "→",
            "Child" => "↓",
            _ => " ",
        }
    }

    // Style builders

    /// Style for selected items
    pub fn selected_style(&self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    /// Style for ID in tree view
    pub fn id_style(&self, selected: bool) -> Style {
        if selected {
            Style::default()
                .fg(self.id_selected)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.id)
        }
    }

    /// Style for selection indicator (blinking cursor)
    pub fn selection_indicator_style(&self) -> Style {
        let style = Style::default().fg(self.selection_indicator);
        if self.cursor_blink {
            style.add_modifier(Modifier::SLOW_BLINK)
        } else {
            style
        }
    }

    /// Border style for blocks
    pub fn border_style(&self, focused: bool) -> Style {
        Style::default().fg(if focused {
            self.border_focused
        } else {
            self.border
        })
    }
}

/// Global theme instance
static THEME: std::sync::OnceLock<Theme> = std::sync::OnceLock::new();

/// Get the current theme
pub fn theme() -> &'static Theme {
    THEME.get_or_init(Theme::default)
}
