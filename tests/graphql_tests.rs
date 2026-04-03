use peas::config::PeasConfig;
use peas::graphql::build_schema;
use tempfile::TempDir;

fn setup_project() -> (TempDir, peas::graphql::PeasSchema) {
    let temp_dir = TempDir::new().unwrap();

    // Initialize a peas project in the temp dir
    let config = PeasConfig::default();
    let data_dir = temp_dir.path().join(".peas");
    std::fs::create_dir_all(&data_dir).unwrap();
    config.save(&data_dir.join("config.toml")).unwrap();

    let schema = build_schema(config, temp_dir.path().to_path_buf());
    (temp_dir, schema)
}

#[tokio::test]
async fn test_stats_empty_project() {
    let (_temp_dir, schema) = setup_project();

    let res = schema
        .execute("{ stats { total byStatus { todo inProgress completed } } }")
        .await;

    assert!(res.errors.is_empty(), "errors: {:?}", res.errors);
    let data = res.data.into_json().unwrap();
    assert_eq!(data["stats"]["total"], 0);
    assert_eq!(data["stats"]["byStatus"]["todo"], 0);
}

#[tokio::test]
async fn test_create_and_query_pea() {
    let (_temp_dir, schema) = setup_project();

    // Create a pea
    let res = schema
        .execute(
            r#"mutation { createPea(input: { title: "Test task" }) { id title peaType status } }"#,
        )
        .await;
    assert!(res.errors.is_empty(), "create errors: {:?}", res.errors);
    let data = res.data.into_json().unwrap();
    let id = data["createPea"]["id"].as_str().unwrap().to_string();
    assert_eq!(data["createPea"]["title"], "Test task");
    assert_eq!(data["createPea"]["peaType"], "TASK");
    assert_eq!(data["createPea"]["status"], "TODO");

    // Query the pea by ID
    let query = format!(r#"{{ pea(id: "{}") {{ id title }} }}"#, id);
    let res = schema.execute(&query).await;
    assert!(res.errors.is_empty(), "query errors: {:?}", res.errors);
    let data = res.data.into_json().unwrap();
    assert_eq!(data["pea"]["title"], "Test task");
}

#[tokio::test]
async fn test_create_pea_with_options() {
    let (_temp_dir, schema) = setup_project();

    let res = schema
        .execute(
            r#"mutation {
                createPea(input: {
                    title: "Bug fix",
                    peaType: BUG,
                    priority: HIGH,
                    body: "Fix the thing",
                    tags: ["urgent"]
                }) { id title peaType priority body tags }
            }"#,
        )
        .await;
    assert!(res.errors.is_empty(), "errors: {:?}", res.errors);
    let data = res.data.into_json().unwrap();
    assert_eq!(data["createPea"]["peaType"], "BUG");
    assert_eq!(data["createPea"]["priority"], "HIGH");
    assert_eq!(data["createPea"]["body"], "Fix the thing");
    assert_eq!(data["createPea"]["tags"][0], "urgent");
}

