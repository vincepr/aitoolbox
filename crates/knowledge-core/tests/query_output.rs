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
    notes
        .write_note(
            "library",
            "mycompany-ebay-custom-client.md",
            "# Client\n\nUsed to call Ebay custom endpoints.",
        )
        .unwrap();
    store
        .attach_note(library_id, "library/mycompany-ebay-custom-client.md")
        .unwrap();

    let answer = store
        .query_exact("MyCompanyName.Ebay.Custom.Client", &notes)
        .unwrap()
        .unwrap();

    assert_eq!(answer.summary, "Used to call Ebay custom endpoints.");
    assert!(answer.navigation_hints.is_empty());
}
