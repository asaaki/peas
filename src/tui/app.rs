//! TUI Application State Machine
//!
//! This module implements the core state management for the Peas TUI.
//! See `docs/tui-state-machine.md` for comprehensive documentation.
//!
//! # State Machine Overview
//!
//! The TUI operates as a modal state machine with the following primary modes:
//! - **Normal**: Browse, navigate, and trigger actions
//! - **Filter**: Search/filter tickets
//! - **EditBody**: Multi-line body editing
//! - **Modal***: Various modal dialogs (Status, Priority, Type, Parent, etc.)
//! - **Create***: Ticket/Memory creation workflows
//!
//! # State Invariants
//!
//! The following invariants must be maintained:
//! - `selected_index` must be < `tree_nodes.len()` in Normal mode
//! - `modal_selection` must be < options count in Modal modes
//! - `body_textarea` must be Some() when `input_mode == EditBody`
//! - `filtered_peas` must be a subset of `all_peas`
//!
//! # Concurrency
//!
//! The TUI implements concurrent edit detection to prevent lost updates when
//! multiple instances are running or when CLI commands modify files.

use super::{body_editor, handlers, modal_operations, relations, tree_builder, ui, url_utils};
use crate::{
    config::PeasConfig,
    error::Result,
    model::{Memory, Pea, PeaPriority, PeaStatus, PeaType},
    storage::{MemoryRepository, PeaRepository},
    undo::UndoManager,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use rat_text::text_area::TextAreaState;
use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};
use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};
use tree_builder::{PageInfo, TreeNode};

/// Top-level view mode for the TUI
///
/// Determines which data set is displayed and which operations are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Ticket tree view - hierarchical display of peas
    Tickets,
    /// Memory list view - key-value session data
    Memory,
}

/// Input mode state machine
///
/// Determines how keyboard input is processed and which UI elements are displayed.
/// All modal modes can return to Normal via Esc or Enter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Default browsing mode - navigate list, view details, trigger actions
    Normal,
    /// Search/filter mode - type to filter tickets
    Filter,
    /// Status selection modal
    StatusModal,
    /// Priority selection modal
    PriorityModal,
    /// Type selection modal
    TypeModal,
    /// Delete confirmation modal
    DeleteConfirm,
    /// Parent ticket selection modal
    ParentModal,
    /// Blocking tickets multi-selection modal
    BlockingModal,
    /// Detail view mode (deprecated, use Normal with detail_pane instead)
    DetailView,
    /// Create new ticket modal (3-field form)
    CreateModal,
    /// Create new memory modal (3-field form)
    MemoryCreateModal,
    /// Multi-line body editing with textarea
    EditBody,
    /// Tag editing modal (comma-separated input)
    TagsModal,
    /// URL selection modal (choose URL from ticket body)
    UrlModal,
}

/// Detail pane selection in Normal mode
///
/// Determines which information is displayed in the detail area when viewing a ticket.
/// Switch between panes with number keys 1-4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailPane {
    /// Ticket metadata: type, status, priority, tags
    Metadata,
    /// Description/markdown content (default)
    #[default]
    Body,
    /// Parent and blocking relationships
    Relations,
    /// Attached asset files
    Assets,
}

/// Main TUI application state
///
/// This struct contains all state for the terminal user interface.
/// See module documentation and `docs/tui-state-machine.md` for details.
///
/// # Invariants
///
/// The following must always hold:
/// - `selected_index < tree_nodes.len()` when `input_mode == Normal` and `view_mode == Tickets`
/// - `modal_selection < options.len()` in any modal mode
/// - `body_textarea.is_some()` if and only if `input_mode == EditBody`
/// - `filtered_peas` is a subset of `all_peas`
/// - `filtered_memories` is a subset of `all_memories`
pub struct App {
    // ========== View State ==========
    /// Current view mode (Tickets or Memory)
    pub view_mode: ViewMode,

    // ========== Data Sources ==========
    /// Repository for ticket operations
    pub repo: PeaRepository,
    /// Repository for memory operations
    pub memory_repo: MemoryRepository,
    /// Path to .peas data directory
    pub data_path: PathBuf,

    // ========== Ticket Data ==========
    /// All tickets (unfiltered)
    pub all_peas: Vec<Pea>,
    /// Filtered/searched tickets (displayed)
    pub filtered_peas: Vec<Pea>,
    /// Tree structure for hierarchical display
    pub tree_nodes: Vec<TreeNode>,
    /// Virtual page table for navigation
    pub page_table: Vec<PageInfo>,

    // ========== Memory Data ==========
    /// All memories (unfiltered)
    pub all_memories: Vec<Memory>,
    /// Filtered/searched memories (displayed)
    pub filtered_memories: Vec<Memory>,

    // ========== Selection & Navigation ==========
    /// Selected index in tree_nodes (Tickets) or filtered_memories (Memory)
    pub selected_index: usize,
    /// Number of items visible per page
    pub page_height: usize,
    /// Ratatui list state for rendering
    pub list_state: ListState,
    /// Multi-selected ticket IDs (for bulk operations)
    pub multi_selected: HashSet<String>,

    // ========== Detail Pane State ==========
    /// Which detail pane is active
    pub detail_pane: DetailPane,
    /// Scroll offset for body/description pane
    pub detail_scroll: u16,
    /// Maximum scroll for body (0 = no scrolling needed)
    pub detail_max_scroll: u16,

