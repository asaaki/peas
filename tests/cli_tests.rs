use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn peas_cmd() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("peas"))
}

// =============================================================================
// Basic CLI
// =============================================================================

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

// =============================================================================
// Initialization
// =============================================================================

#[test]
fn test_init_creates_config() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));

    assert!(temp_dir.path().join(".peas.toml").exists());
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

    let config = std::fs::read_to_string(temp_dir.path().join(".peas.toml")).unwrap();
    assert!(config.contains("myapp-"));
}

// =============================================================================
// Create, List, Show
// =============================================================================

#[test]
fn test_create_and_list() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args(["create", "Test Task", "-t", "task"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

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

    peas_cmd()
        .args(["list", "-t", "epic"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Epic One"))
        .stdout(predicate::str::contains("Task One").not());
}

#[test]
fn test_show_pea() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let output = peas_cmd()
        .args(["create", "Show Test", "-t", "feature", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    peas_cmd()
        .args(["show", id])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Show Test"))
        .stdout(predicate::str::contains("feature"));
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

    peas_cmd()
        .args(["search", "Searchable"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Searchable Task"))
        .stdout(predicate::str::contains("1 results"));
}

// =============================================================================
// Update, Status Workflow
// =============================================================================

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

    peas_cmd()
        .args(["update", id, "-s", "in-progress"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

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

    peas_cmd()
        .args(["start", id])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("in-progress"));

    peas_cmd()
        .args(["done", id])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("completed"));
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

    peas_cmd()
        .args(["archive", id])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archived"));

    peas_cmd()
        .arg("list")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archive Test").not());

    peas_cmd()
        .args(["list", "--archived"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Archive Test"));
}

// =============================================================================
// GraphQL
// =============================================================================

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
        .args(["query", "{ stats { total } }"])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total\": 1"));
}

#[test]
fn test_graphql_mutate() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    peas_cmd()
        .args([
            "mutate",
            "createPea(input: { title: \"Mutation Test\", peaType: TASK }) { id title }",
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Mutation Test"));

    peas_cmd()
        .arg("list")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Mutation Test"));
}

// =============================================================================
// LLM Context Commands
// =============================================================================

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

// =============================================================================
// Frontmatter Format
// =============================================================================

#[test]
fn test_toml_frontmatter_default() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let output = peas_cmd()
        .args(["create", "TOML Test", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    let data_dir = temp_dir.path().join(".peas");
    let entries: Vec<_> = std::fs::read_dir(&data_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .map(|n| n.to_string_lossy().starts_with(id))
                .unwrap_or(false)
        })
        .collect();

    assert_eq!(entries.len(), 1);
    let content = std::fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.starts_with("+++"),
        "Expected TOML frontmatter (+++), got: {}",
        &content[..50.min(content.len())]
    );
}

#[test]
fn test_yaml_frontmatter_config() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Switch config to YAML
    let config_path = temp_dir.path().join(".peas.toml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    let updated_config = config.replace("frontmatter = \"toml\"", "frontmatter = \"yaml\"");
    std::fs::write(&config_path, updated_config).unwrap();

    let output = peas_cmd()
        .args(["create", "YAML Test", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    let data_dir = temp_dir.path().join(".peas");
    let entries: Vec<_> = std::fs::read_dir(&data_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .map(|n| n.to_string_lossy().starts_with(id))
                .unwrap_or(false)
        })
        .collect();

    assert_eq!(entries.len(), 1);
    let content = std::fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.starts_with("---"),
        "Expected YAML frontmatter (---), got: {}",
        &content[..50.min(content.len())]
    );
}

#[test]
fn test_toml_frontmatter_preserved_on_update() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let output = peas_cmd()
        .args(["create", "Preserve TOML Format Test", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    // Switch config to YAML
    let config_path = temp_dir.path().join(".peas.toml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    let updated_config = config.replace("frontmatter = \"toml\"", "frontmatter = \"yaml\"");
    std::fs::write(&config_path, updated_config).unwrap();

    // Update the pea - should preserve TOML format
    peas_cmd()
        .args(["update", id, "-s", "in-progress"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let data_dir = temp_dir.path().join(".peas");
    let entries: Vec<_> = std::fs::read_dir(&data_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .map(|n| n.to_string_lossy().starts_with(id))
                .unwrap_or(false)
        })
        .collect();

    let content = std::fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.starts_with("+++"),
        "Expected TOML frontmatter to be preserved after update"
    );
}

#[test]
fn test_yaml_frontmatter_preserved_on_update() {
    let temp_dir = TempDir::new().unwrap();

    peas_cmd()
        .arg("init")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Switch config to YAML
    let config_path = temp_dir.path().join(".peas.toml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    let yaml_config = config.replace("frontmatter = \"toml\"", "frontmatter = \"yaml\"");
    std::fs::write(&config_path, &yaml_config).unwrap();

    let output = peas_cmd()
        .args(["create", "Preserve YAML Format Test", "--json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let id = json["id"].as_str().unwrap();

    // Switch config back to TOML
    std::fs::write(&config_path, &config).unwrap();

    // Update the pea - should preserve YAML format
    peas_cmd()
        .args(["update", id, "-s", "in-progress"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let data_dir = temp_dir.path().join(".peas");
    let entries: Vec<_> = std::fs::read_dir(&data_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .file_name()
                .map(|n| n.to_string_lossy().starts_with(id))
                .unwrap_or(false)
        })
        .collect();

    let content = std::fs::read_to_string(entries[0].path()).unwrap();
    assert!(
        content.starts_with("---"),
        "Expected YAML frontmatter to be preserved after update"
    );
}
