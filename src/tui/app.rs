use super::ui;
use crate::{
    config::PeasConfig,
    error::Result,
    model::{Pea, PeaStatus, PeaType},
    storage::PeaRepository,
};
use cli_clipboard::ClipboardProvider;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, widgets::ListState};
use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    List,
    #[default]
    Tree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    All,
    Open,
    InProgress,
    Completed,
    Milestones,
    Epics,
    Tasks,
}

impl FilterType {
    pub fn label(&self) -> &'static str {
        match self {
            FilterType::All => "All",
            FilterType::Open => "Open",
            FilterType::InProgress => "In Progress",
            FilterType::Completed => "Completed",
            FilterType::Milestones => "Milestones",
            FilterType::Epics => "Epics",
            FilterType::Tasks => "Tasks",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            FilterType::All => FilterType::Open,
            FilterType::Open => FilterType::InProgress,
            FilterType::InProgress => FilterType::Completed,
            FilterType::Completed => FilterType::Milestones,
            FilterType::Milestones => FilterType::Epics,
            FilterType::Epics => FilterType::Tasks,
            FilterType::Tasks => FilterType::All,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            FilterType::All => FilterType::Tasks,
            FilterType::Open => FilterType::All,
            FilterType::InProgress => FilterType::Open,
            FilterType::Completed => FilterType::InProgress,
            FilterType::Milestones => FilterType::Completed,
            FilterType::Epics => FilterType::Milestones,
            FilterType::Tasks => FilterType::Epics,
        }
    }
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
    pub all_peas: Vec<Pea>,
    pub filtered_peas: Vec<Pea>,
    pub tree_nodes: Vec<TreeNode>, // Flattened tree for display
    pub selected_index: usize,
    pub list_state: ListState,
    pub detail_scroll: u16, // Scroll offset for details view
    pub filter: FilterType,
    pub view_mode: ViewMode,
    pub input_mode: InputMode,
    pub search_query: String,
    pub show_help: bool,
    pub message: Option<String>,
}

impl App {
    pub fn new(config: &PeasConfig, project_root: &Path) -> Result<Self> {
        let repo = PeaRepository::new(config, project_root);
        let all_peas = repo.list()?;
        let filtered_peas = all_peas.clone();

        let mut list_state = ListState::default();
        if !filtered_peas.is_empty() {
            list_state.select(Some(0));
        }

        let mut app = Self {
            repo,
            all_peas,
            filtered_peas,
            tree_nodes: Vec::new(),
            selected_index: 0,
            list_state,
            detail_scroll: 0,
            filter: FilterType::All,
            view_mode: ViewMode::default(),
            input_mode: InputMode::Normal,
            search_query: String::new(),
            show_help: false,
            message: None,
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
                    current_parent_lines.push(!is_last);
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

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::List => ViewMode::Tree,
            ViewMode::Tree => ViewMode::List,
        };
        self.selected_index = 0;
        self.list_state.select(if self.display_count() > 0 {
            Some(0)
        } else {
            None
        });
    }

    /// Returns the number of items in the current view
    pub fn display_count(&self) -> usize {
        match self.view_mode {
            ViewMode::List => self.filtered_peas.len(),
            ViewMode::Tree => self.tree_nodes.len(),
        }
    }

