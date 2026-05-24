use anyhow::{bail, Result};
use rusqlite::{Connection, OptionalExtension};

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
    ensure_can_capture(conn, slug)?;

    let note = format!("# {slug}\n\n{body}\n");
    let path = notes.write_note(folder, &format!("{slug}.md"), &note)?;
    let id = store.upsert_entity(EntityInput::new(&canonical_name, kind))?;
    store.attach_note(id, notes.relative_path(&path)?)?;
    Ok(canonical_name)
}

fn ensure_can_capture(conn: &Connection, slug: &str) -> Result<()> {
    if slug.trim().is_empty() {
        bail!("capture slug must not be empty");
    }

    if slug.contains('/') || slug.contains('\\') || slug.contains('.') {
        bail!("capture slug must be a single safe path component");
    }

    let existing = conn
        .query_row(
            "SELECT 1 FROM entities WHERE canonical_name = ?1 LIMIT 1",
            [slug],
            |_| Ok(()),
        )
        .optional()?;
    if existing.is_some() {
        bail!("capture slug already exists");
    }

    Ok(())
}
