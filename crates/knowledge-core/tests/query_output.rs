use camino::Utf8PathBuf;
use knowledge_core::model::EntityKind;
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::{EntityInput, KnowledgeStore};
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn exact_query_loads_only_the_primary_note_summary() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);
    let temp = tempdir().unwrap();
    let notes = NoteStore::new(Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap());

    let library_id = store
        .upsert_entity(EntityInput::new(
            "MyCompanyName.Ebay.Custom.Client",
            EntityKind::Library,
        ))
        .unwrap();
    let note_path = notes
        .write_note(
            "library",
            "mycompany-ebay-custom-client.md",
            "# Client\n\nUsed to call Ebay custom endpoints.",
        )
        .unwrap();
    let note_path = notes.relative_path(&note_path).unwrap();
    store.attach_note(library_id, note_path).unwrap();

    let answer = store
        .query_exact("MyCompanyName.Ebay.Custom.Client", &notes)
        .unwrap()
        .unwrap();

    assert_eq!(answer.summary, "Used to call Ebay custom endpoints.");
    assert!(answer.location.is_none());
    assert!(answer.navigation_hints.is_empty());
}

#[test]
fn note_store_rejects_paths_that_escape_the_root() {
    let temp = tempdir().unwrap();
    let notes = NoteStore::new(Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap());

    assert!(notes.write_note("..", "escape.md", "outside").is_err());
    assert!(notes.write_note("/tmp", "escape.md", "outside").is_err());
    assert!(notes
        .write_note("library", "../escape.md", "outside")
        .is_err());
    assert!(notes
        .write_note("library", "/tmp/escape.md", "outside")
        .is_err());
    assert!(notes.read_note("../escape.md").is_err());
    assert!(notes.read_note("library/../escape.md").is_err());
    assert!(notes.read_note("/tmp/escape.md").is_err());
    assert!(notes.read_note("library//escape.md").is_err());
}

#[test]
fn relative_path_rejects_paths_outside_the_note_root() {
    let temp = tempdir().unwrap();
    let notes = NoteStore::new(Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap());
    let outside = Utf8PathBuf::from("/tmp/outside-note.md");

    assert!(notes.relative_path(&outside).is_err());
}

#[test]
fn attach_note_rejects_paths_that_escape_the_root() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);
    let library_id = store
        .upsert_entity(EntityInput::new("Example.Client", EntityKind::Library))
        .unwrap();

    assert!(store.attach_note(library_id, "../escape.md").is_err());
    assert!(store
        .attach_note(library_id, "library/../escape.md")
        .is_err());
    assert!(store.attach_note(library_id, "/tmp/escape.md").is_err());
    assert!(store.attach_note(library_id, "library//escape.md").is_err());
}
