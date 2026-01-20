use anyhow::Result;
use peas::model::{PeaStatus, PeaType};

use super::CommandContext;

pub fn handle_context(ctx: &CommandContext) -> Result<()> {
    let peas = ctx.repo.list()?;

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
            "milestone": peas.iter().filter(|p| p.pea_type == PeaType::Milestone).count(),
            "epic": peas.iter().filter(|p| p.pea_type == PeaType::Epic).count(),
            "feature": peas.iter().filter(|p| p.pea_type == PeaType::Feature).count(),
            "bug": peas.iter().filter(|p| p.pea_type == PeaType::Bug).count(),
            "task": peas.iter().filter(|p| p.pea_type == PeaType::Task).count(),
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
