use super::ui;
use crate::{
    config::PeasConfig,
    error::Result,
    model::{Memory, Pea, PeaPriority, PeaStatus, PeaType},
    storage::{MemoryRepository, PeaRepository},
    undo::UndoManager,
};
use cli_clipboard::ClipboardProvider;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};
use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};
use tui_textarea::{Input, TextArea};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Tickets, // Ticket tree view
    Memory,  // Memory list view
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filter,
    StatusModal,
    PriorityModal,
    TypeModal,
    DeleteConfirm,
    ParentModal,
    BlockingModal,
    DetailView,
    CreateModal,
    EditBody,  // Inline body editing with textarea
    TagsModal, // Tag editing modal
    UrlModal,  // URL selection modal
}

/// Which pane is focused in detail view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailPane {
    Metadata, // Ticket properties/metadata
    #[default]
    Body, // Description/markdown content
    Relations, // Relationships pane
}

/// A node in the tree view representing a pea and its depth
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub pea: Pea,
    pub depth: usize,
    pub is_last: bool,           // Is this the last child at this level?
    pub parent_lines: Vec<bool>, // Which parent levels need continuing lines
}

/// Layer 2: Page table entry with references to tree nodes
#[derive(Debug, Clone)]
pub struct PageInfo {
    pub start_index: usize, // Starting index in tree_nodes for regular items
    pub item_count: usize,  // Number of actual items on this page
    pub parent_indices: Vec<usize>, // Indices of parent context nodes to show (top-down order)
}

pub struct App {
    pub view_mode: ViewMode, // Current view (Tickets or Memory)
    pub repo: PeaRepository,
    pub memory_repo: MemoryRepository,
    pub data_path: PathBuf, // Path to .peas data directory
    pub all_peas: Vec<Pea>,
    pub filtered_peas: Vec<Pea>,
    pub all_memories: Vec<Memory>,      // All memories
    pub filtered_memories: Vec<Memory>, // Filtered memories
    pub tree_nodes: Vec<TreeNode>,      // Flattened tree for display
    pub page_table: Vec<PageInfo>,      // Virtual page table accounting for parent rows
    pub selected_index: usize,          // Global index in tree_nodes
    pub page_height: usize,             // Number of items that fit on one page
    pub list_state: ListState,
    pub detail_scroll: u16,         // Scroll offset for body/description
    pub detail_max_scroll: u16,     // Maximum scroll offset (0 means no scrolling)
    pub relations_scroll: u16,      // Scroll offset for relationships pane (future use)
    pub relations_selection: usize, // Selected item in relationships pane
    pub relations_items: Vec<(String, String, String, PeaType)>, // (rel_type, id, title, pea_type) for relationships
    pub metadata_selection: usize, // Selected property in metadata pane (0=type, 1=status, 2=priority, 3=tags)
    pub detail_pane: DetailPane,   // Which pane is focused in detail view
    pub input_mode: InputMode,
    pub previous_mode: InputMode, // Mode to return to after closing modal
    pub search_query: String,
    pub show_help: bool,
    pub message: Option<String>,
    pub modal_selection: usize,      // Current selection in modal dialogs
    pub parent_candidates: Vec<Pea>, // Candidates for parent selection modal
    pub blocking_candidates: Vec<Pea>, // Candidates for blocking selection modal
    pub blocking_selected: Vec<bool>, // Which candidates are selected (multi-select)
    pub create_title: String,        // Title input for create modal
    pub create_type: PeaType,        // Type selection for create modal
    pub tags_input: String,          // Tag input for tags modal
    pub multi_selected: HashSet<String>, // IDs of multi-selected tickets
    pub body_textarea: Option<TextArea<'static>>, // TextArea for body editing
    pub start_time: Instant,         // App start time for pulsing effects
    pub url_candidates: Vec<String>, // URLs found in current ticket
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

