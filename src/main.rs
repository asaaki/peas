use std::io::{self, Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;

use peas::cli::{Cli, Commands};
use peas::config::{PeasConfig, PeasSettings};
use peas::model::{Pea, PeaStatus};
use peas::storage::PeaRepository;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { prefix, id_length } => cmd_init(prefix, id_length),
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
            json,
        } => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);

            let body_content = resolve_body(body, body_file)?;
            let pea_type = r#type.into();
            let id = repo.generate_id();

            let mut pea = Pea::new(id, title, pea_type);

            if let Some(s) = status {
                pea = pea.with_status(s.into());
            }
            if let Some(p) = priority {
                pea = pea.with_priority(p.into());
            }
            if !tag.is_empty() {
                pea = pea.with_tags(tag);
            }
            if parent.is_some() {
                pea = pea.with_parent(parent);
            }
            if !blocking.is_empty() {
                pea = pea.with_blocking(blocking);
            }
            if let Some(b) = body_content {
                pea = pea.with_body(b);
            }

            let path = repo.create(&pea)?;
            let filename = path.file_name().unwrap().to_string_lossy();

            if json {
                println!("{}", serde_json::to_string_pretty(&pea)?);
            } else {
                println!("{} {} {}", "Created".green(), pea.id.cyan(), filename);
            }
            Ok(())
        }
        Commands::Show { id, json } => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);
            let pea = repo.get(&id)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&pea)?);
            } else {
                print_pea(&pea);
            }
            Ok(())
        }
        Commands::List {
            r#type,
            status,
            priority,
            parent,
            tag,
            archived,
            json,
        } => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);

            let mut peas = if archived {
                repo.list_archived()?
            } else {
                repo.list()?
            };

            // Apply filters
            if let Some(t) = r#type {
                let filter_type = t.into();
                peas.retain(|p| p.pea_type == filter_type);
            }
            if let Some(s) = status {
                let filter_status: PeaStatus = s.into();
                peas.retain(|p| p.status == filter_status);
            }
            if let Some(p) = priority {
                let filter_priority = p.into();
                peas.retain(|p| p.priority == filter_priority);
            }
            if let Some(ref parent_id) = parent {
                peas.retain(|p| p.parent.as_deref() == Some(parent_id.as_str()));
            }
            if let Some(ref t) = tag {
                peas.retain(|p| p.tags.contains(t));
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&peas)?);
            } else {
                print_pea_list(&peas);
            }
            Ok(())
        }
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
        } => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);
            let mut pea = repo.get(&id)?;

            if let Some(t) = title {
                pea.title = t;
            }
            if let Some(t) = r#type {
                pea.pea_type = t.into();
            }
            if let Some(s) = status {
                pea.status = s.into();
            }
            if let Some(p) = priority {
                pea.priority = p.into();
            }
            if let Some(b) = body {
                pea.body = b;
            }
            if let Some(p) = parent {
                pea.parent = if p.is_empty() { None } else { Some(p) };
            }
            for t in add_tag {
                if !pea.tags.contains(&t) {
                    pea.tags.push(t);
                }
            }
            for t in remove_tag {
                pea.tags.retain(|x| x != &t);
            }

            pea.touch();
            let path = repo.update(&pea)?;
            let filename = path.file_name().unwrap().to_string_lossy();

            if json {
                println!("{}", serde_json::to_string_pretty(&pea)?);
            } else {
                println!("{} {} {}", "Updated".green(), pea.id.cyan(), filename);
            }
            Ok(())
        }
        Commands::Archive { id } => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);
            let path = repo.archive(&id)?;
            let filename = path.file_name().unwrap().to_string_lossy();
            println!("{} {} -> {}", "Archived".yellow(), id.cyan(), filename);
            Ok(())
        }
        Commands::Delete { id, force } => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);

            if !force {
                print!("Delete {} permanently? [y/N] ", id.cyan());
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            repo.delete(&id)?;
            println!("{} {}", "Deleted".red(), id.cyan());
            Ok(())
        }
        Commands::Search { query, json } => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);
            let peas = repo.list()?;

            let query_lower = query.to_lowercase();
            let results: Vec<_> = peas
                .into_iter()
                .filter(|p| {
                    p.title.to_lowercase().contains(&query_lower)
                        || p.body.to_lowercase().contains(&query_lower)
                        || p.id.to_lowercase().contains(&query_lower)
                })
                .collect();

            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                println!("Found {} results for '{}':\n", results.len(), query);
                print_pea_list(&results);
            }
            Ok(())
        }
        Commands::Start { id } => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);
            let mut pea = repo.get(&id)?;
            pea.status = PeaStatus::InProgress;
            pea.touch();
            repo.update(&pea)?;
            println!(
                "{} {} is now {}",
                "Started".green(),
                pea.id.cyan(),
                "in-progress".yellow()
            );
            Ok(())
        }
        Commands::Done { id } => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);
            let mut pea = repo.get(&id)?;
            pea.status = PeaStatus::Completed;
            pea.touch();
            repo.update(&pea)?;
            println!(
                "{} {} is now {}",
                "Done".green(),
                pea.id.cyan(),
                "completed".green()
            );
            Ok(())
        }
        Commands::Prime => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);
            print_prime_instructions(&config, &repo)?;
            Ok(())
        }
        Commands::Context => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);
            print_context(&repo)?;
            Ok(())
        }
        Commands::Roadmap => {
            let (config, root) = load_config()?;
            let repo = PeaRepository::new(&config, &root);
            print_roadmap(&repo)?;
            Ok(())
        }
        Commands::Graphql { query, variables } => {
            eprintln!("GraphQL support will be implemented in Milestone 2");
            let _ = (query, variables);
            Ok(())
        }
        Commands::Serve { port } => {
            eprintln!("GraphQL server will be implemented in Milestone 2");
            let _ = port;
            Ok(())
        }
        Commands::Tui => {
            eprintln!("TUI will be implemented in Milestone 3");
            Ok(())
        }
    }
}