    pub fn apply_filter(&mut self) {
        self.filtered_peas = self
            .all_peas
            .iter()
            .filter(|p| {
                let matches_filter = match self.filter {
                    FilterType::All => true,
                    FilterType::Open => p.is_open(),
                    FilterType::InProgress => p.status == PeaStatus::InProgress,
                    FilterType::Completed => p.status == PeaStatus::Completed,
                    FilterType::Milestones => p.pea_type == PeaType::Milestone,
                    FilterType::Epics => p.pea_type == PeaType::Epic,
                    FilterType::Tasks => p.pea_type == PeaType::Task,
                };

                let matches_search = if self.search_query.is_empty() {
                    true
                } else {
                    let query = self.search_query.to_lowercase();
                    p.title.to_lowercase().contains(&query)
                        || p.id.to_lowercase().contains(&query)
                        || p.body.to_lowercase().contains(&query)
                };

                matches_filter && matches_search
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
        match self.view_mode {
            ViewMode::List => self.filtered_peas.get(self.selected_index),
            ViewMode::Tree => self.tree_nodes.get(self.selected_index).map(|n| &n.pea),
        }
    }

    pub fn selected_pea_file_path(&self) -> Option<PathBuf> {
        self.selected_pea()
            .and_then(|pea| self.repo.find_file_by_id(&pea.id).ok())
    }

    pub fn next(&mut self) {
        let count = self.display_count();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
            self.list_state.select(Some(self.selected_index));
            self.detail_scroll = 0; // Reset scroll when changing selection
        }
    }

    pub fn previous(&mut self) {
        let count = self.display_count();
        if count > 0 {
            self.selected_index = if self.selected_index == 0 {
                count - 1
            } else {
                self.selected_index - 1
            };
            self.list_state.select(Some(self.selected_index));
            self.detail_scroll = 0; // Reset scroll when changing selection
        }
    }

    pub fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(1);
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(1);
    }

    pub fn next_filter(&mut self) {
        self.filter = self.filter.next();
        self.apply_filter();
    }

    pub fn prev_filter(&mut self) {
        self.filter = self.filter.prev();
        self.apply_filter();
    }

    pub fn toggle_status(&mut self) -> Result<()> {
        if let Some(pea) = self.selected_pea().cloned() {
            let mut updated = pea.clone();
            updated.status = match pea.status {
                PeaStatus::Todo => PeaStatus::InProgress,
                PeaStatus::InProgress => PeaStatus::Completed,
                PeaStatus::Completed => PeaStatus::Todo,
                PeaStatus::Draft => PeaStatus::Todo,
                PeaStatus::Scrapped => PeaStatus::Todo,
            };
            updated.touch();
            self.repo.update(&updated)?;
            self.message = Some(format!("{} -> {}", pea.id, updated.status));
            self.refresh()?;
        }
        Ok(())
    }

    pub fn start_selected(&mut self) -> Result<()> {
        if let Some(pea) = self.selected_pea().cloned() {
            let mut updated = pea.clone();
            updated.status = PeaStatus::InProgress;
            updated.touch();
            self.repo.update(&updated)?;
            self.message = Some(format!("Started {}", pea.id));
            self.refresh()?;
        }
        Ok(())
    }

    pub fn complete_selected(&mut self) -> Result<()> {
        if let Some(pea) = self.selected_pea().cloned() {
            let mut updated = pea.clone();
            updated.status = PeaStatus::Completed;
            updated.touch();
            self.repo.update(&updated)?;
            self.message = Some(format!("Completed {}", pea.id));
            self.refresh()?;
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
    let res = run_app(&mut terminal, &mut app);

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

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
                        } else if !app.search_query.is_empty() {
                            app.search_query.clear();
                            app.apply_filter();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Tab => app.next_filter(),
                    KeyCode::BackTab => app.prev_filter(),
                    KeyCode::Char('/') => {
                        app.input_mode = InputMode::Filter;
                    }
                    KeyCode::Char('v') => {
                        app.toggle_view_mode();
                        let mode_name = match app.view_mode {
                            ViewMode::List => "List",
                            ViewMode::Tree => "Tree",
                        };
                        app.message = Some(format!("{} view", mode_name));
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        let _ = app.toggle_status();
                    }
                    KeyCode::Char('s') => {
                        let _ = app.start_selected();
                    }
                    KeyCode::Char('d') => {
                        let _ = app.complete_selected();
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
                    KeyCode::Char('J') => app.scroll_detail_down(),
                    KeyCode::Char('K') => app.scroll_detail_up(),
                    KeyCode::PageDown => {
                        for _ in 0..5 {
                            app.scroll_detail_down();
                        }
                    }
                    KeyCode::PageUp => {
                        for _ in 0..5 {
                            app.scroll_detail_up();
                        }
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
            }

            // Clear message after any key press
            if app.message.is_some() && key.code != KeyCode::Enter {
                app.message = None;
            }
        }
    }
}
