use crate::cli::commands::{PeaPriorityArg, PeaStatusArg, PeaTypeArg};
use crate::model::PeaStatus;
use anyhow::Result;

use super::CommandContext;
use super::utils::print_pea_list;

/// Parameters for list operation
pub struct ListParams {
    pub r#type: Option<PeaTypeArg>,
    pub status: Option<PeaStatusArg>,
    pub priority: Option<PeaPriorityArg>,
    pub parent: Option<String>,
    pub tag: Option<String>,
    pub archived: bool,
    pub json: bool,
}

pub fn handle_list(ctx: &CommandContext, params: ListParams) -> Result<()> {
    let mut peas = if params.archived {
        ctx.repo.list_archived()?
    } else {
        ctx.repo.list()?
    };

    // Apply filters
    if let Some(t) = params.r#type {
        let filter_type = t.into();
        peas.retain(|p| p.pea_type == filter_type);
    }
    if let Some(s) = params.status {
        let filter_status: PeaStatus = s.into();
        peas.retain(|p| p.status == filter_status);
    }
    if let Some(p) = params.priority {
        let filter_priority = p.into();
        peas.retain(|p| p.priority == filter_priority);
    }
    if let Some(ref parent_id) = params.parent {
        peas.retain(|p| p.parent.as_deref() == Some(parent_id.as_str()));
    }
    if let Some(ref t) = params.tag {
        peas.retain(|p| p.tags.contains(t));
    }

    if params.json {
        println!("{}", serde_json::to_string_pretty(&peas)?);
    } else {
        print_pea_list(&peas);
    }
    Ok(())
}
