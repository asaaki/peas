use crate::cli::commands::{PeaPriorityArg, PeaStatusArg, PeaTypeArg};
use crate::model::PeaStatus;
use anyhow::Result;

use super::CommandContext;
use super::utils::print_pea_list;

pub fn handle_list(
    ctx: &CommandContext,
    r#type: Option<PeaTypeArg>,
    status: Option<PeaStatusArg>,
    priority: Option<PeaPriorityArg>,
    parent: Option<String>,
    tag: Option<String>,
    archived: bool,
    json: bool,
) -> Result<()> {
    let mut peas = if archived {
        ctx.repo.list_archived()?
    } else {
        ctx.repo.list()?
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
