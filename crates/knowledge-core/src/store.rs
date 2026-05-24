use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

use crate::model::{EntityKind, RelationshipKind};
use crate::notes::{validate_note_relative_path, NoteStore};

#[derive(Debug, Clone)]
pub struct EntityInput {
    pub canonical_name: String,
    pub kind: EntityKind,
    pub summary: String,
    pub namespace: Option<String>,
    pub package_name: Option<String>,
    pub repo_name: Option<String>,
}

impl EntityInput {
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

    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct EntityRecord {
    pub id: i64,
    pub canonical_name: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct ExactLookup {
    pub entity: EntityRecord,
    pub related: Vec<EntityRecord>,
}

#[derive(Debug, Clone)]
pub struct QueryAnswer {
    pub canonical_name: String,
    pub summary: String,
    pub navigation_hints: Vec<String>,
}

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

pub struct KnowledgeStore<'a> {
    conn: &'a Connection,
}

impl<'a> KnowledgeStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

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

    pub fn lookup_exact(&self, query: &str) -> Result<Option<ExactLookup>> {
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

        let Some(entity) = entity else {
            return Ok(None);
        };

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
            .query_map([entity.id], read_entity_record)?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(Some(ExactLookup { entity, related }))
    }

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
}

fn read_entity_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<EntityRecord> {
    Ok(EntityRecord {
        id: row.get(0)?,
        canonical_name: row.get(1)?,
        kind: row.get(2)?,
    })
}
