use crate::model::{Pea, PeaType};

/// A relationship item for display (relationship type, id, title, pea_type)
pub type RelationItem = (String, String, String, PeaType);

/// Build the relationships list for a pea
pub fn build_relations(pea: &Pea, all_peas: &[Pea]) -> Vec<RelationItem> {
    let mut relations_items = Vec::new();

    // Add parent if exists
    if let Some(ref parent_id) = pea.parent
        && let Some(parent) = all_peas.iter().find(|p| p.id == *parent_id) {
            relations_items.push((
                "Parent".to_string(),
                parent.id.clone(),
                parent.title.clone(),
                parent.pea_type,
            ));
        }

    // Add blocking tickets
    for id in &pea.blocking {
        if let Some(blocked) = all_peas.iter().find(|p| p.id == *id) {
            relations_items.push((
                "Blocks".to_string(),
                blocked.id.clone(),
                blocked.title.clone(),
                blocked.pea_type,
            ));
        }
    }

    // Add children
    let children: Vec<_> = all_peas
        .iter()
        .filter(|p| p.parent.as_ref() == Some(&pea.id))
        .collect();
    for child in children {
        relations_items.push((
            "Child".to_string(),
            child.id.clone(),
            child.title.clone(),
            child.pea_type,
        ));
    }

    // Add blocked-by (reverse blocking relationships)
    let blocked_by: Vec<_> = all_peas
        .iter()
        .filter(|p| p.blocking.contains(&pea.id))
        .collect();
    for blocker in blocked_by {
        relations_items.push((
            "BlockedBy".to_string(),
            blocker.id.clone(),
            blocker.title.clone(),
            blocker.pea_type,
        ));
    }

    relations_items
}
