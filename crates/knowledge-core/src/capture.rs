use anyhow::{bail, Result};
use rusqlite::{Connection, OptionalExtension};

use crate::audit::{has_idempotency_key, record_mutation_event, MutationEvent};
use crate::model::EntityKind;
use crate::notes::NoteStore;
use crate::store::{EntityInput, KnowledgeStore};

/// Captures a lesson note and stores it as an entity and note reference.
///
/// # Arguments
///
/// * `conn` - Open SQLite connection.
/// * `notes` - Note store used to write markdown content.
/// * `slug` - Canonical identifier and file stem for the lesson.
/// * `body` - Markdown body content.
///
/// # Returns
///
/// The canonical lesson name (`slug`) on success.
///
/// # Errors
///
/// Returns an error when the slug is invalid, already exists, note writing
/// fails, or database updates fail.
pub fn capture_lesson(
    conn: &Connection,
    notes: &NoteStore,
    slug: &str,
    body: &str,
) -> Result<String> {
    capture(conn, notes, slug, body, EntityKind::Lesson, "lesson")
}

/// Captures an issue note and stores it as an entity and note reference.
///
/// # Arguments
///
/// * `conn` - Open SQLite connection.
/// * `notes` - Note store used to write markdown content.
/// * `slug` - Canonical identifier and file stem for the issue.
/// * `body` - Markdown body content.
///
/// # Returns
///
/// The canonical issue name (`slug`) on success.
///
/// # Errors
///
/// Returns an error when the slug is invalid, already exists, note writing
/// fails, or database updates fail.
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

    let idempotency_key = format!("capture:{folder}:{slug}");
    if has_idempotency_key(conn, &idempotency_key)? {
        return Ok(canonical_name);
    }

    let note = format!("# {slug}\n\n{body}\n");
    let path = notes.write_note(folder, &format!("{slug}.md"), &note)?;
    let id = store.upsert_entity(EntityInput::new(&canonical_name, kind))?;
    store.attach_note(id, notes.relative_path(&path)?)?;

    record_mutation_event(
        conn,
        &MutationEvent {
            event_id: format!("capture:{folder}:{slug}:{id}"),
            operation: format!("capture_{folder}"),
            actor: "knowledge-cli".to_string(),
            target_entity_id: Some(id),
            idempotency_key: Some(idempotency_key),
            input_hash: format!("{slug}:{body}"),
        },
    )?;

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
