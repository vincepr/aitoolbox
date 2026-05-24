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
