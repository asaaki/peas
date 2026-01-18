use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use peas::{
    cli::{Cli, Commands},
    config::{PeasConfig, PeasSettings},
    graphql::build_schema,
    model::{Pea, PeaStatus},
    storage::PeaRepository,
};
use std::{
    io::{self, Read, Write},
    path::PathBuf,
};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_opt = cli.config;
    let peas_path_opt = cli.peas_path;

    // Helper closure to load config with CLI overrides
    let load = || load_config(config_opt.clone(), peas_path_opt.clone());

    match cli.command {
        Commands::Init { prefix, id_length } => cmd_init(prefix, id_length, peas_path_opt),
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
            let (config, root) = load()?;
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
            let filename = path
                .file_name()
                .map(|f| f.to_string_lossy())
                .unwrap_or_default();

            if json {
                println!("{}", serde_json::to_string_pretty(&pea)?);
            } else {
                println!("{} {} {}", "Created".green(), pea.id.cyan(), filename);
            }
            Ok(())
        }
        Commands::Show { id, json } => {
            let (config, root) = load()?;
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
            let (config, root) = load()?;
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
            let (config, root) = load()?;
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
            let filename = path
                .file_name()
                .map(|f| f.to_string_lossy())
                .unwrap_or_default();

            if json {
                println!("{}", serde_json::to_string_pretty(&pea)?);
            } else {
                println!("{} {} {}", "Updated".green(), pea.id.cyan(), filename);
            }
            Ok(())
        }
        Commands::Archive { id } => {
            let (config, root) = load()?;
            let repo = PeaRepository::new(&config, &root);
            let path = repo.archive(&id)?;
            let filename = path
                .file_name()
                .map(|f| f.to_string_lossy())
                .unwrap_or_default();
            println!("{} {} -> {}", "Archived".yellow(), id.cyan(), filename);
            Ok(())
        }
        Commands::Delete { id, force } => {
            let (config, root) = load()?;
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
            let (config, root) = load()?;
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
            let (config, root) = load()?;
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
            let (config, root) = load()?;
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
            let (config, root) = load()?;
            let repo = PeaRepository::new(&config, &root);
            print_prime_instructions(&config, &repo)?;
            Ok(())
        }
        Commands::Context => {
            let (config, root) = load()?;
            let repo = PeaRepository::new(&config, &root);
            print_context(&repo)?;
            Ok(())
        }
        Commands::Roadmap => {
            let (config, root) = load()?;
            let repo = PeaRepository::new(&config, &root);
            print_roadmap(&repo)?;
            Ok(())
        }
        Commands::Query { query, variables } => {
            let (config, root) = load()?;
            let schema = build_schema(config, root);

            let vars: async_graphql::Variables = if let Some(v) = variables {
                serde_json::from_str(&v)?
            } else {
                async_graphql::Variables::default()
            };

            let request = async_graphql::Request::new(&query).variables(vars);
            let response = tokio::runtime::Runtime::new()?.block_on(schema.execute(request));

            println!("{}", serde_json::to_string_pretty(&response)?);
            Ok(())
        }
        Commands::Mutate {
            mutation,
            variables,
        } => {
            let (config, root) = load()?;
            let schema = build_schema(config, root);

            let vars: async_graphql::Variables = if let Some(v) = variables {
                serde_json::from_str(&v)?
            } else {
                async_graphql::Variables::default()
            };

            // Auto-wrap in mutation { }
            let query = format!("mutation {{ {} }}", mutation);
            let request = async_graphql::Request::new(&query).variables(vars);
            let response = tokio::runtime::Runtime::new()?.block_on(schema.execute(request));

            println!("{}", serde_json::to_string_pretty(&response)?);
            Ok(())
        }
        Commands::Serve { port } => {
            let (config, root) = load()?;
            let schema = build_schema(config, root);

            println!("Starting GraphQL server on http://localhost:{}", port);
            println!("GraphQL Playground: http://localhost:{}", port);

            tokio::runtime::Runtime::new()?.block_on(async { run_server(schema, port).await })?;
            Ok(())
        }
        Commands::Tui => {
            let (config, root) = load()?;
            peas::tui::run_tui(config, root)?;
            Ok(())
        }
        Commands::ImportBeans { path, dry_run } => {
            let (config, root) = load()?;
            let repo = PeaRepository::new(&config, &root);
            let beans_path = std::path::Path::new(&path);

            let peas = peas::import_export::import_beans_directory(beans_path)?;

            if peas.is_empty() {
                println!("No beans files found to import in {}", path);
                return Ok(());
            }

            println!("Found {} beans to import:", peas.len());
            for pea in &peas {
                println!("  {} [{}] {}", pea.id, pea.pea_type, pea.title);
            }

            if dry_run {
                println!("\nDry run - no changes made.");
            } else {
                let mut imported = 0;
                let mut skipped = 0;
                for pea in peas {
                    // Check if already exists
                    if repo.find_file_by_id(&pea.id).is_ok() {
                        println!("  Skipping {} (already exists)", pea.id);
                        skipped += 1;
                        continue;
                    }
                    match repo.create(&pea) {
                        Ok(_) => imported += 1,
                        Err(e) => eprintln!("  Failed to import {}: {}", pea.id, e),
                    }
                }
                println!("\nImported {} peas, skipped {}", imported, skipped);
            }
            Ok(())
        }
        Commands::ExportBeans { output } => {
            let (config, root) = load()?;
            let repo = PeaRepository::new(&config, &root);
            let output_path = std::path::Path::new(&output);

            std::fs::create_dir_all(output_path)?;

            let peas = repo.list()?;
            if peas.is_empty() {
                println!("No peas to export");
                return Ok(());
            }

            let mut exported = 0;
            for pea in &peas {
                let content = peas::import_export::export_to_beans(pea)?;
                let filename = peas::import_export::beans_filename(pea);
                let file_path = output_path.join(&filename);
                std::fs::write(&file_path, content)?;
                exported += 1;
            }

            println!("Exported {} peas to {}", exported, output);
            Ok(())
        }
    }
}

