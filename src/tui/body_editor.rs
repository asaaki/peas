use crate::{error::Result, model::Pea, storage::PeaRepository, undo::UndoManager};
use std::path::Path;
use tui_textarea::TextArea;

/// Initialize a TextArea for editing a pea's body
pub fn create_textarea(body: &str) -> TextArea<'static> {
    // Split body into lines for TextArea
    let lines: Vec<String> = body.lines().map(|s| s.to_string()).collect();
    let mut textarea = TextArea::new(lines);

    // Configure textarea
    textarea.set_tab_length(2);
    textarea.set_max_histories(100); // Undo/redo buffer

    textarea
}

/// Save edited body content to a pea
pub fn save_body(
    textarea: &TextArea,
    pea: &Pea,
    repo: &PeaRepository,
    data_path: &Path,
) -> Result<()> {
    // Get edited content
    let new_body = textarea.lines().join("\n");

    // Record undo before update
    let undo_manager = UndoManager::new(data_path);
    if let Ok(path) = repo.find_file_by_id(&pea.id) {
        let _ = crate::undo::record_update(&undo_manager, &pea.id, &path);
    }

    // Update pea
    let mut updated = pea.clone();
    updated.body = new_body;
    // NOTE: No touch() call - update() handles it internally now
    repo.update(&mut updated)?;

    Ok(())
}
