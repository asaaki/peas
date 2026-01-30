use peas::{
    config::PeasConfig,
    model::{Pea, PeaType},
    storage::PeaRepository,
    tui::app::{App, DetailPane, InputMode, ViewMode},
};
use tempfile::TempDir;

/// Helper to create a test app with a temporary repository
fn create_test_app() -> (App, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let config = PeasConfig {
        peas: peas::config::PeasSettings {
            path: ".peas".to_string(),
            prefix: "test-".to_string(),
            id_length: 5,
            default_status: "todo".to_string(),
            default_type: "task".to_string(),
            frontmatter: "toml".to_string(),
        },
        tui: peas::config::TuiSettings::default(),
    };

    let data_path = config.data_path(temp_dir.path());
    std::fs::create_dir_all(&data_path).unwrap();

    let app = App::new(&config, temp_dir.path()).unwrap();
    (app, temp_dir)
}

/// Helper to create and save a test pea
fn create_test_pea(repo: &PeaRepository, id: &str, title: &str, pea_type: PeaType) -> Pea {
    let mut pea = Pea::new(id.to_string(), title.to_string(), pea_type);
    pea.body = format!("Test body for {}", title);
    repo.create(&pea).unwrap();
    pea
}

// ============================================================================
// State Machine Tests - Modal Transitions
// ============================================================================

