use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::config::PeasConfig;
use crate::error::Result;
use crate::model::{Pea, PeaStatus, PeaType};
use crate::storage::PeaRepository;

use super::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filter,
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

pub struct App {
    pub repo: PeaRepository,
    pub all_peas: Vec<Pea>,
    pub filtered_peas: Vec<Pea>,
    pub selected_index: usize,
    pub filter: FilterType,
    pub input_mode: InputMode,
    pub search_query: String,
    pub show_help: bool,
    pub message: Option<String>,
}

impl App {
    pub fn new(config: &PeasConfig, project_root: &PathBuf) -> Result<Self> {
        let repo = PeaRepository::new(config, project_root);
        let all_peas = repo.list()?;
        let filtered_peas = all_peas.clone();

        Ok(Self {
            repo,
            all_peas,
            filtered_peas,
            selected_index: 0,
            filter: FilterType::All,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            show_help: false,
            message: None,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.all_peas = self.repo.list()?;
        self.apply_filter();
        Ok(())
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
    }

    pub fn selected_pea(&self) -> Option<&Pea> {
        self.filtered_peas.get(self.selected_index)
    }

    pub fn next(&mut self) {
        if !self.filtered_peas.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_peas.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered_peas.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_peas.len() - 1
            } else {
                self.selected_index - 1
            };
        }
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

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
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
