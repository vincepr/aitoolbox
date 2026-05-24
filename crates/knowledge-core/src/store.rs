use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

use crate::model::{EntityKind, RelationshipKind};
use crate::notes::{validate_note_relative_path, NoteStore};

/// Input payload for creating or updating an entity row.
#[derive(Debug, Clone)]
pub struct EntityInput {
    /// Stable canonical identifier used for exact lookup.
    pub canonical_name: String,
    /// Entity category.
    pub kind: EntityKind,
    /// Optional short description.
    pub summary: String,
    /// Optional namespace alias.
    pub namespace: Option<String>,
    /// Optional package-name alias.
    pub package_name: Option<String>,
    /// Optional repository-name alias.
    pub repo_name: Option<String>,
}

impl EntityInput {
    /// Creates a minimal entity payload with empty summary and optional fields.
    ///
    /// # Arguments
    ///
    /// * `name` - Canonical entity name.
    /// * `kind` - Entity category.
    ///
    /// # Returns
    ///
    /// `EntityInput` initialized for incremental builder-style updates.
    pub fn new(name: &str, kind: EntityKind) -> Self {
        Self {
            canonical_name: name.to_string(),
            kind,
            summary: String::new(),
            namespace: None,
            package_name: None,
            repo_name: None,
        }
    }

    /// Sets the namespace alias.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace alias value.
    ///
    /// # Returns
    ///
    /// Updated `EntityInput`.
    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }
}

/// Core entity record returned by lookup queries.
#[derive(Debug, Clone)]
pub struct EntityRecord {
    /// SQLite primary key.
    pub id: i64,
    /// Canonical entity identifier.
    pub canonical_name: String,
    /// Stored kind string.
    pub kind: String,
}

/// Exact lookup result containing the matched entity and related neighbors.
#[derive(Debug, Clone)]
pub struct ExactLookup {
    /// Primary matched entity.
    pub entity: EntityRecord,
    /// Recursively-related entities via relationship edges.
    pub related: Vec<EntityRecord>,
}

/// Query result rendered by the CLI `get` command.
#[derive(Debug, Clone)]
pub struct QueryAnswer {
    /// Canonical matched entity identifier.
    pub canonical_name: String,
    /// Extracted note summary.
    pub summary: String,
    /// Reserved navigation hints for future output expansions.
    pub navigation_hints: Vec<String>,
}

