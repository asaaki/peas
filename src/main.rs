use anyhow::{Context, Result};
use clap::Parser;
use peas::{
    cli::{Cli, Commands, handlers::CommandContext},
    config::PeasConfig,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine if we're in TUI mode (to disable stderr logging)
    let is_tui_mode = matches!(cli.command, Commands::Tui);

    // Initialize logging system
    // In TUI mode, disable stderr logging to prevent interference with terminal rendering
    let log_file = cli.log_file.as_ref().map(PathBuf::from);
    peas::logging::init(cli.verbose, log_file, is_tui_mode);

    let config_opt = cli.config;

    // Print deprecation warning if --peas-path is used
    if cli.peas_path.is_some() {
        eprintln!(
            "{}: The --peas-path option is deprecated and ignored. Data is always stored in .peas/",
            colored::Colorize::yellow(colored::Colorize::bold("warning"))
        );
    }

    match cli.command {
        Commands::Init { prefix, id_length } => peas::cli::handlers::handle_init(prefix, id_length),
        Commands::Migrate { dry_run } => peas::cli::handlers::handle_migrate(dry_run),
        Commands::Doctor { fix } => peas::cli::handlers::handle_doctor(fix),
        _ => {
            // All other commands require loading config
            let (config, root) = load_config(config_opt)?;
            let ctx = CommandContext::new(config, root);

            match cli.command {
                Commands::Init { .. } | Commands::Migrate { .. } | Commands::Doctor { .. } => {
                    unreachable!()
                }
                Commands::Create {
                    title,
                    r#type,
                    status,
                    priority,
                    body,
                    body_file,
                    parent,
                    blocking,
                    tag,
                    template,
                    json,
                    dry_run,
                } => peas::cli::handlers::handle_create(
                    &ctx, title, r#type, status, priority, body, body_file, parent, blocking, tag,
                    template, json, dry_run,
                ),
                Commands::Show { id, json } => peas::cli::handlers::handle_show(&ctx, id, json),
                Commands::List {
                    r#type,
                    status,
                    priority,
                    parent,
                    tag,
                    archived,
                    json,
                } => peas::cli::handlers::handle_list(
                    &ctx,
                    peas::cli::handlers::ListParams {
                        r#type,
                        status,
                        priority,
                        parent,
                        tag,
                        archived,
                        json,
                    },
                ),
                Commands::Update {
                    id,
                    title,
                    r#type,
                    status,
                    priority,
                    body,
                    parent,
                    add_tag,
                    remove_tag,
                    json,
                    dry_run,
                } => peas::cli::handlers::handle_update(
                    &ctx, id, title, r#type, status, priority, body, parent, add_tag, remove_tag,
                    json, dry_run,
                ),
                Commands::Archive {
                    id,
                    keep_assets,
                    json,
                } => peas::cli::handlers::handle_archive(&ctx, id, keep_assets, json),
                Commands::Delete {
                    id,
                    force,
                    keep_assets,
                    json,
                } => peas::cli::handlers::handle_delete(&ctx, id, force, keep_assets, json),
                Commands::Search { query, json } => {
                    peas::cli::handlers::handle_search(&ctx, query, json)
                }
                Commands::Start { id, json } => peas::cli::handlers::handle_start(&ctx, id, json),
                Commands::Done { id, json } => peas::cli::handlers::handle_done(&ctx, id, json),
                Commands::Prime => peas::cli::handlers::handle_prime(&ctx),
                Commands::Context => peas::cli::handlers::handle_context(&ctx),
                Commands::Suggest { json, limit } => {
                    peas::cli::handlers::handle_suggest(&ctx, json, limit)
                }
                Commands::Roadmap => peas::cli::handlers::handle_roadmap(&ctx),
                Commands::Query { query, variables } => {
                    peas::cli::handlers::handle_query(ctx, query, variables)
                }
                Commands::Mutate {
                    mutation,
                    variables,
                } => peas::cli::handlers::handle_mutate(ctx, mutation, variables),
                Commands::Serve { port } => peas::cli::handlers::handle_serve(ctx, port),
                Commands::Tui => peas::cli::handlers::handle_tui(ctx),
                Commands::ImportBeans { path, dry_run } => {
                    peas::cli::handlers::handle_import_beans(&ctx, path, dry_run)
                }
                Commands::ExportBeans { output } => {
                    peas::cli::handlers::handle_export_beans(&ctx, output)
                }
                Commands::Bulk { action } => peas::cli::handlers::handle_bulk(&ctx, action),
                Commands::Memory { action } => peas::cli::handlers::handle_memory(&ctx, action),
                Commands::Asset { action } => peas::cli::handlers::handle_asset(&ctx, action),
                Commands::Undo { json } => peas::cli::handlers::handle_undo(&ctx, json),
                Commands::Mv {
                    id,
                    new_suffix,
                    force,
                } => peas::cli::handlers::handle_mv(&ctx, id, new_suffix, force),
            }
        }
    }
}

fn load_config(config_path: Option<String>) -> Result<(PeasConfig, PathBuf)> {
    if let Some(path) = config_path {
        let path = PathBuf::from(path);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let config: PeasConfig = serde_yaml::from_str(&content)?;
        let root = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Config path has no parent"))?
            .to_path_buf();
        Ok((config, root))
    } else {
        let cwd = std::env::current_dir()?;
        PeasConfig::load(&cwd).context("Failed to load peas configuration")
    }
}
