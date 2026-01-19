use super::ui;
use crate::{
    config::PeasConfig,
    error::Result,
    model::{Pea, PeaPriority, PeaStatus, PeaType},
    storage::PeaRepository,
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
    time::Duration,
};

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
}

/// A node in the tree view representing a pea and its depth
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub pea: Pea,
    pub depth: usize,
    pub is_last: bool,           // Is this the last child at this level?
    pub parent_lines: Vec<bool>, // Which parent levels need continuing lines
}

pub struct App {
    pub repo: PeaRepository,
    pub data_path: PathBuf, // Path to .peas data directory
    pub all_peas: Vec<Pea>,
    pub filtered_peas: Vec<Pea>,
    pub tree_nodes: Vec<TreeNode>, // Flattened tree for display
    pub selected_index: usize,     // Global index in tree_nodes
    pub page_height: usize,        // Number of items that fit on one page
    pub list_state: ListState,
    pub detail_scroll: u16, // Scroll offset for details view
    pub input_mode: InputMode,
    pub search_query: String,
    pub show_help: bool,
    pub message: Option<String>,
    pub modal_selection: usize,      // Current selection in modal dialogs
    pub parent_candidates: Vec<Pea>, // Candidates for parent selection modal
    pub blocking_candidates: Vec<Pea>, // Candidates for blocking selection modal
    pub blocking_selected: Vec<bool>, // Which candidates are selected (multi-select)
    pub create_title: String,        // Title input for create modal
    pub create_type: PeaType,        // Type selection for create modal
    pub multi_selected: HashSet<String>, // IDs of multi-selected tickets
}

impl App {
    pub fn new(config: &PeasConfig, project_root: &Path) -> Result<Self> {
        let repo = PeaRepository::new(config, project_root);
        let data_path = config.data_path(project_root);
        let all_peas = repo.list()?;
        let filtered_peas = all_peas.clone();

        let mut list_state = ListState::default();
        if !filtered_peas.is_empty() {
            list_state.select(Some(0));
        }

        let mut app = Self {
            repo,
            data_path,
            all_peas,
            filtered_peas,
            tree_nodes: Vec::new(),
            selected_index: 0,
            page_height: 20, // Default, updated when drawing
            list_state,
            detail_scroll: 0,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            show_help: false,
            message: None,
            modal_selection: 0,
            parent_candidates: Vec::new(),
            blocking_candidates: Vec::new(),
            blocking_selected: Vec::new(),
            create_title: String::new(),
            create_type: PeaType::Task,
            multi_selected: HashSet::new(),
        };
        app.build_tree();
        Ok(app)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.all_peas = self.repo.list()?;
        self.apply_filter();
        self.build_tree();
        Ok(())
    }

    /// Build a flattened tree structure from the filtered peas
    pub fn build_tree(&mut self) {
        use std::collections::HashMap;

        self.tree_nodes.clear();

        // Build a map of parent -> children
        let mut children_map: HashMap<Option<String>, Vec<&Pea>> = HashMap::new();
        for pea in &self.filtered_peas {
            children_map
                .entry(pea.parent.clone())
                .or_default()
                .push(pea);
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

        // Start with root nodes (no parent)
        add_children(None, 0, Vec::new(), &children_map, &mut self.tree_nodes);
    }

    /// Returns the number of items in the current view
    pub fn display_count(&self) -> usize {
        self.tree_nodes.len()
    }

    /// Returns the current page number (0-indexed)
    pub fn current_page(&self) -> usize {
        if self.page_height == 0 {
            0
        } else {
            self.selected_index / self.page_height
        }
    }

    /// Returns the total number of pages
    pub fn total_pages(&self) -> usize {
        if self.page_height == 0 {
            1
        } else {
            (self.display_count() + self.page_height - 1) / self.page_height
        }
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
    pub fn page_start(&self) -> usize {
        self.current_page() * self.page_height
    }

    /// Returns the items for the current page
    pub fn current_page_items(&self) -> &[TreeNode] {
        let start = self.page_start();
        let end = (start + self.page_height).min(self.tree_nodes.len());
        &self.tree_nodes[start..end]
    }

    pub fn apply_filter(&mut self) {
        self.filtered_peas = self
            .all_peas
            .iter()
            .filter(|p| {
                // Search filter only (filter tabs removed)
                if self.search_query.is_empty() {
                    true
                } else {
                    let query = self.search_query.to_lowercase();
                    p.title.to_lowercase().contains(&query)
                        || p.id.to_lowercase().contains(&query)
                        || p.body.to_lowercase().contains(&query)
                }
            })
            .cloned()
            .collect();

        if self.selected_index >= self.filtered_peas.len() {
            self.selected_index = self.filtered_peas.len().saturating_sub(1);
        }

        // Rebuild tree after filter changes
        self.build_tree();

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

    /// Jump to next page
    pub fn next_page(&mut self) {
        let count = self.display_count();
        if count > 0 && self.page_height > 0 {
            let next_page_start = (self.current_page() + 1) * self.page_height;
            if next_page_start < count {
                self.selected_index = next_page_start;
            } else {
                // Go to last item if no more pages
                self.selected_index = count - 1;
            }
            self.list_state.select(Some(self.index_in_page()));
            self.detail_scroll = 0;
        }
    }

    /// Jump to previous page
    pub fn previous_page(&mut self) {
        if self.display_count() > 0 && self.page_height > 0 {
            let current_page = self.current_page();
            if current_page > 0 {
                self.selected_index = (current_page - 1) * self.page_height;
            } else {
                self.selected_index = 0;
            }
            self.list_state.select(Some(self.index_in_page()));
            self.detail_scroll = 0;
        }
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
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
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
        self.input_mode = InputMode::Normal;
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
        self.input_mode = InputMode::Normal;
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
        self.input_mode = InputMode::Normal;
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
        self.input_mode = InputMode::Normal;
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
        self.input_mode = InputMode::Normal;
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
                        // Open full-screen detail view
                        if app.selected_pea().is_some() {
                            app.detail_scroll = 0;
                            app.input_mode = InputMode::DetailView;
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
                        app.input_mode = InputMode::Normal;
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
                        app.input_mode = InputMode::Normal;
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
                        app.input_mode = InputMode::Normal;
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
                        app.input_mode = InputMode::Normal;
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
                        app.input_mode = InputMode::Normal;
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
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
                        app.scroll_detail_down();
                    }
                    KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
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
                        // Allow editing from detail view
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
                        }
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
            }

            // Clear message after any key press
            if app.message.is_some() && key.code != KeyCode::Enter {
                app.message = None;
            }
        }
    }
}