    /// Build a flattened tree structure from the filtered peas
    pub fn build_tree(&mut self) {
        use std::collections::{HashMap, HashSet};

        self.tree_nodes.clear();

        // Build a set of IDs that exist in filtered_peas for quick lookup
        let filtered_ids: HashSet<String> =
            self.filtered_peas.iter().map(|p| p.id.clone()).collect();

        // Build a map of parent -> children
        let mut children_map: HashMap<Option<String>, Vec<&Pea>> = HashMap::new();
        for pea in &self.filtered_peas {
            // If the pea has a parent but that parent is not in the filtered set,
            // treat it as a root item (orphaned)
            let effective_parent = if let Some(ref parent_id) = pea.parent {
                if filtered_ids.contains(parent_id) {
                    pea.parent.clone()
                } else {
                    None // Parent not in filtered set, show as root
                }
            } else {
                None
            };

            children_map.entry(effective_parent).or_default().push(pea);
        }

        // Sort children by status (in-progress first, then todo, then completed) then by type hierarchy
        fn status_order(status: &PeaStatus) -> u8 {
            match status {
                PeaStatus::InProgress => 0,
                PeaStatus::Todo => 1,
                PeaStatus::Draft => 2,
                PeaStatus::Completed => 3,
                PeaStatus::Scrapped => 4,
            }
        }

        fn type_order(pea_type: &PeaType) -> u8 {
            match pea_type {
                PeaType::Milestone => 0,
                PeaType::Epic => 1,
                PeaType::Story => 2,
                PeaType::Feature => 3,
                PeaType::Bug => 4,
                PeaType::Chore => 5,
                PeaType::Research => 6,
                PeaType::Task => 7,
            }
        }

        for children in children_map.values_mut() {
            children.sort_by(|a, b| {
                status_order(&a.status)
                    .cmp(&status_order(&b.status))
                    .then_with(|| type_order(&a.pea_type).cmp(&type_order(&b.pea_type)))
                    .then_with(|| a.title.cmp(&b.title))
            });
        }

        // Recursively build tree nodes
        fn add_children(
            parent_id: Option<String>,
            depth: usize,
            parent_lines: Vec<bool>,
            children_map: &HashMap<Option<String>, Vec<&Pea>>,
            nodes: &mut Vec<TreeNode>,
        ) {
            if let Some(children) = children_map.get(&parent_id) {
                let count = children.len();
                for (i, pea) in children.iter().enumerate() {
                    let is_last = i == count - 1;
                    let mut current_parent_lines = parent_lines.clone();

                    nodes.push(TreeNode {
                        pea: (*pea).clone(),
                        depth,
                        is_last,
                        parent_lines: current_parent_lines.clone(),
                    });

                    // For children, add whether this level continues
                    // But only track continuation lines for depth > 0 (not for root items)
                    if depth > 0 {
                        current_parent_lines.push(!is_last);
                    }
                    add_children(
                        Some(pea.id.clone()),
                        depth + 1,
                        current_parent_lines,
                        children_map,
                        nodes,
                    );
                }
            }
        }

        // Start with root nodes (no parent or orphaned items)
        add_children(None, 0, Vec::new(), &children_map, &mut self.tree_nodes);
    }

    /// Build a virtual page table that accounts for parent context rows
    pub fn build_page_table(&mut self) {
        self.page_table.clear();

        if self.tree_nodes.is_empty() || self.page_height == 0 {
            return;
        }

        let mut current_index = 0;
        while current_index < self.tree_nodes.len() {
            // Get parent context indices for this page
            let parent_indices = self.get_parent_indices_at(current_index);
            let parent_count = parent_indices.len();

            // Calculate how many items can fit on this page
            let available_slots = self.page_height.saturating_sub(parent_count).max(1);
            let remaining_items = self.tree_nodes.len() - current_index;
            let item_count = available_slots.min(remaining_items);

            self.page_table.push(PageInfo {
                start_index: current_index,
                item_count,
                parent_indices,
            });

            current_index += item_count;
        }
    }

