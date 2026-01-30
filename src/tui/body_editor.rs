use crate::{error::Result, model::Pea, storage::PeaRepository, undo::UndoManager};
use rat_text::text_area::TextAreaState;
use rat_text::undo_buffer::UndoVec;
use std::path::Path;

/// Initialize a TextAreaState for editing a pea's body
pub fn create_textarea(body: &str) -> TextAreaState {
    let mut state = TextAreaState::default();

    // Set the body text
    state.set_text(body);

    // Configure textarea
    state.set_tab_width(2);
    state.set_expand_tabs(true); // Convert tabs to spaces
    state.set_undo_buffer(Some(UndoVec::new(100))); // Undo/redo buffer with 100 entries

    state
}

/// Save edited body content to a pea
pub fn save_body(
    textarea: &TextAreaState,
    pea: &Pea,
    repo: &PeaRepository,
    data_path: &Path,
) -> Result<()> {
    // Get edited content
    let new_body = textarea.value();

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
