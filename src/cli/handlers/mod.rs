mod archive;
mod bulk;
mod context;
mod create;
mod delete;
mod export_beans;
mod import_beans;
mod init;
mod list;
mod memory;
mod mutate;
mod prime;
mod query;
mod roadmap;
mod search;
mod serve;
mod show;
mod status;
mod suggest;
mod tui;
mod undo;
mod update;
mod utils;

pub use archive::handle_archive;
pub use bulk::handle_bulk;
pub use context::handle_context;
pub use create::handle_create;
pub use delete::handle_delete;
pub use export_beans::handle_export_beans;
pub use import_beans::handle_import_beans;
pub use init::handle_init;
pub use list::handle_list;
pub use memory::handle_memory;
pub use mutate::handle_mutate;
pub use prime::handle_prime;
pub use query::handle_query;
pub use roadmap::handle_roadmap;
pub use search::handle_search;
pub use serve::handle_serve;
pub use show::handle_show;
pub use status::{handle_done, handle_start};
pub use suggest::handle_suggest;
pub use tui::handle_tui;
pub use undo::handle_undo;
pub use update::handle_update;

use crate::config::PeasConfig;

use crate::storage::PeaRepository;
use std::path::PathBuf;

/// Common context passed to all command handlers
pub struct CommandContext {
    pub config: PeasConfig,
    pub root: PathBuf,
    pub repo: PeaRepository,
}

impl CommandContext {
    pub fn new(config: PeasConfig, root: PathBuf) -> Self {
        let repo = PeaRepository::new(&config, &root);
        Self { config, root, repo }
    }
}
