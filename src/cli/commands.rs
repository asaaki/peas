use crate::config::PeasSettings;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "peas")]
#[command(
    author,
    version,
    about = "A CLI-based, flat-file issue tracker for humans and robots"
)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Path to config file (searches upward for .peas.yml by default)
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Path to data directory (overrides config)
    #[arg(long, global = true)]
    pub peas_path: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new peas project
    Init {
        /// Use a custom prefix for pea IDs
        #[arg(long, default_value_t = PeasSettings::default().prefix)]
        prefix: String,

        /// Length of random ID suffix
        #[arg(long, default_value_t = PeasSettings::default().id_length)]
        id_length: usize,
    },

    /// Create a new pea
    #[command(visible_alias = "c", visible_alias = "new")]
    Create {
        /// Title of the pea
        title: String,

        /// Type of pea
        #[arg(short = 't', long, value_enum, default_value = "task")]
        r#type: PeaTypeArg,

        /// Initial status
        #[arg(short, long, value_enum)]
        status: Option<PeaStatusArg>,

        /// Priority level
        #[arg(short, long, value_enum)]
        priority: Option<PeaPriorityArg>,

        /// Body content (use '-' to read from stdin)
        #[arg(short = 'd', long = "body")]
        body: Option<String>,

        /// Read body from file
        #[arg(long)]
        body_file: Option<String>,

        /// Parent pea ID
        #[arg(long)]
        parent: Option<String>,

        /// IDs of peas this blocks
        #[arg(long)]
        blocking: Vec<String>,

        /// Tags to add
        #[arg(long)]
        tag: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Preview what would be created without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Show a pea's contents
    Show {
        /// Pea ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List all peas
    #[command(visible_alias = "ls")]
    List {
        /// Filter by type
        #[arg(short = 't', long, value_enum)]
        r#type: Option<PeaTypeArg>,

        /// Filter by status
        #[arg(short, long, value_enum)]
        status: Option<PeaStatusArg>,

        /// Filter by priority
        #[arg(short, long, value_enum)]
        priority: Option<PeaPriorityArg>,

        /// Filter by parent ID
        #[arg(long)]
        parent: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Include archived peas
        #[arg(long)]
        archived: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Update a pea's properties
    Update {
        /// Pea ID
        id: String,

        /// New title
        #[arg(long)]
        title: Option<String>,

        /// New type
        #[arg(short = 't', long, value_enum)]
        r#type: Option<PeaTypeArg>,

        /// New status
        #[arg(short, long, value_enum)]
        status: Option<PeaStatusArg>,

        /// New priority
        #[arg(short, long, value_enum)]
        priority: Option<PeaPriorityArg>,

        /// New body content
        #[arg(short = 'd', long = "body")]
        body: Option<String>,

        /// New parent ID (use empty string to clear)
        #[arg(long)]
        parent: Option<String>,

        /// Add a tag
        #[arg(long)]
        add_tag: Vec<String>,

        /// Remove a tag
        #[arg(long)]
        remove_tag: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Preview what would be changed without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Archive a pea (move to archive folder)
    Archive {
        /// Pea ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Delete a pea permanently
    Delete {
        /// Pea ID
        id: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Search peas by text
    Search {
        /// Search query
        query: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Mark a pea as in-progress
    Start {
        /// Pea ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Mark a pea as completed
    Done {
        /// Pea ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Execute a GraphQL query
    Query {
        /// GraphQL query string
        query: String,

        /// Variables as JSON
        #[arg(long)]
        variables: Option<String>,
    },

    /// Execute a GraphQL mutation (automatically wraps in 'mutation { }')
    Mutate {
        /// Mutation body (without 'mutation' keyword)
        mutation: String,

        /// Variables as JSON
        #[arg(long)]
        variables: Option<String>,
    },

    /// Start GraphQL HTTP server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "4000")]
        port: u16,
    },

    /// Output instructions for AI coding agents
    Prime,

    /// Output project context for LLMs
    Context,

    /// Suggest the next ticket to work on
    Suggest {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate a Markdown roadmap from milestones and epics
    Roadmap,

    /// Open the interactive TUI
    Tui,

    /// Import from a beans project
    #[command(name = "import-beans")]
    ImportBeans {
        /// Path to .beans directory
        #[arg(default_value = ".beans")]
        path: String,

        /// Dry run - show what would be imported without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Export to beans format
    #[command(name = "export-beans")]
    ExportBeans {
        /// Output directory
        #[arg(default_value = ".beans-export")]
        output: String,
    },

    /// Bulk update multiple peas at once
    Bulk {
        #[command(subcommand)]
        action: BulkAction,
    },
}

#[derive(Subcommand)]
pub enum BulkAction {
    /// Set status of multiple peas
    Status {
        /// New status to set
        #[arg(value_enum)]
        status: PeaStatusArg,

        /// Pea IDs to update
        #[arg(required = true)]
        ids: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Start multiple peas (set to in-progress)
    Start {
        /// Pea IDs to start
        #[arg(required = true)]
        ids: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Complete multiple peas (set to completed)
    Done {
        /// Pea IDs to complete
        #[arg(required = true)]
        ids: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Add a tag to multiple peas
    Tag {
        /// Tag to add
        tag: String,

        /// Pea IDs to tag
        #[arg(required = true)]
        ids: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Set parent of multiple peas
    Parent {
        /// Parent ID to set
        parent: String,

        /// Pea IDs to update
        #[arg(required = true)]
        ids: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Create multiple peas at once (reads titles from stdin, one per line)
    Create {
        /// Type for all created peas
        #[arg(short = 't', long, value_enum, default_value = "task")]
        r#type: PeaTypeArg,

        /// Parent ID for all created peas
        #[arg(long)]
        parent: Option<String>,

        /// Tags to add to all created peas
        #[arg(long)]
        tag: Vec<String>,

        /// Priority for all created peas
        #[arg(short, long, value_enum)]
        priority: Option<PeaPriorityArg>,

        /// Initial status for all created peas
        #[arg(short, long, value_enum)]
        status: Option<PeaStatusArg>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Preview what would be created without making changes
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum PeaTypeArg {
    Milestone,
    Epic,
    Story,
    Feature,
    Bug,
    Chore,
    Research,
    Task,
}

impl From<PeaTypeArg> for crate::model::PeaType {
    fn from(arg: PeaTypeArg) -> Self {
        match arg {
            PeaTypeArg::Milestone => crate::model::PeaType::Milestone,
            PeaTypeArg::Epic => crate::model::PeaType::Epic,
            PeaTypeArg::Story => crate::model::PeaType::Story,
            PeaTypeArg::Feature => crate::model::PeaType::Feature,
            PeaTypeArg::Bug => crate::model::PeaType::Bug,
            PeaTypeArg::Chore => crate::model::PeaType::Chore,
            PeaTypeArg::Research => crate::model::PeaType::Research,
            PeaTypeArg::Task => crate::model::PeaType::Task,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum PeaStatusArg {
    Draft,
    Todo,
    InProgress,
    Completed,
    Scrapped,
}

impl From<PeaStatusArg> for crate::model::PeaStatus {
    fn from(arg: PeaStatusArg) -> Self {
        match arg {
            PeaStatusArg::Draft => crate::model::PeaStatus::Draft,
            PeaStatusArg::Todo => crate::model::PeaStatus::Todo,
            PeaStatusArg::InProgress => crate::model::PeaStatus::InProgress,
            PeaStatusArg::Completed => crate::model::PeaStatus::Completed,
            PeaStatusArg::Scrapped => crate::model::PeaStatus::Scrapped,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub enum PeaPriorityArg {
    Critical,
    High,
    Normal,
    Low,
    Deferred,
}

impl From<PeaPriorityArg> for crate::model::PeaPriority {
    fn from(arg: PeaPriorityArg) -> Self {
        match arg {
            PeaPriorityArg::Critical => crate::model::PeaPriority::Critical,
            PeaPriorityArg::High => crate::model::PeaPriority::High,
            PeaPriorityArg::Normal => crate::model::PeaPriority::Normal,
            PeaPriorityArg::Low => crate::model::PeaPriority::Low,
            PeaPriorityArg::Deferred => crate::model::PeaPriority::Deferred,
        }
    }
}