    // ========== Relations Pane State ==========
    /// Scroll offset for relationships pane
    pub relations_scroll: u16,
    /// Selected item in relationships list
    pub relations_selection: usize,
    /// Relationship items: (type, id, title, pea_type)
    pub relations_items: Vec<(String, String, String, PeaType)>,

    // ========== Assets Pane State ==========
    /// Selected item in assets list
    pub assets_selection: usize,
    /// Asset file information for current ticket
    pub assets_items: Vec<crate::assets::AssetInfo>,

    // ========== Metadata Pane State ==========
    /// Selected property (0=type, 1=status, 2=priority, 3=tags)
    pub metadata_selection: usize,

    // ========== Input Mode ==========
    /// Current input mode (state machine state)
    pub input_mode: InputMode,
    /// Previous mode (for modal return)
    pub previous_mode: InputMode,

    // ========== Filter State ==========
    /// Search query text (supports regex and field-specific search)
    pub search_query: String,

    // ========== UI State ==========
    /// Whether help overlay is shown
    pub show_help: bool,
    /// Status message to display
    pub message: Option<String>,
    /// App start time (for animations)
    pub start_time: Instant,

    // ========== Modal State ==========
    /// Current selection in modal dialogs
    pub modal_selection: usize,
    /// Candidates for parent selection modal
    pub parent_candidates: Vec<Pea>,
    /// Candidates for blocking selection modal
    pub blocking_candidates: Vec<Pea>,
    /// Which blocking candidates are selected (multi-select)
    pub blocking_selected: Vec<bool>,
    /// URLs extracted from current ticket body
    pub url_candidates: Vec<String>,

    // ========== Create Modal State ==========
    /// Title input for create modal
    pub create_title: String,
    /// Type selection for create modal
    pub create_type: PeaType,
    /// Tag input for tags modal (comma-separated)
    pub tags_input: String,

    // ========== Memory Create Modal State ==========
    /// Key input for memory create modal
    pub memory_create_key: String,
    /// Tags input for memory create modal
    pub memory_create_tags: String,
    /// Content input for memory create modal
    pub memory_create_content: String,
    /// Current field in memory create modal (0=key, 1=tags, 2=content)
    pub memory_modal_selection: usize,

    // ========== Body Editor State ==========
    /// TextArea for multi-line body editing (Some when input_mode == EditBody)
    pub body_textarea: Option<TextAreaState>,
}

impl App {
    pub fn new(config: &PeasConfig, project_root: &Path) -> Result<Self> {
        // Initialize TUI config with settings
        super::theme::init_tui_config(config.tui.use_type_emojis);

        let repo = PeaRepository::new(config, project_root);
        let memory_repo = MemoryRepository::new(config, project_root);
        let data_path = config.data_path(project_root);
        let all_peas = repo.list()?;
        let filtered_peas = all_peas.clone();
        let all_memories = memory_repo.list(None).unwrap_or_default();
        let filtered_memories = all_memories.clone();

        let mut list_state = ListState::default();
        if !filtered_peas.is_empty() {
            list_state.select(Some(0));
        }

        let mut app = Self {
            view_mode: ViewMode::Tickets,
            repo,
            memory_repo,
            data_path,
            all_peas,
            filtered_peas,
            all_memories,
            filtered_memories,
            tree_nodes: Vec::new(),
            page_table: Vec::new(),
            selected_index: 0,
            page_height: 20, // Default, updated when drawing
            list_state,
            detail_scroll: 0,
            detail_max_scroll: 0,
            relations_scroll: 0,
            relations_selection: 0,
            relations_items: Vec::new(),
            assets_selection: 0,
            assets_items: Vec::new(),
            metadata_selection: 0,
            detail_pane: DetailPane::default(),
            input_mode: InputMode::Normal,
            previous_mode: InputMode::Normal,
            search_query: String::new(),
            show_help: false,
            message: None,
            modal_selection: 0,
            parent_candidates: Vec::new(),
            blocking_candidates: Vec::new(),
            blocking_selected: Vec::new(),
            create_title: String::new(),
            create_type: PeaType::Task,
            tags_input: String::new(),
            multi_selected: HashSet::new(),
            body_textarea: None,
            start_time: Instant::now(),
            url_candidates: Vec::new(),
            memory_create_key: String::new(),
            memory_create_tags: String::new(),
            memory_create_content: String::new(),
            memory_modal_selection: 0,
        };
        app.build_tree();
        // Note: page_table will be built when page_height is set during first draw
        Ok(app)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.all_peas = self.repo.list()?;
        self.all_memories = self.memory_repo.list(None).unwrap_or_default();
        self.apply_filter();
        self.build_tree();
        if self.page_height > 0 {
            self.build_page_table();
        }
        Ok(())
    }

