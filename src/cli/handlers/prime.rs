use crate::model::PeaStatus;
use anyhow::Result;

use super::CommandContext;

pub fn handle_prime(ctx: &CommandContext) -> Result<()> {
    let peas = ctx.repo.list()?;
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
peas suggest                       # Get next suggested ticket to work on
```

**Working on multiple tasks?** Use `peas suggest` to get the next recommended ticket based on priority, blocking relationships, and work queue. This helps maintain focus during longer work sessions.

## Memory System (Knowledge Base)

**IMPORTANT**: Use the memory system to capture important learnings, facts, decisions, and context as you work. This helps maintain continuity across sessions and builds institutional knowledge.

```bash
peas memory save <key> "<content>" --tag <tag1> --tag <tag2>  # Save a memory
peas memory query <key>                                       # Retrieve a memory
peas memory list                                             # List all memories
peas memory list --tag <tag>                                 # Filter by tag
peas memory edit <key>                                       # Edit in $EDITOR
peas memory delete <key>                                     # Delete a memory
```

**When to create memories:**
- Architecture decisions and their rationale
- Important API patterns or conventions discovered
- Tricky bugs and their solutions
- Performance optimizations and benchmarks
- Security considerations and threat models
- Project-specific gotchas and learnings
- Configuration details and environment setup
- Third-party integration patterns
- Testing strategies that work well

**Memory best practices:**
- Use descriptive keys (e.g., "auth-flow", "database-schema", "ci-pipeline")
- Tag memories for easy discovery (e.g., "architecture", "security", "performance")
- Keep memories focused and actionable
- Update memories when learnings evolve
- Reference ticket IDs when relevant

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
        ctx.config.peas.path
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
