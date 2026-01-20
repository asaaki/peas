use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn peas_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("peas"))
}

fn setup_test_project() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Initialize peas project
    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    temp_dir
}

// =============================================================================
// Memory CRUD Operations
// =============================================================================

#[test]
fn test_memory_save_and_query() {
    let temp_dir = setup_test_project();

    // Save a memory
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("test-key")
        .arg("This is test content")
        .arg("--tag")
        .arg("test")
        .arg("--tag")
        .arg("example")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Saved memory: test-key"));

    // Query the memory
    peas_cmd()
        .arg("memory")
        .arg("query")
        .arg("test-key")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("This is test content"))
        .stdout(predicate::str::contains("test"))
        .stdout(predicate::str::contains("example"));
}

#[test]
fn test_memory_save_creates_file() {
    let temp_dir = setup_test_project();

    // Save a memory
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("file-test")
        .arg("Content here")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Check that file was created
    let memory_file = temp_dir.path().join(".peas/memory/file-test.md");
    assert!(memory_file.exists());

    // Check file content
    let content = fs::read_to_string(&memory_file).unwrap();
    assert!(content.contains("key = \"file-test\""));
    assert!(content.contains("Content here"));
}

#[test]
fn test_memory_list() {
    let temp_dir = setup_test_project();

    // Create multiple memories
    for i in 1..=3 {
        peas_cmd()
            .arg("memory")
            .arg("save")
            .arg(format!("memory-{}", i))
            .arg(format!("Content {}", i))
            .current_dir(temp_dir.path())
            .assert()
            .success();
    }

    // List memories
    peas_cmd()
        .arg("memory")
        .arg("list")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("memory-1"))
        .stdout(predicate::str::contains("memory-2"))
        .stdout(predicate::str::contains("memory-3"));
}

#[test]
fn test_memory_list_with_tag_filter() {
    let temp_dir = setup_test_project();

    // Create memories with different tags
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("tagged-1")
        .arg("Content 1")
        .arg("--tag")
        .arg("important")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("tagged-2")
        .arg("Content 2")
        .arg("--tag")
        .arg("other")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // List memories with tag filter
    peas_cmd()
        .arg("memory")
        .arg("list")
        .arg("--tag")
        .arg("important")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("tagged-1"))
        .stdout(predicate::str::contains("tagged-2").not());
}

#[test]
fn test_memory_delete() {
    let temp_dir = setup_test_project();

    // Create a memory
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("to-delete")
        .arg("Will be deleted")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify it exists
    let memory_file = temp_dir.path().join(".peas/memory/to-delete.md");
    assert!(memory_file.exists());

    // Delete it
    peas_cmd()
        .arg("memory")
        .arg("delete")
        .arg("to-delete")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted"));

    // Verify it's gone
    assert!(!memory_file.exists());
}

#[test]
fn test_memory_json_output() {
    let temp_dir = setup_test_project();

    // Create a memory
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("json-test")
        .arg("JSON content")
        .arg("--tag")
        .arg("json")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Query with JSON output
    peas_cmd()
        .arg("memory")
        .arg("query")
        .arg("json-test")
        .arg("--json")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"key\""))
        .stdout(predicate::str::contains("\"content\""))
        .stdout(predicate::str::contains("\"tags\""));
}

// =============================================================================
// Validation and Error Handling
// =============================================================================

#[test]
fn test_memory_invalid_key_characters() {
    let temp_dir = setup_test_project();

    // Try to create memory with invalid characters
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("invalid/key")
        .arg("Content")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid").or(predicate::str::contains("Key")));
}

#[test]
fn test_memory_query_nonexistent() {
    let temp_dir = setup_test_project();

    // Query non-existent memory
    peas_cmd()
        .arg("memory")
        .arg("query")
        .arg("does-not-exist")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("No such")));
}

#[test]
fn test_memory_update_existing() {
    let temp_dir = setup_test_project();

    // Create a memory
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("update-test")
        .arg("First content")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Update it with new content (save command acts as upsert)
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("update-test")
        .arg("Updated content")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify the content was updated
    peas_cmd()
        .arg("memory")
        .arg("query")
        .arg("update-test")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated content"))
        .stdout(predicate::str::contains("First content").not());
}

// =============================================================================
// Memory Content and Tags
// =============================================================================

#[test]
fn test_memory_multiple_tags() {
    let temp_dir = setup_test_project();

    // Create memory with multiple tags
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("multi-tag")
        .arg("Content with tags")
        .arg("--tag")
        .arg("tag1")
        .arg("--tag")
        .arg("tag2")
        .arg("--tag")
        .arg("tag3")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Query and verify tags
    peas_cmd()
        .arg("memory")
        .arg("query")
        .arg("multi-tag")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("tag1"))
        .stdout(predicate::str::contains("tag2"))
        .stdout(predicate::str::contains("tag3"));
}

#[test]
fn test_memory_multiline_content() {
    let temp_dir = setup_test_project();

    let multiline_content = "Line 1\nLine 2\nLine 3";

    // Create memory with multiline content
    peas_cmd()
        .arg("memory")
        .arg("save")
        .arg("multiline")
        .arg(multiline_content)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Query and verify all lines are preserved
    peas_cmd()
        .arg("memory")
        .arg("query")
        .arg("multiline")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Line 1"))
        .stdout(predicate::str::contains("Line 2"))
        .stdout(predicate::str::contains("Line 3"));
}