    pub fn switch_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Tickets => ViewMode::Memory,
            ViewMode::Memory => ViewMode::Tickets,
        };
        // Reset selection when switching views
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.detail_scroll = 0;
    }

    /// Handle mouse click events
    pub fn handle_mouse_click(&mut self, _column: u16, row: u16) {
        // In Normal mode, clicking on list items should select them
        if self.input_mode == InputMode::Normal {
            // Account for the top border of the list block
            // Row 0 = top border, Row 1+ = content inside the block
            if row >= 1 {
                let clicked_row = (row - 1) as usize;

                match self.view_mode {
                    ViewMode::Tickets => {
                        if clicked_row < self.tree_nodes.len() {
                            self.selected_index = clicked_row;
                            self.list_state.select(Some(clicked_row));
                        }
                    }
                    ViewMode::Memory => {
                        if clicked_row < self.filtered_memories.len() {
                            self.selected_index = clicked_row;
                            self.list_state.select(Some(clicked_row));
                        }
                    }
                }
            }
        }
    }

    /// Build a flattened tree structure from the filtered peas
    pub fn build_tree(&mut self) {
        self.tree_nodes = tree_builder::build_tree(&self.filtered_peas);
    }

    /// Build a virtual page table that accounts for parent context rows
    pub fn build_page_table(&mut self) {
        self.page_table = tree_builder::build_page_table(&self.tree_nodes, self.page_height);
    }

    /// Returns the number of items in the current view
    pub fn display_count(&self) -> usize {
        self.tree_nodes.len()
    }

    /// Returns the current page number (0-indexed) using page table
    pub fn current_page(&self) -> usize {
        if self.page_table.is_empty() {
            return 0;
        }

        // Find which page contains selected_index
        for (page_num, page_info) in self.page_table.iter().enumerate() {
            let end = page_info.start_index + page_info.item_count;
            if self.selected_index < end {
                return page_num;
            }
        }

        // If not found, return last page
        self.page_table.len().saturating_sub(1)
    }

    /// Returns the total number of pages
    pub fn total_pages(&self) -> usize {
        self.page_table.len().max(1)
    }

    /// Returns the index within the current page (0-indexed)
    pub fn index_in_page(&self) -> usize {
        if self.page_height == 0 {
            0
        } else {
            self.selected_index % self.page_height
        }
    }

    /// Returns the start index of the current page
    pub fn apply_filter(&mut self) {
        // Filter tickets
        self.filtered_peas = self
            .all_peas
            .iter()
            .filter(|p| {
                // Search filter (supports field-specific and regex)
                if self.search_query.is_empty() {
                    true
                } else {
                    // Parse search query and apply
                    match crate::search::SearchQuery::parse(&self.search_query) {
                        Ok(query) => query.matches_pea(p),
                        Err(_) => {
                            // If parse fails, fall back to simple substring search
                            let query = self.search_query.to_lowercase();
                            p.title.to_lowercase().contains(&query)
                                || p.id.to_lowercase().contains(&query)
                                || p.body.to_lowercase().contains(&query)
                                || p.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
                        }
                    }
                }
            })
            .cloned()
            .collect();

        // Filter memories
        self.filtered_memories = self
            .all_memories
            .iter()
            .filter(|m| {
                // Search filter (supports field-specific and regex)
                if self.search_query.is_empty() {
                    true
                } else {
                    // Parse search query and apply
                    match crate::search::SearchQuery::parse(&self.search_query) {
                        Ok(query) => query.matches_memory(m),
                        Err(_) => {
                            // If parse fails, fall back to simple substring search
                            let query = self.search_query.to_lowercase();
                            m.key.to_lowercase().contains(&query)
                                || m.content.to_lowercase().contains(&query)
                                || m.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
                        }
                    }
                }
            })
            .cloned()
            .collect();

        // Adjust selection based on current view
        match self.view_mode {
            ViewMode::Tickets => {
                if self.selected_index >= self.filtered_peas.len() {
                    self.selected_index = self.filtered_peas.len().saturating_sub(1);
                }
            }
            ViewMode::Memory => {
                if self.selected_index >= self.filtered_memories.len() {
                    self.selected_index = self.filtered_memories.len().saturating_sub(1);
                }
            }
        }

        // Rebuild tree after filter changes (only for tickets)
        self.build_tree();
        if self.page_height > 0 {
            self.build_page_table();
        }

        let count = self.display_count();
        if count == 0 {
            self.list_state.select(None);
        } else {
            if self.selected_index >= count {
                self.selected_index = count.saturating_sub(1);
            }
            self.list_state.select(Some(self.selected_index));
        }
    }

    pub fn selected_pea(&self) -> Option<&Pea> {
        self.tree_nodes.get(self.selected_index).map(|n| &n.pea)
    }

    /// Check if a ticket is multi-selected
    pub fn is_multi_selected(&self, id: &str) -> bool {
        self.multi_selected.contains(id)
    }

    /// Toggle multi-selection for the current ticket
    pub fn toggle_multi_select(&mut self) {
        if let Some(pea) = self.selected_pea() {
            let id = pea.id.clone();
            if self.multi_selected.contains(&id) {
                self.multi_selected.remove(&id);
            } else {
                self.multi_selected.insert(id);
            }
        }
    }

    /// Clear all multi-selections
    pub fn clear_multi_select(&mut self) {
        self.multi_selected.clear();
    }

    /// Get the IDs to operate on: multi-selected if any, otherwise current selection
    pub fn target_ids(&self) -> Vec<String> {
        if self.multi_selected.is_empty() {
            self.selected_pea()
                .map(|p| vec![p.id.clone()])
                .unwrap_or_default()
        } else {
            self.multi_selected.iter().cloned().collect()
        }
    }

    /// Get count of multi-selected items
    pub fn multi_select_count(&self) -> usize {
        self.multi_selected.len()
    }

    pub fn selected_pea_file_path(&self) -> Option<PathBuf> {
        self.selected_pea()
            .and_then(|pea| self.repo.find_file_by_id(&pea.id).ok())
    }

    pub fn next(&mut self) {
        let count = self.display_count();
        if count > 0 {
            if self.selected_index + 1 < count {
                self.selected_index += 1;
            }
            // list_state selection is relative to the current page
            self.list_state.select(Some(self.index_in_page()));
            self.detail_scroll = 0;
        }
    }

    pub fn previous(&mut self) {
        if self.display_count() > 0 && self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.index_in_page()));
            self.detail_scroll = 0;
        }
    }

    /// Jump to next page using page table
    pub fn next_page(&mut self) {
        if self.page_table.is_empty() {
            return;
        }

        let current_page = self.current_page();
        if current_page + 1 < self.page_table.len() {
            // Go to first item of next page
            self.selected_index = self.page_table[current_page + 1].start_index;
        } else {
            // Already on last page, go to last item
            self.selected_index = self.tree_nodes.len().saturating_sub(1);
        }
        self.list_state.select(Some(self.index_in_page()));
        self.detail_scroll = 0;
    }

    /// Jump to previous page using page table
    pub fn previous_page(&mut self) {
        if self.page_table.is_empty() {
            return;
        }

        let current_page = self.current_page();
        if current_page > 0 {
            // Go to first item of previous page
            self.selected_index = self.page_table[current_page - 1].start_index;
        } else {
            // Already on first page, go to first item
            self.selected_index = 0;
        }
        self.list_state.select(Some(self.index_in_page()));
        self.detail_scroll = 0;
    }

    /// Jump to first item
    pub fn first(&mut self) {
        if self.display_count() > 0 {
            self.selected_index = 0;
            self.list_state.select(Some(0));
            self.detail_scroll = 0;
        }
    }

    /// Jump to last item
    pub fn last(&mut self) {
        let count = self.display_count();
        if count > 0 {
            self.selected_index = count - 1;
            self.list_state.select(Some(self.index_in_page()));
            self.detail_scroll = 0;
        }
    }

    pub fn scroll_detail_down(&mut self) {
        if self.detail_scroll < self.detail_max_scroll {
            self.detail_scroll = self.detail_scroll.saturating_add(1);
        }
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    /// Set the maximum scroll value (called from UI during render)
    pub fn set_detail_max_scroll(&mut self, max_scroll: u16) {
        self.detail_max_scroll = max_scroll;
        // Clamp current scroll to new max
        if self.detail_scroll > max_scroll {
            self.detail_scroll = max_scroll;
        }
    }

    /// Build the relationships list for the current pea
    pub fn build_relations(&mut self) {
        self.relations_selection = 0;
        self.relations_scroll = 0;

        if let Some(pea) = self.selected_pea() {
            self.relations_items = relations::build_relations(pea, &self.all_peas);
        } else {
            self.relations_items.clear();
        }

        // Also rebuild assets when updating relations
        self.rebuild_assets();
    }

    /// Navigate down in relationships pane
    pub fn relations_next(&mut self) {
        if !self.relations_items.is_empty() {
            self.relations_selection = (self.relations_selection + 1) % self.relations_items.len();
        }
    }

    /// Navigate up in relationships pane
    pub fn relations_previous(&mut self) {
        if !self.relations_items.is_empty() {
            self.relations_selection = if self.relations_selection == 0 {
                self.relations_items.len() - 1
            } else {
                self.relations_selection - 1
            };
        }
    }

    /// Jump to the selected relationship ticket
    pub fn jump_to_relation(&mut self) -> bool {
        if let Some((_, id, _, _)) = self.relations_items.get(self.relations_selection) {
            let target_id = id.clone();
            // Find the ticket in tree_nodes
            if let Some(idx) = self.tree_nodes.iter().position(|n| n.pea.id == target_id) {
                self.selected_index = idx;
                self.list_state.select(Some(self.index_in_page()));
                self.detail_scroll = 0;
                self.build_relations(); // Rebuild for new ticket
                return true;
            }
        }
        false
    }

    /// Navigate down in assets pane
    pub fn assets_next(&mut self) {
        if !self.assets_items.is_empty() {
            self.assets_selection = (self.assets_selection + 1) % self.assets_items.len();
        }
    }

    /// Navigate up in assets pane
    pub fn assets_previous(&mut self) {
        if !self.assets_items.is_empty() {
            self.assets_selection = if self.assets_selection == 0 {
                self.assets_items.len() - 1
            } else {
                self.assets_selection - 1
            };
        }
    }

    /// Open the selected asset
    pub fn open_selected_asset(&self) -> std::io::Result<()> {
        if let Some(asset) = self.assets_items.get(self.assets_selection) {
            // Open with platform-specific command
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("cmd")
                    .args(["/C", "start", "", asset.path.to_str().unwrap()])
                    .spawn()?;
            }

            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(&asset.path)
                    .spawn()?;
            }

            #[cfg(target_os = "linux")]
            {
                std::process::Command::new("xdg-open")
                    .arg(&asset.path)
                    .spawn()?;
            }
        }
        Ok(())
    }

    /// Rebuild assets list for the current ticket
    pub fn rebuild_assets(&mut self) {
        self.assets_selection = 0;
        if let Some(pea) = self.selected_pea() {
            // Get project root from data_path (parent of .peas)
            if let Some(project_root) = self.data_path.parent() {
                let asset_manager = crate::assets::AssetManager::new(project_root);
                match asset_manager.list_assets(&pea.id) {
                    Ok(assets) => {
                        self.assets_items = assets;
                    }
                    Err(_) => {
                        self.assets_items.clear();
                    }
                }
            } else {
                self.assets_items.clear();
            }
        } else {
            self.assets_items.clear();
        }
    }

    /// Toggle between detail view panes (Metadata -> Body -> Relations -> Assets -> Metadata)
    pub fn toggle_detail_pane(&mut self) {
        self.detail_pane = match self.detail_pane {
            DetailPane::Metadata => DetailPane::Body,
            DetailPane::Body => {
                if !self.relations_items.is_empty() {
                    DetailPane::Relations
                } else if !self.assets_items.is_empty() {
                    DetailPane::Assets
                } else {
                    DetailPane::Metadata
                }
            }
            DetailPane::Relations => {
                if !self.assets_items.is_empty() {
                    DetailPane::Assets
                } else {
                    DetailPane::Metadata
                }
            }
            DetailPane::Assets => DetailPane::Metadata,
        };
    }

    /// Returns the list of available statuses for the modal
    pub fn status_options() -> &'static [PeaStatus] {
        &[
            PeaStatus::Draft,
            PeaStatus::Todo,
            PeaStatus::InProgress,
            PeaStatus::Completed,
            PeaStatus::Scrapped,
        ]
    }

    /// Open the status modal with the current pea's status preselected
    pub fn open_status_modal(&mut self) {
        if let Some(pea) = self.selected_pea() {
            let options = Self::status_options();
            self.modal_selection = options.iter().position(|s| *s == pea.status).unwrap_or(0);
            self.previous_mode = self.input_mode;
            self.input_mode = InputMode::StatusModal;
        }
    }

    /// Apply the selected status from the modal (to all selected tickets)
    pub fn apply_modal_status(&mut self) -> Result<()> {
        let options = Self::status_options();
        if let Some(&new_status) = options.get(self.modal_selection) {
            let target_ids = self.target_ids();
            let message = modal_operations::apply_status_change(
                &target_ids,
                &self.all_peas,
                &self.repo,
                &self.data_path,
                new_status,
            )?;
            if !message.is_empty() {
                self.message = Some(message);
            }
            self.clear_multi_select();
            self.refresh()?;
        }
        self.input_mode = self.previous_mode;
        Ok(())
    }

    /// Returns the list of available priorities for the modal
    pub fn priority_options() -> &'static [PeaPriority] {
        &[
            PeaPriority::Critical,
            PeaPriority::High,
            PeaPriority::Normal,
            PeaPriority::Low,
            PeaPriority::Deferred,
        ]
    }

    /// Open the priority modal with the current pea's priority preselected
    pub fn open_priority_modal(&mut self) {
        if let Some(pea) = self.selected_pea() {
            let options = Self::priority_options();
            self.modal_selection = options.iter().position(|p| *p == pea.priority).unwrap_or(0);
            self.previous_mode = self.input_mode;
            self.input_mode = InputMode::PriorityModal;
        }
    }

    /// Apply the selected priority from the modal (to all selected tickets)
    pub fn apply_modal_priority(&mut self) -> Result<()> {
        let options = Self::priority_options();
        if let Some(&new_priority) = options.get(self.modal_selection) {
            let target_ids = self.target_ids();
            let message = modal_operations::apply_priority_change(
                &target_ids,
                &self.all_peas,
                &self.repo,
                &self.data_path,
                new_priority,
            )?;
            if !message.is_empty() {
                self.message = Some(message);
            }
            self.clear_multi_select();
            self.refresh()?;
        }
        self.input_mode = self.previous_mode;
        Ok(())
    }

    /// Returns the list of available types for the modal
    pub fn type_options() -> &'static [PeaType] {
        &[
            PeaType::Milestone,
            PeaType::Epic,
            PeaType::Story,
            PeaType::Feature,
            PeaType::Bug,
            PeaType::Chore,
            PeaType::Research,
            PeaType::Task,
        ]
    }

    /// Open the type modal with the current pea's type preselected
    pub fn open_type_modal(&mut self) {
        if let Some(pea) = self.selected_pea() {
            let options = Self::type_options();
            self.modal_selection = options.iter().position(|t| *t == pea.pea_type).unwrap_or(0);
            self.previous_mode = self.input_mode;
            self.input_mode = InputMode::TypeModal;
        }
    }

    /// Apply the selected type from the modal (to all selected tickets)
    pub fn apply_modal_type(&mut self) -> Result<()> {
        let options = Self::type_options();
        if let Some(&new_type) = options.get(self.modal_selection) {
            let target_ids = self.target_ids();
            let message = modal_operations::apply_type_change(
                &target_ids,
                &self.all_peas,
                &self.repo,
                &self.data_path,
                new_type,
            )?;
            if !message.is_empty() {
                self.message = Some(message);
            }
            self.clear_multi_select();
            self.refresh()?;
        }
        self.input_mode = self.previous_mode;
        Ok(())
    }

    /// Open the tags modal with the current pea's tags
    pub fn open_tags_modal(&mut self) {
        if let Some(pea) = self.selected_pea() {
            // Convert tags vec to comma-separated string
            self.tags_input = pea.tags.join(", ");
            self.previous_mode = self.input_mode;
            self.input_mode = InputMode::TagsModal;
        }
    }

    /// Apply the tags from the modal
    pub fn apply_tags_modal(&mut self) -> Result<()> {
        if let Some(pea) = self.selected_pea().cloned() {
            // Parse comma-separated tags, trim whitespace, filter empty
            let new_tags: Vec<String> = self
                .tags_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            modal_operations::apply_tags_change(
                &pea.id,
                &self.all_peas,
                &self.repo,
                &self.data_path,
                new_tags,
            )?;

            self.message = Some("Tags updated".to_string());
            self.refresh()?;
        }
        self.input_mode = self.previous_mode;
        Ok(())
    }

    /// Open delete confirmation dialog
    pub fn open_delete_confirm(&mut self) {
        match self.view_mode {
            ViewMode::Tickets => {
                if self.selected_pea().is_some() {
                    self.input_mode = InputMode::DeleteConfirm;
                }
            }
            ViewMode::Memory => {
                if self.selected_index < self.filtered_memories.len() {
                    self.input_mode = InputMode::DeleteConfirm;
                }
            }
        }
    }

    /// Delete the currently selected pea or memory
    pub fn delete_selected(&mut self) -> Result<()> {
        match self.view_mode {
            ViewMode::Tickets => {
                if let Some(pea) = self.selected_pea().cloned() {
                    // Record undo before delete
                    let undo_manager = UndoManager::new(&self.data_path);
                    if let Ok(path) = self.repo.find_file_by_id(&pea.id) {
                        let _ = crate::undo::record_delete(&undo_manager, &pea.id, &path);
                    }

                    self.repo.delete(&pea.id)?;
                    self.message = Some(format!("Deleted {}", pea.id));
                    self.refresh()?;
                }
            }
            ViewMode::Memory => {
                if let Some(memory) = self.filtered_memories.get(self.selected_index).cloned() {
                    self.memory_repo.delete(&memory.key)?;
                    self.message = Some(format!("Deleted memory '{}'", memory.key));
                    self.refresh()?;

                    // Adjust selection if needed
                    if self.selected_index >= self.filtered_memories.len()
                        && self.selected_index > 0
                    {
                        self.selected_index -= 1;
                    }
                }
            }
        }
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    /// Open the parent selection modal
    /// Shows only tickets that can be valid parents (milestones, epics, stories, features)
    pub fn open_parent_modal(&mut self) {
        let current_info = self
            .selected_pea()
            .map(|p| (p.id.clone(), p.parent.clone()));

        if let Some((current_id, current_parent)) = current_info {
            // Build list of potential parents:
            // - Milestones, Epics, Stories, Features can be parents
            // - Can't be self or descendants of current
            // - First option is "(none)" to clear parent
            self.parent_candidates = self
                .all_peas
                .iter()
                .filter(|p| {
                    // Can't be self
                    if p.id == current_id {
                        return false;
                    }
                    // Only container types can be parents
                    matches!(
                        p.pea_type,
                        PeaType::Milestone | PeaType::Epic | PeaType::Story | PeaType::Feature
                    )
                })
                .cloned()
                .collect();

            // Sort by type hierarchy, then title
            self.parent_candidates.sort_by(|a, b| {
                fn type_order(t: &PeaType) -> u8 {
                    match t {
                        PeaType::Milestone => 0,
                        PeaType::Epic => 1,
                        PeaType::Story => 2,
                        PeaType::Feature => 3,
                        _ => 4,
                    }
                }
                type_order(&a.pea_type)
                    .cmp(&type_order(&b.pea_type))
                    .then_with(|| a.title.cmp(&b.title))
            });

            // Find current parent's position, or default to 0 (which will be "none")
            self.modal_selection = if let Some(ref parent_id) = current_parent {
                self.parent_candidates
                    .iter()
                    .position(|p| p.id == *parent_id)
                    .map(|i| i + 1) // +1 because index 0 is "(none)"
                    .unwrap_or(0)
            } else {
                0 // No parent = "(none)" selected
            };

            self.previous_mode = self.input_mode;
            self.input_mode = InputMode::ParentModal;
        }
    }

    /// Apply the selected parent from the modal
    pub fn apply_modal_parent(&mut self) -> Result<()> {
        let new_parent = if self.modal_selection == 0 {
            None // "(none)" selected
        } else {
            self.parent_candidates
                .get(self.modal_selection - 1)
                .map(|p| p.id.clone())
        };

        if let Some(pea) = self.selected_pea().cloned() {
            let message = modal_operations::apply_parent_change(
                &pea.id,
                &self.all_peas,
                &self.repo,
                &self.data_path,
                new_parent,
            )?;
            if !message.is_empty() {
                self.message = Some(message);
            }
            self.refresh()?;
        }
        self.input_mode = self.previous_mode;
        Ok(())
    }

    /// Open the blocking selection modal (multi-select)
    pub fn open_blocking_modal(&mut self) {
        let current_info = self
            .selected_pea()
            .map(|p| (p.id.clone(), p.blocking.clone()));

        if let Some((current_id, current_blocking)) = current_info {
            // Build list of all tickets that could be blockers (any ticket except self)
            self.blocking_candidates = self
                .all_peas
                .iter()
                .filter(|p| p.id != current_id)
                .cloned()
                .collect();

            // Sort by status (open first), then type, then title
            self.blocking_candidates.sort_by(|a, b| {
                fn status_order(s: &PeaStatus) -> u8 {
                    match s {
                        PeaStatus::InProgress => 0,
                        PeaStatus::Todo => 1,
                        PeaStatus::Draft => 2,
                        PeaStatus::Completed => 3,
                        PeaStatus::Scrapped => 4,
                    }
                }
                status_order(&a.status)
                    .cmp(&status_order(&b.status))
                    .then_with(|| a.title.cmp(&b.title))
            });

            // Initialize selection state based on current blocking list
            self.blocking_selected = self
                .blocking_candidates
                .iter()
                .map(|p| current_blocking.contains(&p.id))
                .collect();

            self.modal_selection = 0;
            self.previous_mode = self.input_mode;
            self.input_mode = InputMode::BlockingModal;
        }
    }

    /// Toggle selection of current item in blocking modal
    pub fn toggle_blocking_selection(&mut self) {
        if let Some(selected) = self.blocking_selected.get_mut(self.modal_selection) {
            *selected = !*selected;
        }
    }

    /// Apply the selected blockers from the modal
    pub fn apply_modal_blocking(&mut self) -> Result<()> {
        let new_blocking: Vec<String> = self
            .blocking_candidates
            .iter()
            .zip(self.blocking_selected.iter())
            .filter_map(
                |(pea, &selected)| {
                    if selected { Some(pea.id.clone()) } else { None }
                },
            )
            .collect();

        if let Some(pea) = self.selected_pea().cloned() {
            let message = modal_operations::apply_blocking_change(
                &pea.id,
                &self.all_peas,
                &self.repo,
                &self.data_path,
                new_blocking,
            )?;
            if !message.is_empty() {
                self.message = Some(message);
            }
            self.refresh()?;
        }
        self.input_mode = self.previous_mode;
        Ok(())
    }

    /// Open the create ticket modal
    pub fn open_create_modal(&mut self) {
        self.create_title.clear();
        self.create_type = PeaType::Task;
        self.modal_selection = 0; // 0 = title field, 1 = type field
        self.input_mode = InputMode::CreateModal;
    }

    /// Create a new ticket from the modal inputs
    pub fn create_from_modal(&mut self) -> Result<()> {
        if self.create_title.trim().is_empty() {
            self.message = Some("Title cannot be empty".to_string());
            return Ok(());
        }

        // If current selection is a container type, use it as parent
        let parent = self.selected_pea().and_then(|p| {
            if matches!(
                p.pea_type,
                PeaType::Milestone | PeaType::Epic | PeaType::Story | PeaType::Feature
            ) {
                Some(p.id.clone())
            } else {
                None
            }
        });

        let id = self.repo.generate_id()?;
        let pea = crate::model::Pea::new(
            id.clone(),
            self.create_title.trim().to_string(),
            self.create_type,
        )
        .with_parent(parent);

        let path = self.repo.create(&pea)?;

        // Record undo after create
        let undo_manager = UndoManager::new(&self.data_path);
        let _ = crate::undo::record_create(&undo_manager, &id, &path);

        self.message = Some(format!("Created {}", id));
        self.refresh()?;
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    /// Open the memory creation modal
    pub fn open_memory_create_modal(&mut self) {
        self.memory_create_key.clear();
        self.memory_create_tags.clear();
        self.memory_create_content.clear();
        self.memory_modal_selection = 0; // 0 = key field, 1 = tags field, 2 = content field
        self.input_mode = InputMode::MemoryCreateModal;
    }

    /// Create a new memory from the modal inputs
    pub fn create_memory_from_modal(&mut self) -> Result<()> {
        let key = self.memory_create_key.trim();

        // Validate key
        if key.is_empty() {
            self.message = Some("Key cannot be empty".to_string());
            return Ok(());
        }

        // Validate key for filename safety (no path separators, no special chars)
        if key.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
            self.message = Some("Key contains invalid characters".to_string());
            return Ok(());
        }

        // Check if memory already exists
        if self.memory_repo.get(key).is_ok() {
            self.message = Some(format!("Memory '{}' already exists", key));
            return Ok(());
        }

        // Parse tags (comma-separated)
        let tags: Vec<String> = self
            .memory_create_tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Create memory
        let memory = crate::model::Memory::new(key.to_string())
            .with_tags(tags)
            .with_content(self.memory_create_content.clone());

        self.memory_repo.create(&memory)?;

        self.message = Some(format!("Created memory '{}'", key));
        self.refresh()?;
        self.input_mode = InputMode::Normal;
        Ok(())
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> Result<()> {
        let undo_manager = UndoManager::new(&self.data_path);
        match undo_manager.undo() {
            Ok(msg) => {
                self.message = Some(format!("Undo: {}", msg));
                self.refresh()?;
            }
            Err(e) => {
                self.message = Some(format!("Nothing to undo: {}", e));
            }
        }
        Ok(())
    }

    /// Get the number of operations that can be undone
    pub fn undo_count(&self) -> usize {
        let undo_manager = UndoManager::new(&self.data_path);
        undo_manager.undo_count()
    }

    /// Open URL modal showing all URLs found in ticket body
    pub fn open_url_modal(&mut self) {
        if let Some(pea) = self.selected_pea() {
            self.url_candidates = url_utils::extract_urls(&pea.body);
            if !self.url_candidates.is_empty() {
                self.modal_selection = 0;
                self.previous_mode = self.input_mode;
                self.input_mode = InputMode::UrlModal;
            } else {
                self.message = Some("No URLs found in ticket body".to_string());
            }
        }
    }

    /// Open selected URL from modal
    pub fn open_selected_url(&mut self) -> Result<()> {
        if let Some(url) = self.url_candidates.get(self.modal_selection) {
            match open::that(url) {
                Ok(_) => {
                    self.message = Some(format!("Opening: {}", url));
                }
                Err(e) => {
                    self.message = Some(format!("Failed to open URL: {}", e));
                }
            }
        }
        self.input_mode = self.previous_mode;
        Ok(())
    }

    /// Start editing body inline with TextArea
    pub fn start_body_edit(&mut self) {
        if let Some(pea) = self.selected_pea() {
            self.body_textarea = Some(body_editor::create_textarea(&pea.body));
            self.input_mode = InputMode::EditBody;
            self.detail_pane = DetailPane::Body; // Force Body pane focus
        }
    }

    /// Save body edit and update the pea
    pub fn save_body_edit(&mut self) -> Result<()> {
        if let (Some(textarea), Some(pea)) = (&self.body_textarea, self.selected_pea().cloned()) {
            body_editor::save_body(textarea, &pea, &self.repo, &self.data_path)?;

            // Cleanup
            self.body_textarea = None;
            self.input_mode = InputMode::DetailView;
            self.refresh()?;
        }
        Ok(())
    }

    /// Cancel body edit without saving
    pub fn cancel_body_edit(&mut self) {
        self.body_textarea = None;
        self.input_mode = InputMode::DetailView;
    }
}

pub fn run_tui(config: PeasConfig, project_root: PathBuf) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(&config, &project_root)?;

    // Set up file watcher for .peas directory
    let (fs_tx, fs_rx) = mpsc::channel();
    let peas_dir = config.data_path(&project_root);

    // Create debounced watcher (300ms debounce)
    let mut debouncer = new_debouncer(Duration::from_millis(300), fs_tx)?;
    debouncer
        .watcher()
        .watch(&peas_dir, RecursiveMode::Recursive)?;

    let res = run_app(&mut terminal, &mut app, fs_rx);

    // Stop watching (debouncer dropped automatically)
    drop(debouncer);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    fs_rx: mpsc::Receiver<
        std::result::Result<
            Vec<notify_debouncer_mini::DebouncedEvent>,
            notify_debouncer_mini::notify::Error,
        >,
    >,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        // Check for file system events (non-blocking)
        if let Ok(Ok(_events)) = fs_rx.try_recv() {
            // Files changed - refresh the list
            let _ = app.refresh();
            app.message = Some("Files changed - refreshed".to_string());
            continue;
        }

        // Poll for keyboard events with a short timeout
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let event = event::read()?;

        match event {
            Event::Mouse(mouse_event) => {
                handlers::mouse::handle_mouse(app, mouse_event);
                continue;
            }
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                let should_quit = match app.input_mode {
                    InputMode::Normal => {
                        handlers::normal_mode::handle_normal_mode(app, key, terminal)?
                    }
                    InputMode::Filter => handlers::filter::handle_filter_mode(app, key)?,
                    InputMode::StatusModal => {
                        handlers::modal_status::handle_status_modal(app, key)?
                    }
                    InputMode::PriorityModal => {
                        handlers::modal_priority::handle_priority_modal(app, key)?
                    }
                    InputMode::TypeModal => handlers::modal_type::handle_type_modal(app, key)?,
                    InputMode::DeleteConfirm => {
                        handlers::modal_delete::handle_delete_confirm(app, key)?
                    }
                    InputMode::ParentModal => {
                        handlers::modal_parent::handle_parent_modal(app, key)?
                    }
                    InputMode::BlockingModal => {
                        handlers::modal_blocking::handle_blocking_modal(app, key)?
                    }
                    InputMode::DetailView => {
                        handlers::detail_view::handle_detail_view(app, key, terminal)?
                    }
                    InputMode::CreateModal => {
                        handlers::modal_create::handle_create_modal(app, key)?
                    }
                    InputMode::MemoryCreateModal => {
                        handlers::modal_memory_create::handle_memory_create_modal(app, key)?
                    }
                    InputMode::EditBody => handlers::edit_body::handle_edit_body(app, key)?,
                    InputMode::TagsModal => handlers::modal_tags::handle_tags_modal(app, key)?,
                    InputMode::UrlModal => handlers::modal_url::handle_url_modal(app, key)?,
                };

                if should_quit {
                    return Ok(());
                }

                // Clear message after any key press
                if app.message.is_some() && key.code != KeyCode::Enter {
                    app.message = None;
                }
            }
            _ => {}
        }
    }
}