fn cmd_init(prefix: String, id_length: usize, peas_path: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join(".peas.yml");

    if config_path.exists() {
        anyhow::bail!("Project already initialized at {}", config_path.display());
    }

    let data_dir = peas_path.unwrap_or_else(|| ".peas".to_string());

    let config = PeasConfig {
        peas: PeasSettings {
            path: data_dir.clone(),
            prefix,
            id_length,
            default_status: "todo".to_string(),
            default_type: "task".to_string(),
            frontmatter: "toml".to_string(),
        },
    };

    // Create data directory
    let data_path = cwd.join(&data_dir);
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
    let in_progress: Vec<_> = peas
        .iter()
        .filter(|p| p.status == PeaStatus::InProgress)
        .collect();

    println!(
        r#"# Peas - Issue Tracker

This project uses **peas** for issue tracking. Issues are stored as markdown files in the `{}` directory.

## CLI Commands

```bash
peas list                          # List all peas
peas list -t epic                  # List by type
peas list -s in-progress           # List by status
peas show <id>                     # Show pea details
peas create "<title>" -t <type>    # Create a new pea
peas update <id> -s <status>       # Update pea status
peas start <id>                    # Mark as in-progress
peas done <id>                     # Mark as completed
peas search "<query>"              # Search peas
peas roadmap                       # Show project roadmap
```

## GraphQL Interface

For complex queries, use the GraphQL interface:

```bash
# Get project stats
peas query '{{ stats {{ total byStatus {{ todo inProgress completed }} }} }}'

# List all open peas
peas query '{{ peas(filter: {{ isOpen: true }}) {{ nodes {{ id title peaType status }} }} }}'

# Create a pea (mutate auto-wraps in 'mutation {{ }}')
peas mutate 'createPea(input: {{ title: "New Task", peaType: TASK }}) {{ id }}'

# Update status
peas mutate 'setStatus(id: "<id>", status: IN_PROGRESS) {{ id status }}'
```

## Pea Types
milestone, epic, feature, bug, task

## Pea Statuses
draft, todo, in-progress, completed, scrapped
"#,
        config.peas.path
    );

    if !in_progress.is_empty() {
        println!("## Currently In Progress ({})", in_progress.len());
        for pea in &in_progress {
            println!("- [{}] {} - {}", pea.id, pea.pea_type, pea.title);
        }
        println!();
    }

    println!("## Open Peas ({} total)", open_peas.len());
    for pea in open_peas.iter().take(15) {
        println!("- [{}] {} - {}", pea.id, pea.pea_type, pea.title);
    }

    if open_peas.len() > 15 {
        println!(
            "... and {} more (use `peas list` for full list)",
            open_peas.len() - 15
        );
    }

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

async fn run_server(schema: peas::graphql::PeasSchema, port: u16) -> Result<()> {
    use async_graphql::http::GraphiQLSource;
    use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
    use axum::{
        Router,
        extract::Extension,
        response::{Html, IntoResponse},
        routing::get,
    };

    async fn graphql_handler(
        Extension(schema): Extension<peas::graphql::PeasSchema>,
        req: GraphQLRequest,
    ) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }

    async fn graphiql() -> impl IntoResponse {
        Html(GraphiQLSource::build().endpoint("/").finish())
    }

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
