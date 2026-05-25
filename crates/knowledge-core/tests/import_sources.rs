use camino::Utf8PathBuf;
use knowledge_core::import::{apply_source_file, SourceFile};
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;

#[test]
fn source_file_import_is_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let temp = tempdir().unwrap();
    let file = temp.path().join("sources.json");
    let source = r#"{
          "$schema": "https://aitoolbox/schemas/entity.v1.json",
          "entities": [
            {
              "canonical_name": "ebay-common",
              "kind": "project",
              "summary": null,
              "namespace": null,
              "package_name": null,
              "repo_name": "Common",
              "aliases": [],
              "location": null,
              "notes": null,
              "local_path": "C:/repos/Ebay/Common",
              "git_url": "https://example.invalid/marketplaces/ebay/Common.git"
            }
          ]
        }"#;
    let source_file: SourceFile = serde_json::from_str(source).unwrap();
    assert_eq!(source_file.entities.len(), 1);
    fs::write(&file, source).unwrap();

    apply_source_file(
        &conn,
        Utf8PathBuf::from_path_buf(file.clone()).unwrap().as_path(),
    )
    .unwrap();
    apply_source_file(&conn, Utf8PathBuf::from_path_buf(file).unwrap().as_path()).unwrap();

    let store = KnowledgeStore::new(&conn);
    let result = store.lookup_exact("ebay-common").unwrap().unwrap();

    assert_eq!(result.entity.canonical_name, "ebay-common");

    let entity_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .unwrap();
    let location_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM locations", [], |row| row.get(0))
        .unwrap();
    let (local_path, git_url): (String, String) = conn
        .query_row(
            "
            SELECT l.local_path, l.git_url
            FROM locations l
            JOIN entities e ON e.id = l.entity_id
            WHERE e.canonical_name = 'ebay-common'
            ",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(entity_count, 1);
    assert_eq!(location_count, 1);
    assert_eq!(local_path, "C:/repos/Ebay/Common");
    assert_eq!(
        git_url,
        "https://example.invalid/marketplaces/ebay/Common.git"
    );
}

#[test]
fn source_file_refresh_preserves_omitted_fields() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let temp = tempdir().unwrap();
    let file = temp.path().join("sources.json");
    fs::write(
        &file,
        r#"{
          "$schema": "https://aitoolbox/schemas/entity.v1.json",
          "entities": [
            {
              "canonical_name": "ebay-common",
              "kind": "project",
              "summary": null,
              "repo_name": "Common",
              "namespace": "MyCompanyName.Ebay.Common",
              "package_name": "MyCompanyName.Ebay.Common",
              "aliases": [],
              "location": null,
              "notes": [],
              "local_path": "C:/repos/Ebay/Common",
              "git_url": "https://example.invalid/marketplaces/ebay/Common.git"
            }
          ]
        }"#,
    )
    .unwrap();
    apply_source_file(
        &conn,
        Utf8PathBuf::from_path_buf(file.clone()).unwrap().as_path(),
    )
    .unwrap();

    conn.execute(
        "UPDATE entities SET summary = 'Existing summary' WHERE canonical_name = 'ebay-common'",
        [],
    )
    .unwrap();

    fs::write(
        &file,
        r#"{
          "$schema": "https://aitoolbox/schemas/entity.v1.json",
          "entities": [
            {
              "canonical_name": "ebay-common",
              "kind": "project",
              "summary": null,
              "namespace": null,
              "package_name": null,
              "repo_name": "Common.Refreshed",
              "aliases": null,
              "location": null,
              "notes": null,
              "local_path": "D:/repos/Ebay/Common"
            }
          ]
        }"#,
    )
    .unwrap();
    apply_source_file(&conn, Utf8PathBuf::from_path_buf(file).unwrap().as_path()).unwrap();

    let (summary, namespace, package_name, repo_name, local_path, git_url): (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) = conn
        .query_row(
            "
            SELECT e.summary, e.namespace, e.package_name, e.repo_name, l.local_path, l.git_url
            FROM entities e
            JOIN locations l ON l.entity_id = e.id
            WHERE e.canonical_name = 'ebay-common'
            ",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .unwrap();

    assert_eq!(summary, "");
    assert_eq!(namespace, None);
    assert_eq!(package_name, None);
    assert_eq!(repo_name, Some("Common.Refreshed".to_string()));
    assert_eq!(local_path, Some("D:/repos/Ebay/Common".to_string()));
    assert_eq!(
        git_url,
        Some("https://example.invalid/marketplaces/ebay/Common.git".to_string())
    );
}

#[test]
fn source_file_validation_failure_leaves_no_partial_writes() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let temp = tempdir().unwrap();
    let file = temp.path().join("sources.json");
    fs::write(
        &file,
        r#"{
          "$schema": "https://aitoolbox/schemas/entity.v1.json",
          "entities": [
            {
              "canonical_name": "ebay-common",
              "kind": "project",
              "summary": null,
              "namespace": null,
              "package_name": null,
              "repo_name": null,
              "aliases": null,
              "location": null,
              "notes": null,
              "local_path": "C:/repos/Ebay/Common"
            },
            {
              "canonical_name": "unsupported",
              "kind": "unsupported",
              "summary": null,
              "namespace": null,
              "package_name": null,
              "repo_name": null,
              "aliases": null,
              "location": null,
              "notes": null
            }
          ]
        }"#,
    )
    .unwrap();

    let err = apply_source_file(&conn, Utf8PathBuf::from_path_buf(file).unwrap().as_path())
        .unwrap_err()
        .to_string();

    let entity_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .unwrap();
    let location_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM locations", [], |row| row.get(0))
        .unwrap();

    assert!(err.contains("source file failed schema validation"));
    assert_eq!(entity_count, 0);
    assert_eq!(location_count, 0);
}

#[test]
fn source_file_commit_failure_rolls_back_and_closes_transaction() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    conn.execute_batch(
        "
        CREATE TABLE commit_failure_parent (
            id INTEGER PRIMARY KEY
        );

        CREATE TABLE commit_failure_child (
            parent_id INTEGER NOT NULL,
            FOREIGN KEY(parent_id) REFERENCES commit_failure_parent(id)
                DEFERRABLE INITIALLY DEFERRED
        );

        CREATE TRIGGER force_deferred_commit_failure
        AFTER INSERT ON locations
        BEGIN
            INSERT INTO commit_failure_child(parent_id) VALUES (NEW.entity_id);
        END;
        ",
    )
    .unwrap();

    let temp = tempdir().unwrap();
    let file = temp.path().join("sources.json");
    fs::write(
        &file,
        r#"{
          "$schema": "https://aitoolbox/schemas/entity.v1.json",
          "entities": [
            {
              "canonical_name": "ebay-common",
              "kind": "project",
              "summary": null,
              "namespace": null,
              "package_name": null,
              "repo_name": null,
              "aliases": [],
              "location": null,
              "notes": null,
              "local_path": "C:/repos/Ebay/Common"
            }
          ]
        }"#,
    )
    .unwrap();

    let err = apply_source_file(&conn, Utf8PathBuf::from_path_buf(file).unwrap().as_path())
        .unwrap_err()
        .to_string();

    let entity_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .unwrap();
    let location_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM locations", [], |row| row.get(0))
        .unwrap();

    assert!(err.contains("FOREIGN KEY constraint failed"));
    assert!(conn.is_autocommit());
    assert_eq!(entity_count, 0);
    assert_eq!(location_count, 0);
}
