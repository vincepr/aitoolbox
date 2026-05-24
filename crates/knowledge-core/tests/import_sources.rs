use camino::Utf8PathBuf;
#[allow(unused_imports)]
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
    fs::write(
        &file,
        r#"{
          "entities": [
            {
              "canonical_name": "ebay-common",
              "kind": "project",
              "repo_name": "Common",
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
    apply_source_file(&conn, Utf8PathBuf::from_path_buf(file).unwrap().as_path()).unwrap();

    let store = KnowledgeStore::new(&conn);
    let result = store.lookup_exact("ebay-common").unwrap().unwrap();

    assert_eq!(result.entity.canonical_name, "ebay-common");
}
