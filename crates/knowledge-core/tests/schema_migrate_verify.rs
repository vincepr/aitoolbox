use knowledge_core::schema::{bootstrap, verify_schema};
use rusqlite::Connection;

#[test]
fn verify_fails_when_db_version_is_behind_latest() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute("CREATE TABLE entities (id INTEGER PRIMARY KEY)", [])
        .unwrap();

    let err = verify_schema(&conn).unwrap_err();
    assert!(err.to_string().contains("schema version mismatch"));
}

#[test]
fn verify_passes_after_bootstrap_migrations() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    verify_schema(&conn).unwrap();
}
