use knowledge_core::schema::bootstrap;
use rusqlite::Connection;

#[test]
fn bootstrap_creates_core_tables() {
    let conn = Connection::open_in_memory().unwrap();

    bootstrap(&conn).unwrap();

    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .unwrap();
    let names = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert!(names.contains(&"entities".to_string()));
    assert!(names.contains(&"aliases".to_string()));
    assert!(names.contains(&"relationships".to_string()));
    assert!(names.contains(&"locations".to_string()));
    assert!(names.contains(&"note_refs".to_string()));
}
