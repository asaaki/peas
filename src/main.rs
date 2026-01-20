use anyhow::{Context, Result};
use clap::Parser;
use peas::{
    cli::{Cli, Commands, handlers::CommandContext},
    config::PeasConfig,
};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_opt = cli.config;
    let peas_path_opt = cli.peas_path;

    match cli.command {
        Commands::Init { prefix, id_length } => {
            peas::cli::handlers::handle_init(prefix, id_length, peas_path_opt)
        }
        _ => {
            // All other commands require loading config
            let (config, root) = load_config(config_opt, peas_path_opt)?;
            let ctx = CommandContext::new(config, root);

            match cli.command {
                Commands::Init { .. } => unreachable!(),
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
                    &ctx, r#type, status, priority, parent, tag, archived, json,
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
                Commands::Archive { id, json } => {
                    peas::cli::handlers::handle_archive(&ctx, id, json)
                }
                Commands::Delete { id, force, json } => {
                    peas::cli::handlers::handle_delete(&ctx, id, force, json)
                }
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
                Commands::Undo { json } => peas::cli::handlers::handle_undo(&ctx, json),
            }
        }
    }
}

fn load_config(
    config_path: Option<String>,
    peas_path: Option<String>,
) -> Result<(PeasConfig, PathBuf)> {
    let (mut config, project_root) = if let Some(path) = config_path {
        let path = PathBuf::from(path);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;
        let config: PeasConfig = serde_yaml::from_str(&content)?;
        let root = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Config path has no parent"))?
            .to_path_buf();
        (config, root)
    } else {
        let cwd = std::env::current_dir()?;
        PeasConfig::load(&cwd).context("Failed to load peas configuration")?
    };

    if let Some(path) = peas_path {
        config.peas.path = path;
    }

    Ok((config, project_root))
}