    /// Get parent context indices for a given start index (top-down order)
    fn get_parent_indices_at(&self, start_index: usize) -> Vec<usize> {
        if start_index >= self.tree_nodes.len() {
            return Vec::new();
        }

        let first_node = &self.tree_nodes[start_index];
        if let Some(parent_id) = &first_node.pea.parent {
            // Build the parent chain (stores indices)
            let mut parent_indices = Vec::new();
            let mut current_parent_id = Some(parent_id.clone());

            while let Some(pid) = current_parent_id {
                if let Some(parent_index) = self.tree_nodes.iter().position(|n| n.pea.id == pid) {
                    // Check if this parent would be visible in items starting from start_index
                    // A parent is visible if it appears at or after start_index
                    if parent_index >= start_index {
                        // Parent is on or after this page, no need for context
                        break;
                    }
                    parent_indices.push(parent_index);
                    current_parent_id = self.tree_nodes[parent_index].pea.parent.clone();
                } else {
                    break;
                }
            }

            // Reverse to get top-down order (root ancestor first)
            parent_indices.reverse();
            return parent_indices;
        }
        Vec::new()
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
        self.filtered_peas = self
            .all_peas
            .iter()
            .filter(|p| {
                // Search filter (searches in id, title, body, and tags)
                if self.search_query.is_empty() {
                    true
                } else {
                    let query = self.search_query.to_lowercase();
                    p.title.to_lowercase().contains(&query)
                        || p.id.to_lowercase().contains(&query)
                        || p.body.to_lowercase().contains(&query)
                        || p.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
                }
            })
            .cloned()
            .collect();

        if self.selected_index >= self.filtered_peas.len() {
            self.selected_index = self.filtered_peas.len().saturating_sub(1);
        }

        // Rebuild tree after filter changes
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
        self.relations_items.clear();
        self.relations_selection = 0;
        self.relations_scroll = 0;

        if let Some(pea) = self.selected_pea().cloned() {
            // Add parent if exists
            if let Some(ref parent_id) = pea.parent {
                if let Some(parent) = self.all_peas.iter().find(|p| p.id == *parent_id) {
                    self.relations_items.push((
                        "Parent".to_string(),
                        parent.id.clone(),
                        parent.title.clone(),
                        parent.pea_type,
                    ));
                }
            }

            // Add blocking tickets
            for id in &pea.blocking {
                if let Some(blocked) = self.all_peas.iter().find(|p| p.id == *id) {
                    self.relations_items.push((
                        "Blocks".to_string(),
                        blocked.id.clone(),
                        blocked.title.clone(),
                        blocked.pea_type,
                    ));
                }
            }

            // Add children
            let children: Vec<_> = self
                .all_peas
                .iter()
                .filter(|p| p.parent.as_ref() == Some(&pea.id))
                .collect();
            for child in children {
                self.relations_items.push((
                    "Child".to_string(),
                    child.id.clone(),
                    child.title.clone(),
                    child.pea_type,
                ));
            }

            // Add blocked-by (reverse blocking relationships)
            let blocked_by: Vec<_> = self
                .all_peas
                .iter()
                .filter(|p| p.blocking.contains(&pea.id))
                .collect();
            for blocker in blocked_by {
                self.relations_items.push((
                    "BlockedBy".to_string(),
                    blocker.id.clone(),
                    blocker.title.clone(),
                    blocker.pea_type,
                ));
            }
        }
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

    /// Toggle between detail view panes (Metadata -> Body -> Relations -> Metadata)
    pub fn toggle_detail_pane(&mut self) {
        self.detail_pane = match self.detail_pane {
            DetailPane::Metadata => DetailPane::Body,
            DetailPane::Body => {
                if !self.relations_items.is_empty() {
                    DetailPane::Relations
                } else {
                    DetailPane::Metadata
                }
            }
            DetailPane::Relations => DetailPane::Metadata,
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
            let count = target_ids.len();
            let undo_manager = UndoManager::new(&self.data_path);
            for (i, id) in target_ids.iter().enumerate() {
                if let Some(pea) = self.all_peas.iter().find(|p| p.id == *id).cloned() {
                    // Record undo for the last item (will be what gets undone)
                    if i == count - 1 {
                        if let Ok(path) = self.repo.find_file_by_id(&pea.id) {
                            let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
                        }
                    }
                    let mut updated = pea;
                    updated.status = new_status;
                    updated.touch();
                    self.repo.update(&updated)?;
                }
            }
            if count > 1 {
                self.message = Some(format!("{} tickets -> {}", count, new_status));
            } else if count == 1 {
                self.message = Some(format!("-> {}", new_status));
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
            let count = target_ids.len();
            let undo_manager = UndoManager::new(&self.data_path);
            for (i, id) in target_ids.iter().enumerate() {
                if let Some(pea) = self.all_peas.iter().find(|p| p.id == *id).cloned() {
                    // Record undo for the last item
                    if i == count - 1 {
                        if let Ok(path) = self.repo.find_file_by_id(&pea.id) {
                            let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
                        }
                    }
                    let mut updated = pea;
                    updated.priority = new_priority;
                    updated.touch();
                    self.repo.update(&updated)?;
                }
            }
            if count > 1 {
                self.message = Some(format!("{} tickets -> {}", count, new_priority));
            } else if count == 1 {
                self.message = Some(format!("-> {}", new_priority));
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
            let count = target_ids.len();
            let undo_manager = UndoManager::new(&self.data_path);
            for (i, id) in target_ids.iter().enumerate() {
                if let Some(pea) = self.all_peas.iter().find(|p| p.id == *id).cloned() {
                    // Record undo for the last item
                    if i == count - 1 {
                        if let Ok(path) = self.repo.find_file_by_id(&pea.id) {
                            let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
                        }
                    }
                    let mut updated = pea;
                    updated.pea_type = new_type;
                    updated.touch();
                    self.repo.update(&updated)?;
                }
            }
            if count > 1 {
                self.message = Some(format!("{} tickets -> {}", count, new_type));
            } else if count == 1 {
                self.message = Some(format!("-> {}", new_type));
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

            // Record undo before update
            let undo_manager = UndoManager::new(&self.data_path);
            if let Ok(path) = self.repo.find_file_by_id(&pea.id) {
                let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
            }

            // Update pea
            let mut updated = pea;
            updated.tags = new_tags;
            updated.touch();
            self.repo.update(&updated)?;

            self.message = Some("Tags updated".to_string());
            self.refresh()?;
        }
        self.input_mode = self.previous_mode;
        Ok(())
    }

    /// Open delete confirmation dialog
    pub fn open_delete_confirm(&mut self) {
        if self.selected_pea().is_some() {
            self.input_mode = InputMode::DeleteConfirm;
        }
    }

    /// Delete the currently selected pea
    pub fn delete_selected(&mut self) -> Result<()> {
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
            // Record undo before update
            let undo_manager = UndoManager::new(&self.data_path);
            if let Ok(path) = self.repo.find_file_by_id(&pea.id) {
                let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
            }

            let mut updated = pea.clone();
            updated.parent = new_parent.clone();
            updated.touch();
            self.repo.update(&updated)?;

            let parent_display = new_parent.unwrap_or_else(|| "(none)".to_string());
            self.message = Some(format!("{} parent -> {}", pea.id, parent_display));
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
            // Record undo before update
            let undo_manager = UndoManager::new(&self.data_path);
            if let Ok(path) = self.repo.find_file_by_id(&pea.id) {
                let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
            }

            let mut updated = pea.clone();
            updated.blocking = new_blocking.clone();
            updated.touch();
            self.repo.update(&updated)?;

            let count = new_blocking.len();
            self.message = Some(format!("{} blocking {} tickets", pea.id, count));
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

        let id = self.repo.generate_id();
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

    /// Extract all URLs from ticket body with smart punctuation handling
    fn extract_urls(text: &str) -> Vec<String> {
        let mut urls = Vec::new();

        // Find potential URLs with regex
        let url_pattern = regex::Regex::new(r"https?://[^\s<>]+").unwrap();

        for matched in url_pattern.find_iter(text) {
            let mut url_str = matched.as_str();

            // Trim trailing punctuation that's likely not part of the URL
            // Common cases: "Check out https://example.com." or "(see https://example.com)"
            while !url_str.is_empty() {
                let last_char = url_str.chars().last().unwrap();
                if matches!(
                    last_char,
                    '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '\'' | '"'
                ) {
                    // Check if this is actually part of the URL or sentence punctuation
                    // If removing it still gives a valid URL, it was probably sentence punctuation
                    let trimmed = &url_str[..url_str.len() - last_char.len_utf8()];
                    if url::Url::parse(trimmed).is_ok() {
                        url_str = trimmed;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            // Validate and add if it's a proper URL
            if url::Url::parse(url_str).is_ok() {
                urls.push(url_str.to_string());
            }
        }

        // Deduplicate while preserving order
        let mut seen = HashSet::new();
        urls.retain(|url| seen.insert(url.clone()));

        urls
    }

    /// Open URL modal showing all URLs found in ticket body
    pub fn open_url_modal(&mut self) {
        if let Some(pea) = self.selected_pea() {
            self.url_candidates = Self::extract_urls(&pea.body);
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
        if let Some(pea) = self.selected_pea().cloned() {
            // Split body into lines for TextArea
            let lines: Vec<String> = pea.body.lines().map(|s| s.to_string()).collect();
            let mut textarea = TextArea::new(lines);

            // Configure textarea
            textarea.set_tab_length(2);
            textarea.set_max_histories(100); // Undo/redo buffer

            self.body_textarea = Some(textarea);
            self.input_mode = InputMode::EditBody;
            self.detail_pane = DetailPane::Body; // Force Body pane focus
        }
    }

    /// Save body edit and update the pea
    pub fn save_body_edit(&mut self) -> Result<()> {
        if let (Some(textarea), Some(pea)) = (&self.body_textarea, self.selected_pea().cloned()) {
            // Get edited content
            let new_body = textarea.lines().join("\n");

            // Record undo before update
            let undo_manager = UndoManager::new(&self.data_path);
            if let Ok(path) = self.repo.find_file_by_id(&pea.id) {
                let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
            }

            // Update pea
            let mut updated = pea;
            updated.body = new_body;
            updated.touch();
            self.repo.update(&updated)?;

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
    let peas_dir = project_root.join(&config.peas.path);

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

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('?') => app.show_help = !app.show_help,
                    KeyCode::Tab => {
                        app.switch_view();
                    }
                    KeyCode::Esc => {
                        if app.show_help {
                            app.show_help = false;
                        } else if !app.multi_selected.is_empty() {
                            app.clear_multi_select();
                        } else if !app.search_query.is_empty() {
                            app.search_query.clear();
                            app.apply_filter();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Right | KeyCode::PageDown | KeyCode::Char('J') => app.next_page(),
                    KeyCode::Left | KeyCode::PageUp | KeyCode::Char('K') => app.previous_page(),
                    KeyCode::Home | KeyCode::Char('g') => app.first(),
                    KeyCode::End | KeyCode::Char('G') => app.last(),
                    KeyCode::Char('/') => {
                        app.input_mode = InputMode::Filter;
                    }
                    KeyCode::Enter => {
                        match app.view_mode {
                            ViewMode::Tickets => {
                                // Open full-screen detail view for tickets
                                if app.selected_pea().is_some() {
                                    app.detail_scroll = 0;
                                    app.build_relations();
                                    app.input_mode = InputMode::DetailView;
                                }
                            }
                            ViewMode::Memory => {
                                // Open memory detail view
                                if app.selected_index < app.all_memories.len() {
                                    app.detail_scroll = 0;
                                    app.input_mode = InputMode::DetailView;
                                }
                            }
                        }
                    }
                    KeyCode::Char(' ') => {
                        app.toggle_multi_select();
                    }
                    KeyCode::Char('s') => {
                        app.open_status_modal();
                    }
                    KeyCode::Char('P') => {
                        app.open_priority_modal();
                    }
                    KeyCode::Char('t') => {
                        app.open_type_modal();
                    }
                    KeyCode::Char('p') => {
                        app.open_parent_modal();
                    }
                    KeyCode::Char('b') => {
                        app.open_blocking_modal();
                    }
                    KeyCode::Char('c') => {
                        app.open_create_modal();
                    }
                    KeyCode::Char('d') => {
                        app.open_delete_confirm();
                    }
                    KeyCode::Char('r') => {
                        let _ = app.refresh();
                        app.message = Some("Refreshed".to_string());
                    }
                    KeyCode::Char('y') => {
                        if let Some(pea) = app.selected_pea() {
                            let id = pea.id.clone();
                            if let Ok(mut ctx) = cli_clipboard::ClipboardContext::new() {
                                if ctx.set_contents(id.clone()).is_ok() {
                                    app.message = Some(format!("Copied: {}", id));
                                } else {
                                    app.message = Some("Failed to copy to clipboard".to_string());
                                }
                            } else {
                                app.message = Some("Clipboard not available".to_string());
                            }
                        }
                    }
                    KeyCode::Char('e') => {
                        if let Some(file_path) = app.selected_pea_file_path() {
                            // Leave alternate screen temporarily
                            disable_raw_mode()?;
                            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

                            // Get editor from environment
                            let editor = std::env::var("EDITOR")
                                .or_else(|_| std::env::var("VISUAL"))
                                .unwrap_or_else(|_| {
                                    if cfg!(windows) {
                                        "notepad".to_string()
                                    } else {
                                        "vi".to_string()
                                    }
                                });

                            // Spawn editor and wait
                            let status =
                                std::process::Command::new(&editor).arg(&file_path).status();

                            // Re-enter alternate screen
                            enable_raw_mode()?;
                            execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                            terminal.clear()?;

                            // Refresh and show result
                            let _ = app.refresh();
                            match status {
                                Ok(s) if s.success() => {
                                    app.message = Some("Editor closed".to_string());
                                }
                                Ok(_) => {
                                    app.message = Some("Editor exited with error".to_string());
                                }
                                Err(e) => {
                                    app.message = Some(format!("Failed to open editor: {}", e));
                                }
                            }
                        }
                    }
                    KeyCode::Char('u') => {
                        let _ = app.undo();
                    }
                    _ => {}
                },
                InputMode::Filter => match key.code {
                    KeyCode::Enter | KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.apply_filter();
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.apply_filter();
                    }
                    _ => {}
                },
                InputMode::StatusModal => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = app.previous_mode;
                    }
                    KeyCode::Enter => {
                        let _ = app.apply_modal_status();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let count = App::status_options().len();
                        app.modal_selection = (app.modal_selection + 1) % count;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let count = App::status_options().len();
                        app.modal_selection = if app.modal_selection == 0 {
                            count - 1
                        } else {
                            app.modal_selection - 1
                        };
                    }
                    _ => {}
                },
                InputMode::PriorityModal => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = app.previous_mode;
                    }
                    KeyCode::Enter => {
                        let _ = app.apply_modal_priority();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let count = App::priority_options().len();
                        app.modal_selection = (app.modal_selection + 1) % count;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let count = App::priority_options().len();
                        app.modal_selection = if app.modal_selection == 0 {
                            count - 1
                        } else {
                            app.modal_selection - 1
                        };
                    }
                    _ => {}
                },
                InputMode::TypeModal => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = app.previous_mode;
                    }
                    KeyCode::Enter => {
                        let _ = app.apply_modal_type();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let count = App::type_options().len();
                        app.modal_selection = (app.modal_selection + 1) % count;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let count = App::type_options().len();
                        app.modal_selection = if app.modal_selection == 0 {
                            count - 1
                        } else {
                            app.modal_selection - 1
                        };
                    }
                    _ => {}
                },
                InputMode::DeleteConfirm => match key.code {
                    KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                        let _ = app.delete_selected();
                    }
                    _ => {}
                },
                InputMode::ParentModal => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = app.previous_mode;
                    }
                    KeyCode::Enter => {
                        let _ = app.apply_modal_parent();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let count = app.parent_candidates.len() + 1; // +1 for "(none)"
                        app.modal_selection = (app.modal_selection + 1) % count;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let count = app.parent_candidates.len() + 1; // +1 for "(none)"
                        app.modal_selection = if app.modal_selection == 0 {
                            count - 1
                        } else {
                            app.modal_selection - 1
                        };
                    }
                    _ => {}
                },
                InputMode::BlockingModal => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = app.previous_mode;
                    }
                    KeyCode::Enter => {
                        let _ = app.apply_modal_blocking();
                    }
                    KeyCode::Char(' ') => {
                        app.toggle_blocking_selection();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let count = app.blocking_candidates.len();
                        if count > 0 {
                            app.modal_selection = (app.modal_selection + 1) % count;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let count = app.blocking_candidates.len();
                        if count > 0 {
                            app.modal_selection = if app.modal_selection == 0 {
                                count - 1
                            } else {
                                app.modal_selection - 1
                            };
                        }
                    }
                    _ => {}
                },
                InputMode::DetailView => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.input_mode = InputMode::Normal;
                        app.detail_pane = DetailPane::Body;
                    }
                    KeyCode::Tab => {
                        app.toggle_detail_pane();
                    }
                    KeyCode::Enter => {
                        // Open modal for selected metadata property or jump to relation
                        if app.detail_pane == DetailPane::Metadata {
                            match app.metadata_selection {
                                0 => app.open_type_modal(),     // Type
                                1 => app.open_status_modal(),   // Status
                                2 => app.open_priority_modal(), // Priority
                                3 => app.open_tags_modal(),     // Tags
                                _ => {}
                            }
                        } else if app.detail_pane == DetailPane::Relations
                            && !app.relations_items.is_empty()
                        {
                            app.jump_to_relation();
                        } else {
                            app.input_mode = InputMode::Normal;
                            app.detail_pane = DetailPane::Body;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => match app.detail_pane {
                        DetailPane::Metadata => {
                            // Navigate down through metadata properties (type, status, priority, tags)
                            if app.metadata_selection < 3 {
                                app.metadata_selection += 1;
                            }
                        }
                        DetailPane::Body => app.scroll_detail_down(),
                        DetailPane::Relations => app.relations_next(),
                    },
                    KeyCode::Up | KeyCode::Char('k') => match app.detail_pane {
                        DetailPane::Metadata => {
                            // Navigate up through metadata properties
                            if app.metadata_selection > 0 {
                                app.metadata_selection -= 1;
                            }
                        }
                        DetailPane::Body => app.scroll_detail_up(),
                        DetailPane::Relations => app.relations_previous(),
                    },
                    KeyCode::Char('J') => {
                        // Always scroll body
                        app.scroll_detail_down();
                    }
                    KeyCode::Char('K') => {
                        // Always scroll body
                        app.scroll_detail_up();
                    }
                    KeyCode::PageDown => {
                        for _ in 0..10 {
                            app.scroll_detail_down();
                        }
                    }
                    KeyCode::PageUp => {
                        for _ in 0..10 {
                            app.scroll_detail_up();
                        }
                    }
                    KeyCode::Char('e') => {
                        // Start inline editing
                        app.start_body_edit();
                    }
                    KeyCode::Char('E') => {
                        // External editor (uppercase E)
                        if let Some(file_path) = app.selected_pea_file_path() {
                            disable_raw_mode()?;
                            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

                            let editor = std::env::var("EDITOR")
                                .or_else(|_| std::env::var("VISUAL"))
                                .unwrap_or_else(|_| {
                                    if cfg!(windows) {
                                        "notepad".to_string()
                                    } else {
                                        "vi".to_string()
                                    }
                                });

                            let _ = std::process::Command::new(&editor).arg(&file_path).status();

                            enable_raw_mode()?;
                            execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                            terminal.clear()?;
                            let _ = app.refresh();
                            app.build_relations(); // Rebuild relations after edit
                        }
                    }
                    // Property editing hotkeys (same as normal mode)
                    KeyCode::Char('s') => {
                        app.open_status_modal();
                    }
                    KeyCode::Char('P') => {
                        app.open_priority_modal();
                    }
                    KeyCode::Char('t') => {
                        app.open_type_modal();
                    }
                    KeyCode::Char('p') => {
                        app.open_parent_modal();
                    }
                    KeyCode::Char('b') => {
                        app.open_blocking_modal();
                    }
                    KeyCode::Char('y') => {
                        // Copy ticket ID to clipboard
                        if let Some(pea) = app.selected_pea() {
                            let id = pea.id.clone();
                            if let Ok(mut ctx) = cli_clipboard::ClipboardContext::new() {
                                if ctx.set_contents(id.clone()).is_ok() {
                                    app.message = Some(format!("Copied: {}", id));
                                } else {
                                    app.message = Some("Failed to copy to clipboard".to_string());
                                }
                            } else {
                                app.message = Some("Clipboard not available".to_string());
                            }
                        }
                    }
                    KeyCode::Char('o') => {
                        // Open URL selection modal
                        app.open_url_modal();
                    }
                    _ => {}
                },
                InputMode::CreateModal => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Enter => {
                        let _ = app.create_from_modal();
                    }
                    KeyCode::Tab => {
                        // Toggle between title (0) and type (1) fields
                        app.modal_selection = (app.modal_selection + 1) % 2;
                    }
                    KeyCode::BackTab => {
                        app.modal_selection = if app.modal_selection == 0 { 1 } else { 0 };
                    }
                    KeyCode::Char(c) => {
                        if app.modal_selection == 0 {
                            // Title field - add character
                            app.create_title.push(c);
                        } else {
                            // Type field - cycle through types with space
                            // (handled below)
                        }
                    }
                    KeyCode::Backspace => {
                        if app.modal_selection == 0 {
                            app.create_title.pop();
                        }
                    }
                    KeyCode::Left | KeyCode::Right => {
                        if app.modal_selection == 1 {
                            // Cycle type
                            let types = App::type_options();
                            let current_idx = types
                                .iter()
                                .position(|t| *t == app.create_type)
                                .unwrap_or(0);
                            let new_idx = if key.code == KeyCode::Right {
                                (current_idx + 1) % types.len()
                            } else {
                                if current_idx == 0 {
                                    types.len() - 1
                                } else {
                                    current_idx - 1
                                }
                            };
                            app.create_type = types[new_idx];
                        }
                    }
                    _ => {}
                },
                InputMode::EditBody => match key.code {
                    KeyCode::Esc => {
                        app.cancel_body_edit();
                    }
                    KeyCode::Char('s')
                        if key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        if let Err(e) = app.save_body_edit() {
                            app.message = Some(format!("Save failed: {}", e));
                        } else {
                            app.message = Some("Saved successfully".to_string());
                        }
                    }
                    _ => {
                        // Pass all other events to textarea
                        if let Some(ref mut textarea) = app.body_textarea {
                            let event = Event::Key(key);
                            textarea.input(Input::from(event));
                        }
                    }
                },
                InputMode::TagsModal => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = app.previous_mode;
                    }
                    KeyCode::Enter => {
                        if let Err(e) = app.apply_tags_modal() {
                            app.message = Some(format!("Failed to update tags: {}", e));
                        }
                    }
                    KeyCode::Char(c) => {
                        app.tags_input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.tags_input.pop();
                    }
                    _ => {}
                },
                InputMode::UrlModal => match key.code {
                    KeyCode::Esc => {
                        app.input_mode = app.previous_mode;
                    }
                    KeyCode::Enter => {
                        let _ = app.open_selected_url();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let count = app.url_candidates.len();
                        if count > 0 {
                            app.modal_selection = (app.modal_selection + 1) % count;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let count = app.url_candidates.len();
                        if count > 0 {
                            app.modal_selection = if app.modal_selection == 0 {
                                count - 1
                            } else {
                                app.modal_selection - 1
                            };
                        }
                    }
                    _ => {}
                },
            }

            // Clear message after any key press
            if app.message.is_some() && key.code != KeyCode::Enter {
                app.message = None;
            }
        }
    }
}