#[tokio::test]
async fn test_update_pea() {
    let (_temp_dir, schema) = setup_project();

    // Create
    let res = schema
        .execute(r#"mutation { createPea(input: { title: "Original" }) { id } }"#)
        .await;
    let data = res.data.into_json().unwrap();
    let id = data["createPea"]["id"].as_str().unwrap().to_string();

    // Update
    let mutation = format!(
        r#"mutation {{ updatePea(input: {{ id: "{}", title: "Updated", status: IN_PROGRESS, addTags: ["done"] }}) {{ id title status tags }} }}"#,
        id
    );
    let res = schema.execute(&mutation).await;
    assert!(res.errors.is_empty(), "update errors: {:?}", res.errors);
    let data = res.data.into_json().unwrap();
    assert_eq!(data["updatePea"]["title"], "Updated");
    assert_eq!(data["updatePea"]["status"], "IN_PROGRESS");
    assert_eq!(data["updatePea"]["tags"][0], "done");
}

#[tokio::test]
async fn test_set_status() {
    let (_temp_dir, schema) = setup_project();

    let res = schema
        .execute(r#"mutation { createPea(input: { title: "Status test" }) { id } }"#)
        .await;
    let data = res.data.into_json().unwrap();
    let id = data["createPea"]["id"].as_str().unwrap().to_string();

    let mutation = format!(
        r#"mutation {{ setStatus(id: "{}", status: COMPLETED) {{ id status }} }}"#,
        id
    );
    let res = schema.execute(&mutation).await;
    assert!(res.errors.is_empty(), "errors: {:?}", res.errors);
    let data = res.data.into_json().unwrap();
    assert_eq!(data["setStatus"]["status"], "COMPLETED");
}

#[tokio::test]
async fn test_list_with_filter() {
    let (_temp_dir, schema) = setup_project();

    // Create a bug and a task
    schema
        .execute(r#"mutation { createPea(input: { title: "A bug", peaType: BUG }) { id } }"#)
        .await;
    schema
        .execute(r#"mutation { createPea(input: { title: "A task", peaType: TASK }) { id } }"#)
        .await;

    // Filter by type
    let res = schema
        .execute(r#"{ peas(filter: { peaType: BUG }) { nodes { title } totalCount } }"#)
        .await;
    assert!(res.errors.is_empty());
    let data = res.data.into_json().unwrap();
    assert_eq!(data["peas"]["totalCount"], 1);
    assert_eq!(data["peas"]["nodes"][0]["title"], "A bug");
}

#[tokio::test]
async fn test_search() {
    let (_temp_dir, schema) = setup_project();

    schema
        .execute(r#"mutation { createPea(input: { title: "Fix login page", body: "The login form is broken" }) { id } }"#)
        .await;
    schema
        .execute(r#"mutation { createPea(input: { title: "Add feature" }) { id } }"#)
        .await;

    let res = schema
        .execute(r#"{ search(query: "login") { id title } }"#)
        .await;
    assert!(res.errors.is_empty());
    let data = res.data.into_json().unwrap();
    assert_eq!(data["search"].as_array().unwrap().len(), 1);
    assert_eq!(data["search"][0]["title"], "Fix login page");
}

#[tokio::test]
async fn test_query_nonexistent_pea() {
    let (_temp_dir, schema) = setup_project();

    let res = schema.execute(r#"{ pea(id: "nonexistent") { id } }"#).await;
    assert!(res.errors.is_empty());
    let data = res.data.into_json().unwrap();
    assert!(data["pea"].is_null());
}

#[tokio::test]
async fn test_delete_pea() {
    let (_temp_dir, schema) = setup_project();

    let res = schema
        .execute(r#"mutation { createPea(input: { title: "To delete" }) { id } }"#)
        .await;
    let data = res.data.into_json().unwrap();
    let id = data["createPea"]["id"].as_str().unwrap().to_string();

    let mutation = format!(r#"mutation {{ deletePea(id: "{}") }}"#, id);
    let res = schema.execute(&mutation).await;
    assert!(res.errors.is_empty(), "delete errors: {:?}", res.errors);

    // Verify it's gone
    let query = format!(r#"{{ pea(id: "{}") {{ id }} }}"#, id);
    let res = schema.execute(&query).await;
    let data = res.data.into_json().unwrap();
    assert!(data["pea"].is_null());
}

#[tokio::test]
async fn test_archive_pea() {
    let (_temp_dir, schema) = setup_project();

    let res = schema
        .execute(r#"mutation { createPea(input: { title: "To archive" }) { id } }"#)
        .await;
    let data = res.data.into_json().unwrap();
    let id = data["createPea"]["id"].as_str().unwrap().to_string();

    let mutation = format!(r#"mutation {{ archivePea(id: "{}") }}"#, id);
    let res = schema.execute(&mutation).await;
    assert!(res.errors.is_empty(), "archive errors: {:?}", res.errors);
}

#[tokio::test]
async fn test_children_query() {
    let (_temp_dir, schema) = setup_project();

    // Create parent
    let res = schema
        .execute(r#"mutation { createPea(input: { title: "Parent", peaType: EPIC }) { id } }"#)
        .await;
    let data = res.data.into_json().unwrap();
    let parent_id = data["createPea"]["id"].as_str().unwrap().to_string();

    // Create child
    let mutation = format!(
        r#"mutation {{ createPea(input: {{ title: "Child", parent: "{}" }}) {{ id }} }}"#,
        parent_id
    );
    schema.execute(&mutation).await;

    // Query children
    let query = format!(r#"{{ children(parentId: "{}") {{ title }} }}"#, parent_id);
    let res = schema.execute(&query).await;
    assert!(res.errors.is_empty());
    let data = res.data.into_json().unwrap();
    assert_eq!(data["children"].as_array().unwrap().len(), 1);
    assert_eq!(data["children"][0]["title"], "Child");
}