fn cmd_init(prefix: String, id_length: usize) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join(".peas.yml");

    if config_path.exists() {
        anyhow::bail!("Project already initialized at {}", config_path.display());
    }

    let config = PeasConfig {
        peas: PeasSettings {
            path: ".peas".to_string(),
            prefix,
            id_length,
            default_status: "todo".to_string(),
            default_type: "task".to_string(),
        },
    };

    // Create data directory
    let data_path = cwd.join(&config.peas.path);
    std::fs::create_dir_all(&data_path)?;

    // Save config
    config.save(&config_path)?;

    println!(
        "{} peas project in {}",
        "Initialized".green(),
        cwd.display()
    );
    println!("  Config: {}", config_path.display());
    println!("  Data:   {}", data_path.display());

    Ok(())
}

fn load_config() -> Result<(PeasConfig, PathBuf)> {
    let cwd = std::env::current_dir()?;
    PeasConfig::load(&cwd).context("Failed to load peas configuration")
}

fn resolve_body(body: Option<String>, body_file: Option<String>) -> Result<Option<String>> {
    if let Some(b) = body {
        if b == "-" {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            return Ok(Some(content.trim().to_string()));
        }
        return Ok(Some(b));
    }
    if let Some(path) = body_file {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read body from {}", path))?;
        return Ok(Some(content.trim().to_string()));
    }
    Ok(None)
}

fn print_pea(pea: &Pea) {
    println!("{} {}", pea.id.cyan().bold(), pea.title.bold());
    println!("Type:     {}", format!("{}", pea.pea_type).blue());
    println!("Status:   {}", format_status(pea.status));
    println!("Priority: {}", format_priority(pea.priority));

    if let Some(ref parent) = pea.parent {
        println!("Parent:   {}", parent.cyan());
    }
    if !pea.blocking.is_empty() {
        println!("Blocking: {}", pea.blocking.join(", ").cyan());
    }
    if !pea.tags.is_empty() {
        println!("Tags:     {}", pea.tags.join(", ").magenta());
    }

    println!("Created:  {}", pea.created.format("%Y-%m-%d %H:%M"));
    println!("Updated:  {}", pea.updated.format("%Y-%m-%d %H:%M"));

    if !pea.body.is_empty() {
        println!("\n{}", pea.body);
    }
}

fn print_pea_list(peas: &[Pea]) {
    if peas.is_empty() {
        println!("No peas found.");
        return;
    }

    for pea in peas {
        let status_str = format_status(pea.status);
        let type_str = format!("{}", pea.pea_type).blue();
        println!(
            "{} {} [{}] {}",
            pea.id.cyan(),
            status_str,
            type_str,
            pea.title
        );
    }
}

fn format_status(status: PeaStatus) -> colored::ColoredString {
    match status {
        PeaStatus::Draft => "draft".dimmed(),
        PeaStatus::Todo => "todo".white(),
        PeaStatus::InProgress => "in-progress".yellow(),
        PeaStatus::Completed => "completed".green(),
        PeaStatus::Scrapped => "scrapped".red(),
    }
}

