use anyhow::Result;
use colored::Colorize;
use peas::cli::MemoryAction;
use peas::model::Memory;
use peas::storage::MemoryRepository;

use super::CommandContext;

pub fn handle_memory(ctx: &CommandContext, action: MemoryAction) -> Result<()> {
    let repo = MemoryRepository::new(&ctx.config, &ctx.root);

    match action {
        MemoryAction::Save {
            key,
            content,
            tag,
            json,
        } => handle_memory_save(&repo, key, content, tag, json),
        MemoryAction::Query { key, json } => handle_memory_query(&repo, key, json),
        MemoryAction::List { tag, json } => handle_memory_list(&repo, tag, json),
        MemoryAction::Edit { key } => handle_memory_edit(&repo, ctx, key),
        MemoryAction::Delete { key, json } => handle_memory_delete(&repo, key, json),
    }
}

fn handle_memory_save(
    repo: &MemoryRepository,
    key: String,
    content: String,
    tag: Vec<String>,
    json: bool,
) -> Result<()> {
    let is_update = repo.get(&key).is_ok();

    let memory = if is_update {
        // Update existing memory
        let mut existing_memory = repo.get(&key)?;
        existing_memory.content = content;
        existing_memory.tags = tag;
        existing_memory.touch();
        existing_memory
    } else {
        // Create new memory
        Memory::new(key.clone())
            .with_content(content)
            .with_tags(tag)
    };

    let file_path = if is_update {
        repo.update(&memory)?
    } else {
        repo.create(&memory)?
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "key": memory.key,
                "file": file_path,
                "tags": memory.tags,
            }))?
        );
    } else {
        println!("{} {}", "Saved memory:".green(), memory.key);
        println!("  File: {}", file_path.display());
        if !memory.tags.is_empty() {
            println!("  Tags: {}", memory.tags.join(", "));
        }
    }

    Ok(())
}

fn handle_memory_query(repo: &MemoryRepository, key: String, json: bool) -> Result<()> {
    let memory = repo.get(&key)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "key": memory.key,
                "content": memory.content,
                "tags": memory.tags,
                "created": memory.created,
                "updated": memory.updated,
            }))?
        );
    } else {
        println!("{} {}", "Memory:".cyan().bold(), memory.key.bold());
        if !memory.tags.is_empty() {
            println!("  Tags: {}", memory.tags.join(", ").yellow());
        }
        println!("  Created: {}", memory.created.to_rfc3339());
        println!("  Updated: {}", memory.updated.to_rfc3339());
        println!();
        println!("{}", memory.content);
    }

    Ok(())
}

fn handle_memory_list(repo: &MemoryRepository, tag: Option<String>, json: bool) -> Result<()> {
    let memories = repo.list(tag.as_deref())?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "memories": memories.iter().map(|m| serde_json::json!({
                    "key": m.key,
                    "tags": m.tags,
                    "created": m.created,
                    "updated": m.updated,
                })).collect::<Vec<_>>(),
                "count": memories.len(),
            }))?
        );
    } else {
        if memories.is_empty() {
            println!("No memories found.");
        } else {
            println!("{} {} memories:", "Found".green(), memories.len());
            for memory in &memories {
                print!("  {} {}", "•".cyan(), memory.key.bold());
                if !memory.tags.is_empty() {
                    print!(" [{}]", memory.tags.join(", ").yellow());
                }
                println!();
            }
        }
    }

    Ok(())
}

fn handle_memory_edit(repo: &MemoryRepository, ctx: &CommandContext, key: String) -> Result<()> {
    let _memory = repo.get(&key)?;
    let memory_path = ctx
        .config
        .data_path(&ctx.root)
        .join("memory")
        .join(format!("{}.md", key));

    // Open in $EDITOR
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let status = std::process::Command::new(&editor)
        .arg(&memory_path)
        .status()?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }

    println!("{} {}", "Edited memory:".green(), key);

    Ok(())
}

fn handle_memory_delete(repo: &MemoryRepository, key: String, json: bool) -> Result<()> {
    repo.delete(&key)?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "deleted": key,
            }))?
        );
    } else {
        println!("{} {}", "Deleted memory:".red(), key);
    }

    Ok(())
}
