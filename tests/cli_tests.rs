use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn peas_cmd() -> Command {
    Command::cargo_bin("peas").unwrap()
}

#[test]
fn test_help() {
    peas_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("issue tracker"));
}

#[test]
fn test_version() {
    peas_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("peas"));
}

#[test]
fn test_init_creates_config() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));

    assert!(temp_dir.path().join(".peas.yml").exists());
    assert!(temp_dir.path().join(".peas").exists());
}

#[test]
fn test_init_with_custom_prefix() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .args(["init", "--prefix", "myapp-"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let config = std::fs::read_to_string(temp_dir.path().join(".peas.yml")).unwrap();
    assert!(config.contains("myapp-"));
}

#[test]
fn test_create_and_list() {
    let temp_dir = TempDir::new().unwrap();

    // Initialize
    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Create a pea
    peas_cmd()
        .args(["create", "Test Task", "-t", "task"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    // List should show the pea
    peas_cmd()
        .arg("list")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Task"));
}

#[test]
fn test_create_with_body() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args([
            "create",
            "Task with body",
            "-t",
            "task",
            "-d",
            "This is the body content",
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args(["list", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Task with body"));
}

#[test]
fn test_show_pea() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Create and capture ID
    let output = peas_cmd()
        .args(["create", "Show Test", "-t", "feature", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    // Show the pea
    peas_cmd()
        .args(["show", id])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Test"))
        .stdout(predicate::str::contains("feature"));
}

#[test]
fn test_update_status() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let output = peas_cmd()
        .args(["create", "Update Test", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    // Update status
    peas_cmd()
        .args(["update", id, "-s", "in-progress"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify status changed
    peas_cmd()
        .args(["show", id, "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("in-progress"));
}

#[test]
fn test_start_and_done() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let output = peas_cmd()
        .args(["create", "Workflow Test", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    // Start
    peas_cmd()
        .args(["start", id])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("in-progress"));

    // Done
    peas_cmd()
        .args(["done", id])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("completed"));
}

#[test]
fn test_search() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args(["create", "Searchable Task"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args(["create", "Another Item"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Search should find the first one
    peas_cmd()
        .args(["search", "Searchable"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Searchable Task"))
        .stdout(predicate::str::contains("1 results"));
}

#[test]
fn test_list_filter_by_type() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args(["create", "Epic One", "-t", "epic"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args(["create", "Task One", "-t", "task"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Filter by epic
    peas_cmd()
        .args(["list", "-t", "epic"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Epic One"))
        .stdout(predicate::str::contains("Task One").not());
}

#[test]
fn test_graphql_query() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args(["create", "GraphQL Test"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args(["graphql", "{ stats { total } }"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total\": 1"));
}

#[test]
fn test_archive() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let output = peas_cmd()
        .args(["create", "Archive Test", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    // Archive
    peas_cmd()
        .args(["archive", id])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived"));

    // Should not appear in normal list
    peas_cmd()
        .arg("list")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archive Test").not());

    // Should appear in archived list
    peas_cmd()
        .args(["list", "--archived"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archive Test"));
}

#[test]
fn test_prime_command() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .arg("prime")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Peas - Issue Tracker"))
        .stdout(predicate::str::contains("GraphQL Interface"));
}

#[test]
fn test_context_command() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args(["create", "Context Test"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .arg("context")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total\": 1"))
        .stdout(predicate::str::contains("\"by_status\""));
}

#[test]
fn test_not_initialized_error() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("list")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not initialized")
                .or(predicate::str::contains("Failed to load")),
        );
}
