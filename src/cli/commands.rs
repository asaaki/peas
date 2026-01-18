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
        #[arg(long, default_value = "peas-")]
        prefix: String,

        /// Length of random ID suffix
        #[arg(long, default_value = "5")]
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
    },

    /// Archive a pea (move to archive folder)
    Archive {
        /// Pea ID
        id: String,
    },

    /// Delete a pea permanently
    Delete {
        /// Pea ID
        id: String,

        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
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
    },

    /// Mark a pea as completed
    Done {
        /// Pea ID
        id: String,
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

    /// Generate a Markdown roadmap from milestones and epics
    Roadmap,

    /// Open the interactive TUI
    Tui,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum PeaTypeArg {
    Milestone,
    Epic,
    Feature,
    Bug,
    Task,
}

impl From<PeaTypeArg> for crate::model::PeaType {
    fn from(arg: PeaTypeArg) -> Self {
        match arg {
            PeaTypeArg::Milestone => crate::model::PeaType::Milestone,
            PeaTypeArg::Epic => crate::model::PeaType::Epic,
            PeaTypeArg::Feature => crate::model::PeaType::Feature,
            PeaTypeArg::Bug => crate::model::PeaType::Bug,
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