fn format_priority(priority: peas::model::PeaPriority) -> colored::ColoredString {
    use peas::model::PeaPriority;
    match priority {
        PeaPriority::Critical => "critical".red().bold(),
        PeaPriority::High => "high".red(),
        PeaPriority::Normal => "normal".white(),
        PeaPriority::Low => "low".dimmed(),
        PeaPriority::Deferred => "deferred".dimmed(),
    }
}

fn print_prime_instructions(config: &PeasConfig, repo: &PeaRepository) -> Result<()> {
    let peas = repo.list()?;
    let open_peas: Vec<_> = peas.iter().filter(|p| p.is_open()).collect();

    println!(
        r#"# Peas - Issue Tracker

This project uses **peas** for issue tracking. Issues are stored as markdown files in the `{}` directory.

## Available Commands

- `peas list` - List all peas
- `peas show <id>` - Show pea details
- `peas create "<title>" -t <type>` - Create a new pea
- `peas update <id> -s <status>` - Update pea status
- `peas start <id>` - Mark pea as in-progress
- `peas done <id>` - Mark pea as completed
- `peas search "<query>"` - Search peas

## Pea Types
- milestone, epic, feature, bug, task

## Pea Statuses
- draft, todo, in-progress, completed, scrapped

## Current Open Peas ({})
"#,
        config.peas.path,
        open_peas.len()
    );

    for pea in open_peas.iter().take(20) {
        println!("- [{}] {} - {}", pea.id, pea.pea_type, pea.title);
    }

    if open_peas.len() > 20 {
        println!("... and {} more", open_peas.len() - 20);
    }

    println!("\nUse `peas list` for the full list or `peas show <id>` for details.");

    Ok(())
}

fn print_context(repo: &PeaRepository) -> Result<()> {
    let peas = repo.list()?;

    let context = serde_json::json!({
        "total": peas.len(),
        "by_status": {
            "draft": peas.iter().filter(|p| p.status == PeaStatus::Draft).count(),
            "todo": peas.iter().filter(|p| p.status == PeaStatus::Todo).count(),
            "in_progress": peas.iter().filter(|p| p.status == PeaStatus::InProgress).count(),
            "completed": peas.iter().filter(|p| p.status == PeaStatus::Completed).count(),
            "scrapped": peas.iter().filter(|p| p.status == PeaStatus::Scrapped).count(),
        },
        "by_type": {
            "milestone": peas.iter().filter(|p| p.pea_type == peas::model::PeaType::Milestone).count(),
            "epic": peas.iter().filter(|p| p.pea_type == peas::model::PeaType::Epic).count(),
            "feature": peas.iter().filter(|p| p.pea_type == peas::model::PeaType::Feature).count(),
            "bug": peas.iter().filter(|p| p.pea_type == peas::model::PeaType::Bug).count(),
            "task": peas.iter().filter(|p| p.pea_type == peas::model::PeaType::Task).count(),
        },
        "open_peas": peas.iter().filter(|p| p.is_open()).map(|p| {
            serde_json::json!({
                "id": p.id,
                "title": p.title,
                "type": format!("{}", p.pea_type),
                "status": format!("{}", p.status),
            })
        }).collect::<Vec<_>>(),
    });

    println!("{}", serde_json::to_string_pretty(&context)?);
    Ok(())
}

fn print_roadmap(repo: &PeaRepository) -> Result<()> {
    use peas::model::PeaType;

    let peas = repo.list()?;
    let milestones: Vec<_> = peas
        .iter()
        .filter(|p| p.pea_type == PeaType::Milestone)
        .collect();

    println!("# Roadmap\n");

    for milestone in &milestones {
        println!("## Milestone: {} ({})\n", milestone.title, milestone.id);
        if !milestone.body.is_empty() {
            println!("> {}\n", milestone.body.lines().next().unwrap_or(""));
        }

        let epics: Vec<_> = peas
            .iter()
            .filter(|p| p.pea_type == PeaType::Epic && p.parent.as_deref() == Some(&milestone.id))
            .collect();

        for epic in &epics {
            println!("### Epic: {} ({})\n", epic.title, epic.id);
            if !epic.body.is_empty() {
                println!("> {}\n", epic.body.lines().next().unwrap_or(""));
            }

            let tasks: Vec<_> = peas
                .iter()
                .filter(|p| p.parent.as_deref() == Some(&epic.id))
                .collect();

            for task in &tasks {
                let status_icon = match task.status {
                    PeaStatus::Completed => "[x]",
                    PeaStatus::InProgress => "[-]",
                    _ => "[ ]",
                };
                println!("- {} {} ({})", status_icon, task.title, task.id);
            }
            println!();
        }
    }

    Ok(())
}
