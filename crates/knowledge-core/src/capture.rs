use anyhow::Result;
use rusqlite::Connection;

use crate::model::EntityKind;
use crate::notes::NoteStore;
use crate::store::{EntityInput, KnowledgeStore};

pub fn capture_lesson(
    conn: &Connection,
    notes: &NoteStore,
    slug: &str,
    body: &str,
) -> Result<String> {
    capture(conn, notes, slug, body, EntityKind::Lesson, "lesson")
}

pub fn capture_issue(
    conn: &Connection,
    notes: &NoteStore,
    slug: &str,
    body: &str,
) -> Result<String> {
    capture(conn, notes, slug, body, EntityKind::Issue, "issue")
}

fn capture(
    conn: &Connection,
    notes: &NoteStore,
    slug: &str,
    body: &str,
    kind: EntityKind,
    folder: &str,
) -> Result<String> {
    let store = KnowledgeStore::new(conn);
    let canonical_name = slug.to_string();
    let id = store.upsert_entity(EntityInput::new(&canonical_name, kind))?;
    let note = format!("# {slug}\n\n{body}\n");
    let path = notes.write_note(folder, &format!("{slug}.md"), &note)?;
    store.attach_note(id, notes.relative_path(&path)?)?;
    Ok(canonical_name)
}
