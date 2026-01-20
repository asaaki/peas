use crate::{
    error::Result,
    model::{Pea, PeaPriority, PeaStatus, PeaType},
    storage::PeaRepository,
    undo::UndoManager,
};
use std::path::Path;

/// Generic function to apply a property change to multiple tickets
fn apply_property_change<T, F>(
    target_ids: &[String],
    all_peas: &[Pea],
    repo: &PeaRepository,
    data_path: &Path,
    _property_name: &str,
    new_value: T,
    mut update_fn: F,
) -> Result<String>
where
    T: std::fmt::Display + Copy,
    F: FnMut(&mut Pea, T),
{
    let count = target_ids.len();
    let undo_manager = UndoManager::new(data_path);

    for (i, id) in target_ids.iter().enumerate() {
        if let Some(pea) = all_peas.iter().find(|p| p.id == *id).cloned() {
            // Record undo for the last item (will be what gets undone)
            if i == count - 1 {
                if let Ok(path) = repo.find_file_by_id(&pea.id) {
                    let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
                }
            }
            let mut updated = pea;
            update_fn(&mut updated, new_value);
            updated.touch();
            repo.update(&updated)?;
        }
    }

    let message = if count > 1 {
        format!("{} tickets -> {}", count, new_value)
    } else if count == 1 {
        format!("-> {}", new_value)
    } else {
        String::new()
    };

    Ok(message)
}

/// Apply status change to target tickets
pub fn apply_status_change(
    target_ids: &[String],
    all_peas: &[Pea],
    repo: &PeaRepository,
    data_path: &Path,
    new_status: PeaStatus,
) -> Result<String> {
    apply_property_change(
        target_ids,
        all_peas,
        repo,
        data_path,
        "status",
        new_status,
        |pea, status| pea.status = status,
    )
}

/// Apply priority change to target tickets
pub fn apply_priority_change(
    target_ids: &[String],
    all_peas: &[Pea],
    repo: &PeaRepository,
    data_path: &Path,
    new_priority: PeaPriority,
) -> Result<String> {
    apply_property_change(
        target_ids,
        all_peas,
        repo,
        data_path,
        "priority",
        new_priority,
        |pea, priority| pea.priority = priority,
    )
}

/// Apply type change to target tickets
pub fn apply_type_change(
    target_ids: &[String],
    all_peas: &[Pea],
    repo: &PeaRepository,
    data_path: &Path,
    new_type: PeaType,
) -> Result<String> {
    apply_property_change(
        target_ids,
        all_peas,
        repo,
        data_path,
        "type",
        new_type,
        |pea, pea_type| pea.pea_type = pea_type,
    )
}

/// Apply parent change to a ticket
pub fn apply_parent_change(
    ticket_id: &str,
    all_peas: &[Pea],
    repo: &PeaRepository,
    data_path: &Path,
    new_parent: Option<String>,
) -> Result<String> {
    let undo_manager = UndoManager::new(data_path);

    if let Some(pea) = all_peas.iter().find(|p| p.id == ticket_id).cloned() {
        if let Ok(path) = repo.find_file_by_id(&pea.id) {
            let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
        }

        let mut updated = pea.clone();
        updated.parent = new_parent.clone();
        updated.touch();
        repo.update(&updated)?;

        let parent_display = new_parent.unwrap_or_else(|| "(none)".to_string());
        Ok(format!("{} parent -> {}", pea.id, parent_display))
    } else {
        Ok(String::new())
    }
}

/// Apply blocking changes to a ticket
pub fn apply_blocking_change(
    ticket_id: &str,
    all_peas: &[Pea],
    repo: &PeaRepository,
    data_path: &Path,
    new_blocking: Vec<String>,
) -> Result<String> {
    let undo_manager = UndoManager::new(data_path);

    if let Some(pea) = all_peas.iter().find(|p| p.id == ticket_id).cloned() {
        if let Ok(path) = repo.find_file_by_id(&pea.id) {
            let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
        }

        let count = new_blocking.len();
        let mut updated = pea.clone();
        updated.blocking = new_blocking;
        updated.touch();
        repo.update(&updated)?;

        Ok(format!("{} blocking {} tickets", pea.id, count))
    } else {
        Ok(String::new())
    }
}

/// Apply tags change to a ticket
pub fn apply_tags_change(
    ticket_id: &str,
    all_peas: &[Pea],
    repo: &PeaRepository,
    data_path: &Path,
    new_tags: Vec<String>,
) -> Result<()> {
    let undo_manager = UndoManager::new(data_path);

    if let Some(pea) = all_peas.iter().find(|p| p.id == ticket_id).cloned() {
        if let Ok(path) = repo.find_file_by_id(&pea.id) {
            let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
        }

        let mut updated = pea;
        updated.tags = new_tags;
        updated.touch();
        repo.update(&updated)?;
    }

    Ok(())
}
