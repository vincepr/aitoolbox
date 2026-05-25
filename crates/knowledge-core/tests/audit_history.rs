use camino::Utf8PathBuf;
use knowledge_core::audit::list_entity_history;
use knowledge_core::capture::capture_lesson;
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn capture_lesson_writes_mutation_event() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let notes_root = tempdir().unwrap();
    let notes =
        NoteStore::new(Utf8PathBuf::from_path_buf(notes_root.path().to_path_buf()).unwrap());

    capture_lesson(&conn, &notes, "audit-slug", "audit body").unwrap();

    let store = KnowledgeStore::new(&conn);
    let entity_id = store.find_entity_id_by_name("audit-slug").unwrap().unwrap();
    let history = list_entity_history(&conn, entity_id, 10).unwrap();

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].operation, "capture_lesson");
}