/// Extracts the first non-heading paragraph from markdown.
///
/// # Arguments
///
/// * `markdown` - Markdown document text.
///
/// # Returns
///
/// Single-line paragraph summary. Returns an empty string when no paragraph is found.
///
/// # Examples
///
/// ```
/// # use knowledge_core::store::first_paragraph;
/// let md = "# Header\n\nFirst line\nsecond line\n\nMore text";
/// assert_eq!(first_paragraph(md), "First line second line");
/// ```
pub fn first_paragraph(markdown: &str) -> String {
    markdown
        .lines()
        .skip_while(|line| line.trim().is_empty() || line.starts_with('#'))
        .take_while(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// High-level SQLite-backed API for knowledge reads and writes.
pub struct KnowledgeStore<'a> {
    conn: &'a Connection,
}

impl<'a> KnowledgeStore<'a> {
    /// Creates a store using a shared SQLite connection.
    ///
    /// # Arguments
    ///
    /// * `conn` - Open SQLite connection.
    ///
    /// # Returns
    ///
    /// New store instance borrowing `conn`.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Upserts an entity and returns its stable row id.
    ///
    /// # Arguments
    ///
    /// * `input` - Entity attributes to insert or update.
    ///
    /// # Returns
    ///
    /// SQLite row id of the upserted entity.
    ///
    /// # Errors
    ///
    /// Returns an error if SQL writes or lookups fail.
    pub fn upsert_entity(&self, input: EntityInput) -> Result<i64> {
        self.conn.execute(
            "
            INSERT INTO entities (canonical_name, kind, summary, namespace, package_name, repo_name)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(canonical_name) DO UPDATE SET
                kind = excluded.kind,
                summary = excluded.summary,
                namespace = excluded.namespace,
                package_name = excluded.package_name,
                repo_name = excluded.repo_name,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                &input.canonical_name,
                input.kind.as_str(),
                &input.summary,
                input.namespace.as_deref(),
                input.package_name.as_deref(),
                input.repo_name.as_deref(),
            ],
        )?;

        let id = self.conn.query_row(
            "SELECT id FROM entities WHERE canonical_name = ?1",
            [input.canonical_name.as_str()],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    /// Links two entities with a typed relationship.
    ///
    /// # Arguments
    ///
    /// * `from_id` - Source entity id.
    /// * `to_id` - Destination entity id.
    /// * `kind` - Relationship category.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the link exists (new or pre-existing).
    ///
    /// # Errors
    ///
    /// Returns an error if SQL insertion fails.
    pub fn link(&self, from_id: i64, to_id: i64, kind: RelationshipKind) -> Result<()> {
        self.conn.execute(
            "
            INSERT OR IGNORE INTO relationships (from_entity_id, to_entity_id, kind)
            VALUES (?1, ?2, ?3)
            ",
            params![from_id, to_id, kind.as_str()],
        )?;
        Ok(())
    }

    /// Attaches or replaces the note reference for an entity.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity row id.
    /// * `note_path` - Validated relative note path.
    ///
    /// # Returns
    ///
    /// `Ok(())` when the reference is persisted.
    ///
    /// # Errors
    ///
    /// Returns an error if path validation fails or SQL write fails.
    pub fn attach_note(&self, entity_id: i64, note_path: &str) -> Result<()> {
        validate_note_relative_path(note_path)?;

        self.conn.execute(
            "
            INSERT INTO note_refs (entity_id, note_path)
            VALUES (?1, ?2)
            ON CONFLICT(entity_id) DO UPDATE SET
                note_path = excluded.note_path
            ",
            params![entity_id, note_path],
        )?;
        Ok(())
    }

    /// Resolves one entity via exact identifier/alias matches and loads neighbors.
    ///
    /// # Arguments
    ///
    /// * `query` - Canonical name, namespace, package name, repo name, or alias.
    ///
    /// # Returns
    ///
    /// `Some(ExactLookup)` when a match exists, otherwise `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if SQL queries fail.
    pub fn lookup_exact(&self, query: &str) -> Result<Option<ExactLookup>> {
        let entity = self.find_primary_entity(query)?;

        let Some(entity) = entity else {
            return Ok(None);
        };

        let related = self.load_related_entities(entity.id)?;

        Ok(Some(ExactLookup { entity, related }))
    }

    /// Resolves an entity and reads summary text from its attached note.
    ///
    /// # Arguments
    ///
    /// * `query` - Exact lookup query string.
    /// * `notes` - Note store used to read note content.
    ///
    /// # Returns
    ///
    /// `Some(QueryAnswer)` when an entity exists, otherwise `None`.
    ///
    /// # Errors
    ///
    /// Returns an error if SQL operations fail, note references are invalid,
    /// or note files cannot be read.
    pub fn query_exact(&self, query: &str, notes: &NoteStore) -> Result<Option<QueryAnswer>> {
        let lookup = match self.lookup_exact(query)? {
            Some(lookup) => lookup,
            None => return Ok(None),
        };

        let note_path = self
            .conn
            .query_row(
                "SELECT note_path FROM note_refs WHERE entity_id = ?1",
                [lookup.entity.id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        let summary = match note_path {
            Some(path) => first_paragraph(&notes.read_note(&path)?),
            None => String::new(),
        };

        Ok(Some(QueryAnswer {
            canonical_name: lookup.entity.canonical_name,
            summary,
            navigation_hints: Vec::new(),
        }))
    }

    fn find_primary_entity(&self, query: &str) -> Result<Option<EntityRecord>> {
        let entity = self
            .conn
            .query_row(
                "
                SELECT id, canonical_name, kind
                FROM entities e
                WHERE e.canonical_name = ?1
                    OR e.namespace = ?1
                    OR e.package_name = ?1
                    OR e.repo_name = ?1
                    OR EXISTS (
                        SELECT 1
                        FROM aliases a
                        WHERE a.entity_id = e.id AND a.alias = ?1
                    )
                ORDER BY
                    CASE
                        WHEN e.canonical_name = ?1 THEN 1
                        WHEN e.namespace = ?1 THEN 2
                        WHEN e.package_name = ?1 THEN 3
                        WHEN e.repo_name = ?1 THEN 4
                        ELSE 5
                    END,
                    e.canonical_name,
                    e.id
                LIMIT 1
                ",
                [query],
                read_entity_record,
            )
            .optional()?;
        Ok(entity)
    }

    fn load_related_entities(&self, entity_id: i64) -> Result<Vec<EntityRecord>> {
        let mut stmt = self.conn.prepare(
            "
            WITH RECURSIVE related(id, canonical_name, kind) AS (
                SELECT id, canonical_name, kind
                FROM entities
                WHERE id = ?1

                UNION

                SELECT e.id, e.canonical_name, e.kind
                FROM relationships r
                JOIN related current
                    ON r.from_entity_id = current.id OR r.to_entity_id = current.id
                JOIN entities e
                    ON e.id = CASE
                        WHEN r.from_entity_id = current.id THEN r.to_entity_id
                        ELSE r.from_entity_id
                    END
            )
            SELECT id, canonical_name, kind
            FROM related
            WHERE id != ?1
            ORDER BY canonical_name
            ",
        )?;

        let related = stmt
            .query_map([entity_id], read_entity_record)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(related)
    }
}

fn read_entity_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<EntityRecord> {
    Ok(EntityRecord {
        id: row.get(0)?,
        canonical_name: row.get(1)?,
        kind: row.get(2)?,
    })
}
