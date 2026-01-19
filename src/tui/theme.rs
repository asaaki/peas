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
        // Monokai color scheme
        // Based on classic Monokai: https://monokai.pro/
        Self {
            // General UI - Monokai borders and focus
            border: Color::Rgb(117, 113, 94), // Muted brownish-gray
            border_focused: Color::Rgb(166, 226, 46), // Monokai green
            selection_indicator: Color::Rgb(166, 226, 46), // Monokai green
            cursor_blink: true,

            // Text - Monokai foreground colors
            text: Color::Rgb(248, 248, 242), // Monokai foreground
            text_muted: Color::Rgb(117, 113, 94), // Muted comment color
            text_highlight: Color::Rgb(248, 248, 242), // Bright foreground

            // Status colors - Monokai palette
            status_draft: Color::Rgb(117, 113, 94), // Muted
            status_todo: Color::Rgb(166, 226, 46),  // Green
            status_in_progress: Color::Rgb(230, 219, 116), // Yellow
            status_completed: Color::Rgb(117, 113, 94), // Muted (de-emphasized)
            status_scrapped: Color::Rgb(117, 113, 94), // Muted

            // Priority colors - Monokai vibrant colors
            priority_critical: Color::Rgb(249, 38, 114), // Monokai pink/red
            priority_high: Color::Rgb(253, 151, 31),     // Monokai orange
            priority_normal: Color::Rgb(248, 248, 242),  // Normal foreground
            priority_low: Color::Rgb(117, 113, 94),      // Muted
            priority_deferred: Color::Rgb(117, 113, 94), // Muted

            // Type colors - Monokai palette variety
            type_milestone: Color::Rgb(174, 129, 255), // Monokai purple
            type_epic: Color::Rgb(102, 217, 239),      // Monokai blue
            type_story: Color::Rgb(102, 217, 239),     // Monokai blue
            type_feature: Color::Rgb(166, 226, 46),    // Monokai green
            type_bug: Color::Rgb(249, 38, 114),        // Monokai pink
            type_chore: Color::Rgb(230, 219, 116),     // Monokai yellow
            type_research: Color::Rgb(174, 129, 255),  // Monokai purple
            type_task: Color::Rgb(248, 248, 242),      // Normal text

            // Relation colors
            relation_parent: Color::Rgb(230, 219, 116), // Yellow
            relation_blocks: Color::Rgb(253, 151, 31),  // Orange
            relation_child: Color::Rgb(102, 217, 239),  // Blue

            // ID colors - Monokai green
            id: Color::Rgb(166, 226, 46),          // Monokai green
            id_selected: Color::Rgb(166, 226, 46), // Same green (bright already)

            // Tags - Monokai purple
            tags: Color::Rgb(174, 129, 255),

            // Timestamps
            timestamp: Color::Rgb(117, 113, 94), // Muted

            // Modal colors
            modal_border: Color::Rgb(230, 219, 116), // Yellow
            modal_border_delete: Color::Rgb(249, 38, 114), // Pink/red
            modal_border_create: Color::Rgb(102, 217, 239), // Blue

            // Footer/Mode colors (bg, fg)
            mode_normal: (Color::Rgb(102, 217, 239), Color::Rgb(39, 40, 34)), // Blue bg
            mode_search: (Color::Rgb(230, 219, 116), Color::Rgb(39, 40, 34)), // Yellow bg
            mode_status: (Color::Rgb(166, 226, 46), Color::Rgb(39, 40, 34)),  // Green bg
            mode_priority: (Color::Rgb(249, 38, 114), Color::Rgb(248, 248, 242)), // Pink bg
            mode_type: (Color::Rgb(174, 129, 255), Color::Rgb(248, 248, 242)), // Purple bg
            mode_delete: (Color::Rgb(249, 38, 114), Color::Rgb(248, 248, 242)), // Pink bg
            mode_parent: (Color::Rgb(102, 217, 239), Color::Rgb(39, 40, 34)), // Blue bg
            mode_blocking: (Color::Rgb(253, 151, 31), Color::Rgb(39, 40, 34)), // Orange bg
            mode_detail: (Color::Rgb(166, 226, 46), Color::Rgb(39, 40, 34)),  // Green bg
            mode_create: (Color::Rgb(102, 217, 239), Color::Rgb(39, 40, 34)), // Blue bg

            // Checkbox colors
            checkbox_checked: Color::Rgb(166, 226, 46), // Green
            checkbox_unchecked: Color::Rgb(117, 113, 94), // Muted

            // Multi-select
            multi_select: Color::Rgb(102, 217, 239), // Blue

            // Tree lines
            tree_lines: Color::Rgb(117, 113, 94), // Muted

            // Message
            message: Color::Rgb(166, 226, 46), // Green

            // Modal UI elements
            modal_cursor: Color::Rgb(102, 217, 239), // Blue
            modal_highlight_bg: Color::Rgb(73, 72, 62), // Slightly lighter than bg

            // Help popup
            help_key: Color::Rgb(102, 217, 239),    // Blue
            help_border: Color::Rgb(230, 219, 116), // Yellow
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
