use camino::Utf8PathBuf;
use knowledge_core::capture::{capture_issue, capture_lesson};
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn captured_lesson_is_queryable_by_exact_name() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let notes_root = tempdir().unwrap();
    let notes =
        NoteStore::new(Utf8PathBuf::from_path_buf(notes_root.path().to_path_buf()).unwrap());

    let lesson_name = capture_lesson(
        &conn,
        &notes,
        "prefer-curated-mappings-over-guesses",
        "Never invent a repo mapping when the configured source is missing.",
    )
    .unwrap();

    let store = KnowledgeStore::new(&conn);
    let answer = store.query_exact(&lesson_name, &notes).unwrap().unwrap();

    assert_eq!(
        answer.summary,
        "Never invent a repo mapping when the configured source is missing."
    );
}

#[test]
fn captured_issue_is_queryable_by_exact_name() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let notes_root = tempdir().unwrap();
    let notes =
        NoteStore::new(Utf8PathBuf::from_path_buf(notes_root.path().to_path_buf()).unwrap());

    let issue_name = capture_issue(
        &conn,
        &notes,
        "missing-source-mapping",
        "A requested repo has no configured source mapping.",
    )
    .unwrap();

    let store = KnowledgeStore::new(&conn);
    let answer = store.query_exact(&issue_name, &notes).unwrap().unwrap();

    assert_eq!(
        answer.summary,
        "A requested repo has no configured source mapping."
    );
}

#[test]
fn invalid_slug_returns_error_without_entity_row() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let notes_root = tempdir().unwrap();
    let notes =
        NoteStore::new(Utf8PathBuf::from_path_buf(notes_root.path().to_path_buf()).unwrap());

    let result = capture_lesson(&conn, &notes, "../bad", "This should not be captured.");

    assert!(result.is_err());
    assert_eq!(entity_count(&conn), 0);
}

#[test]
fn duplicate_lesson_slug_returns_error_and_preserves_original_summary() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let notes_root = tempdir().unwrap();
    let notes =
        NoteStore::new(Utf8PathBuf::from_path_buf(notes_root.path().to_path_buf()).unwrap());

    capture_lesson(&conn, &notes, "repeat-slug", "Original lesson.").unwrap();

    let result = capture_lesson(&conn, &notes, "repeat-slug", "Replacement lesson.");

    let store = KnowledgeStore::new(&conn);
    let answer = store.query_exact("repeat-slug", &notes).unwrap().unwrap();

    assert!(result.is_err());
    assert_eq!(answer.summary, "Original lesson.");
    assert_eq!(entity_count(&conn), 1);
}

#[test]
fn cross_kind_collision_returns_error_and_preserves_original_summary() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let notes_root = tempdir().unwrap();
    let notes =
        NoteStore::new(Utf8PathBuf::from_path_buf(notes_root.path().to_path_buf()).unwrap());

    capture_issue(&conn, &notes, "same-slug", "Original issue.").unwrap();

    let result = capture_lesson(&conn, &notes, "same-slug", "Replacement lesson.");

    let store = KnowledgeStore::new(&conn);
    let answer = store.query_exact("same-slug", &notes).unwrap().unwrap();

    assert!(result.is_err());
    assert_eq!(answer.summary, "Original issue.");
    assert_eq!(entity_count(&conn), 1);
}

fn entity_count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .unwrap()
}
