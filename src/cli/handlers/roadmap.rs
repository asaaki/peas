use crate::model::{PeaStatus, PeaType};
use anyhow::Result;

use super::CommandContext;

pub fn handle_roadmap(ctx: &CommandContext) -> Result<()> {
    let peas = ctx.repo.list()?;
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