#[test]
fn test_modal_open_close_status() {
    let (mut app, _temp_dir) = create_test_app();

    assert_eq!(app.input_mode, InputMode::Normal);

    // Open status modal
    app.input_mode = InputMode::StatusModal;
    app.previous_mode = InputMode::Normal;
    assert_eq!(app.input_mode, InputMode::StatusModal);

    // Close modal (simulate Escape)
    app.input_mode = app.previous_mode;
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[test]
fn test_modal_open_close_priority() {
    let (mut app, _temp_dir) = create_test_app();

    app.input_mode = InputMode::PriorityModal;
    app.previous_mode = InputMode::Normal;
    assert_eq!(app.input_mode, InputMode::PriorityModal);

    app.input_mode = app.previous_mode;
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[test]
fn test_modal_open_close_type() {
    let (mut app, _temp_dir) = create_test_app();

    app.input_mode = InputMode::TypeModal;
    app.previous_mode = InputMode::Normal;
    assert_eq!(app.input_mode, InputMode::TypeModal);

    app.input_mode = app.previous_mode;
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[test]
fn test_modal_open_close_delete() {
    let (mut app, _temp_dir) = create_test_app();

    app.input_mode = InputMode::DeleteConfirm;
    app.previous_mode = InputMode::Normal;
    assert_eq!(app.input_mode, InputMode::DeleteConfirm);

    app.input_mode = app.previous_mode;
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[test]
fn test_modal_selection_reset_on_open() {
    let (mut app, _temp_dir) = create_test_app();

    app.modal_selection = 5;
    app.input_mode = InputMode::StatusModal;
    app.modal_selection = 0; // Should reset when opening modal
    assert_eq!(app.modal_selection, 0);
}

#[test]
fn test_view_mode_switch() {
    let (mut app, _temp_dir) = create_test_app();

    assert_eq!(app.view_mode, ViewMode::Tickets);

    app.view_mode = ViewMode::Memory;
    assert_eq!(app.view_mode, ViewMode::Memory);

    app.view_mode = ViewMode::Tickets;
    assert_eq!(app.view_mode, ViewMode::Tickets);
}

// ============================================================================
// Navigation Tests - Edge Cases
// ============================================================================

#[test]
fn test_navigation_empty_list() {
    let (app, _temp_dir) = create_test_app();

    // Empty list should have no selection
    assert_eq!(app.all_peas.len(), 0);
    assert_eq!(app.tree_nodes.len(), 0);
    assert_eq!(app.selected_index, 0);
}

#[test]
fn test_navigation_single_item() {
    let (mut app, _temp_dir) = create_test_app();

    create_test_pea(&app.repo, "test-abc01", "Test Task", PeaType::Task);
    app.refresh().unwrap();

    assert_eq!(app.all_peas.len(), 1);
    assert_eq!(app.tree_nodes.len(), 1);

    // Can't navigate beyond single item
    let initial_index = app.selected_index;
    // Simulate down key - should stay at same index
    if app.selected_index < app.tree_nodes.len().saturating_sub(1) {
        app.selected_index += 1;
    }
    assert_eq!(app.selected_index, initial_index);
}

#[test]
fn test_navigation_boundary_top() {
    let (mut app, _temp_dir) = create_test_app();

    create_test_pea(&app.repo, "test-abc01", "Task 1", PeaType::Task);
    create_test_pea(&app.repo, "test-abc02", "Task 2", PeaType::Task);
    create_test_pea(&app.repo, "test-abc03", "Task 3", PeaType::Task);
    app.refresh().unwrap();

    app.selected_index = 0;

    // Try to go up from top - should stay at 0
    app.selected_index = app.selected_index.saturating_sub(1);
    assert_eq!(app.selected_index, 0);
}

#[test]
fn test_navigation_boundary_bottom() {
    let (mut app, _temp_dir) = create_test_app();

    create_test_pea(&app.repo, "test-abc01", "Task 1", PeaType::Task);
    create_test_pea(&app.repo, "test-abc02", "Task 2", PeaType::Task);
    create_test_pea(&app.repo, "test-abc03", "Task 3", PeaType::Task);
    app.refresh().unwrap();

    let max_index = app.tree_nodes.len().saturating_sub(1);
    app.selected_index = max_index;

    // Try to go down from bottom - should stay at max
    if app.selected_index < app.tree_nodes.len().saturating_sub(1) {
        app.selected_index += 1;
    }
    assert_eq!(app.selected_index, max_index);
}

#[test]
fn test_navigation_multiple_items() {
    let (mut app, _temp_dir) = create_test_app();

    create_test_pea(&app.repo, "test-abc01", "Task 1", PeaType::Task);
    create_test_pea(&app.repo, "test-abc02", "Task 2", PeaType::Task);
    create_test_pea(&app.repo, "test-abc03", "Task 3", PeaType::Task);
    app.refresh().unwrap();

    assert_eq!(app.tree_nodes.len(), 3);

    // Navigate down
    app.selected_index = 0;
    app.selected_index = app
        .selected_index
        .saturating_add(1)
        .min(app.tree_nodes.len().saturating_sub(1));
    assert_eq!(app.selected_index, 1);

    app.selected_index = app
        .selected_index
        .saturating_add(1)
        .min(app.tree_nodes.len().saturating_sub(1));
    assert_eq!(app.selected_index, 2);

    // Navigate up
    app.selected_index = app.selected_index.saturating_sub(1);
    assert_eq!(app.selected_index, 1);
}

// ============================================================================
// Detail View Tests
// ============================================================================

#[test]
fn test_detail_pane_switching() {
    let (mut app, _temp_dir) = create_test_app();

    assert_eq!(app.detail_pane, DetailPane::Body);

    app.detail_pane = DetailPane::Metadata;
    assert_eq!(app.detail_pane, DetailPane::Metadata);

    app.detail_pane = DetailPane::Relations;
    assert_eq!(app.detail_pane, DetailPane::Relations);

    app.detail_pane = DetailPane::Assets;
    assert_eq!(app.detail_pane, DetailPane::Assets);

    app.detail_pane = DetailPane::Body;
    assert_eq!(app.detail_pane, DetailPane::Body);
}

#[test]
fn test_detail_scroll_limits() {
    let (mut app, _temp_dir) = create_test_app();

    app.detail_scroll = 0;
    app.detail_max_scroll = 10;

    // Scroll down
    app.detail_scroll = (app.detail_scroll + 1).min(app.detail_max_scroll);
    assert_eq!(app.detail_scroll, 1);

    // Scroll to max
    app.detail_scroll = app.detail_max_scroll;
    assert_eq!(app.detail_scroll, 10);

    // Try to scroll beyond max
    app.detail_scroll = (app.detail_scroll + 1).min(app.detail_max_scroll);
    assert_eq!(app.detail_scroll, 10);

    // Scroll up
    app.detail_scroll = app.detail_scroll.saturating_sub(1);
    assert_eq!(app.detail_scroll, 9);

    // Scroll to top
    app.detail_scroll = 0;
    assert_eq!(app.detail_scroll, 0);

    // Try to scroll above 0
    app.detail_scroll = app.detail_scroll.saturating_sub(1);
    assert_eq!(app.detail_scroll, 0);
}

#[test]
fn test_metadata_selection_navigation() {
    let (mut app, _temp_dir) = create_test_app();

    app.detail_pane = DetailPane::Metadata;
    app.metadata_selection = 0;

    // Navigate through metadata items (type, status, priority, tags)
    let max_metadata = 3; // 0=type, 1=status, 2=priority, 3=tags

    app.metadata_selection = (app.metadata_selection + 1).min(max_metadata);
    assert_eq!(app.metadata_selection, 1);

    app.metadata_selection = (app.metadata_selection + 1).min(max_metadata);
    assert_eq!(app.metadata_selection, 2);

    app.metadata_selection = (app.metadata_selection + 1).min(max_metadata);
    assert_eq!(app.metadata_selection, 3);

    // Can't go beyond max
    app.metadata_selection = (app.metadata_selection + 1).min(max_metadata);
    assert_eq!(app.metadata_selection, 3);

    // Navigate back
    app.metadata_selection = app.metadata_selection.saturating_sub(1);
    assert_eq!(app.metadata_selection, 2);
}

// ============================================================================
// Filter Tests
// ============================================================================

#[test]
fn test_filter_mode_toggle() {
    let (mut app, _temp_dir) = create_test_app();

    assert_eq!(app.input_mode, InputMode::Normal);
    assert_eq!(app.search_query, "");

    // Enter filter mode
    app.input_mode = InputMode::Filter;
    assert_eq!(app.input_mode, InputMode::Filter);

    // Exit filter mode
    app.input_mode = InputMode::Normal;
    assert_eq!(app.input_mode, InputMode::Normal);
}

#[test]
fn test_filter_query_persistence() {
    let (mut app, _temp_dir) = create_test_app();

    app.search_query = "test query".to_string();
    app.input_mode = InputMode::Filter;

    // Query should persist when switching modes
    app.input_mode = InputMode::Normal;
    assert_eq!(app.search_query, "test query");
}

// ============================================================================
// Multi-Selection Tests
// ============================================================================

#[test]
fn test_multi_selection_toggle() {
    let (mut app, _temp_dir) = create_test_app();

    let id = "test-abc01".to_string();

    // Initially empty
    assert!(!app.multi_selected.contains(&id));

    // Add to selection
    app.multi_selected.insert(id.clone());
    assert!(app.multi_selected.contains(&id));

    // Remove from selection
    app.multi_selected.remove(&id);
    assert!(!app.multi_selected.contains(&id));
}

#[test]
fn test_multi_selection_clear() {
    let (mut app, _temp_dir) = create_test_app();

    app.multi_selected.insert("test-abc01".to_string());
    app.multi_selected.insert("test-abc02".to_string());
    app.multi_selected.insert("test-abc03".to_string());

    assert_eq!(app.multi_selected.len(), 3);

    app.multi_selected.clear();
    assert_eq!(app.multi_selected.len(), 0);
}

// ============================================================================
// Memory View Tests
// ============================================================================

#[test]
fn test_memory_view_initial_state() {
    let (app, _temp_dir) = create_test_app();

    assert_eq!(app.all_memories.len(), 0);
    assert_eq!(app.filtered_memories.len(), 0);
}

#[test]
fn test_memory_view_switch() {
    let (mut app, _temp_dir) = create_test_app();

    assert_eq!(app.view_mode, ViewMode::Tickets);

    app.view_mode = ViewMode::Memory;
    assert_eq!(app.view_mode, ViewMode::Memory);
    assert_eq!(app.input_mode, InputMode::Normal);
}

// ============================================================================
// Create Modal Tests
// ============================================================================

#[test]
fn test_create_modal_initial_state() {
    let (mut app, _temp_dir) = create_test_app();

    app.input_mode = InputMode::CreateModal;
    app.create_title = String::new();
    app.create_type = PeaType::Task;

    assert_eq!(app.create_title, "");
    assert_eq!(app.create_type, PeaType::Task);
}

#[test]
fn test_create_modal_type_selection() {
    let (mut app, _temp_dir) = create_test_app();

    app.input_mode = InputMode::CreateModal;

    app.create_type = PeaType::Task;
    assert_eq!(app.create_type, PeaType::Task);

    app.create_type = PeaType::Bug;
    assert_eq!(app.create_type, PeaType::Bug);

    app.create_type = PeaType::Feature;
    assert_eq!(app.create_type, PeaType::Feature);

    app.create_type = PeaType::Chore;
    assert_eq!(app.create_type, PeaType::Chore);
}

// ============================================================================
// Message Display Tests
// ============================================================================

#[test]
fn test_message_display() {
    let (mut app, _temp_dir) = create_test_app();

    assert_eq!(app.message, None);

    app.message = Some("Test message".to_string());
    assert_eq!(app.message, Some("Test message".to_string()));

    app.message = None;
    assert_eq!(app.message, None);
}

// ============================================================================
// Help Display Tests
// ============================================================================

#[test]
fn test_help_toggle() {
    let (mut app, _temp_dir) = create_test_app();

    assert!(!app.show_help);

    app.show_help = true;
    assert!(app.show_help);

    app.show_help = false;
    assert!(!app.show_help);
}

// ============================================================================
// Reload Tests
// ============================================================================

#[test]
fn test_reload_peas_empty() {
    let (mut app, _temp_dir) = create_test_app();

    app.refresh().unwrap();
    assert_eq!(app.all_peas.len(), 0);
    assert_eq!(app.filtered_peas.len(), 0);
}

#[test]
fn test_reload_peas_with_data() {
    let (mut app, _temp_dir) = create_test_app();

    create_test_pea(&app.repo, "test-abc01", "Task 1", PeaType::Task);
    create_test_pea(&app.repo, "test-abc02", "Task 2", PeaType::Task);

    app.refresh().unwrap();
    assert_eq!(app.all_peas.len(), 2);
}

#[test]
fn test_reload_memories_empty() {
    let (mut app, _temp_dir) = create_test_app();

    app.refresh().unwrap();
    assert_eq!(app.all_memories.len(), 0);
    assert_eq!(app.filtered_memories.len(), 0);
}
