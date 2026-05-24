use anyhow::{Context, Result};
use camino::Utf8Path;
use rusqlite::{params, Connection};
use serde::Deserialize;

use crate::model::EntityKind;

/// Top-level JSON source document used for batch import.
#[derive(Debug, Deserialize)]
pub struct SourceFile {
    /// Imported entities to upsert into the store.
    pub entities: Vec<SourceEntity>,
}

/// A single source entity entry from import JSON.
#[derive(Debug, Deserialize)]
pub struct SourceEntity {
    /// Stable canonical identifier for exact lookup.
    pub canonical_name: String,
    /// Entity kind string, e.g. `library` or `project`.
    pub kind: String,
    /// Optional repository name alias for lookup.
    pub repo_name: Option<String>,
    /// Optional namespace alias for lookup.
    pub namespace: Option<String>,
    /// Optional package-name alias for lookup.
    pub package_name: Option<String>,
    /// Optional local filesystem location.
    pub local_path: Option<String>,
    /// Optional remote Git URL location.
    pub git_url: Option<String>,
}

/// Applies a source JSON payload to the connected SQLite database.
///
/// # Arguments
///
/// * `conn` - Open SQLite connection.
/// * `json` - Source JSON string with a top-level `entities` array.
/// * `source_label` - Human-readable label included in parse errors.
///
/// # Returns
///
/// `Ok(())` after all entities and locations are upserted in one transaction.
///
/// # Errors
///
/// Returns an error if JSON parsing fails, if an entity kind is unsupported,
/// or if any SQL operation fails.
///
/// # Examples
///
/// ```
/// # use anyhow::Result;
/// # use knowledge_core::import::apply_source_json;
/// # use knowledge_core::schema::bootstrap;
/// # use rusqlite::Connection;
/// # fn demo() -> Result<()> {
/// let conn = Connection::open_in_memory()?;
/// bootstrap(&conn)?;
/// apply_source_json(
///     &conn,
///     r#"{"entities":[{"canonical_name":"example.lib","kind":"library"}]}"#,
///     "--source-json",
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn apply_source_json(conn: &Connection, json: &str, source_label: &str) -> Result<()> {
    let source: SourceFile = serde_json::from_str(json)
        .with_context(|| format!("failed to parse source file: {source_label}"))?;
    let entities = source
        .entities
        .into_iter()
        .map(ValidatedSourceEntity::try_from)
        .collect::<Result<Vec<_>>>()?;

    let tx = conn.unchecked_transaction()?;
    apply_validated_source(&tx, entities)?;
    tx.commit()?;

    Ok(())
}

/// Reads a source JSON file from disk and applies it to the connected database.
///
/// # Arguments
///
/// * `conn` - Open SQLite connection.
/// * `path` - UTF-8 path to the source JSON file.
///
/// # Returns
///
/// `Ok(())` after the file is parsed and applied.
///
/// # Errors
///
/// Returns an error if the file cannot be read, the JSON is invalid, entity
/// validation fails, or database writes fail.
pub fn apply_source_file(conn: &Connection, path: &Utf8Path) -> Result<()> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read source file: {path}"))?;
    apply_source_json(conn, &json, path.as_str())
}

struct ValidatedSourceEntity {
    canonical_name: String,
    kind: EntityKind,
    repo_name: Option<String>,
    namespace: Option<String>,
    package_name: Option<String>,
    local_path: Option<String>,
    git_url: Option<String>,
}

impl TryFrom<SourceEntity> for ValidatedSourceEntity {
    type Error = anyhow::Error;

    fn try_from(entity: SourceEntity) -> Result<Self> {
        Ok(Self {
            canonical_name: entity.canonical_name,
            kind: parse_entity_kind(&entity.kind)?,
            repo_name: entity.repo_name,
            namespace: entity.namespace,
            package_name: entity.package_name,
            local_path: entity.local_path,
            git_url: entity.git_url,
        })
    }
}

fn apply_validated_source(conn: &Connection, entities: Vec<ValidatedSourceEntity>) -> Result<()> {
    for entity in entities {
        conn.execute(
            "
            INSERT INTO entities (canonical_name, kind, summary, namespace, package_name, repo_name)
            VALUES (?1, ?2, '', ?3, ?4, ?5)
            ON CONFLICT(canonical_name) DO UPDATE SET
                kind = excluded.kind,
                namespace = COALESCE(excluded.namespace, entities.namespace),
                package_name = COALESCE(excluded.package_name, entities.package_name),
                repo_name = COALESCE(excluded.repo_name, entities.repo_name),
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                entity.canonical_name,
                entity.kind.as_str(),
                entity.namespace,
                entity.package_name,
                entity.repo_name,
            ],
        )?;

        let id = conn.query_row(
            "SELECT id FROM entities WHERE canonical_name = ?1",
            [entity.canonical_name.as_str()],
            |row| row.get::<_, i64>(0),
        )?;

        conn.execute(
            "
            INSERT INTO locations (entity_id, local_path, git_url)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(entity_id) DO UPDATE SET
                local_path = COALESCE(excluded.local_path, locations.local_path),
                git_url = COALESCE(excluded.git_url, locations.git_url)
            ",
            params![id, entity.local_path, entity.git_url],
        )?;
    }

    Ok(())
}

fn parse_entity_kind(kind: &str) -> Result<EntityKind> {
    match kind {
        "domain" => Ok(EntityKind::Domain),
        "system" => Ok(EntityKind::System),
        "project" => Ok(EntityKind::Project),
        "library" => Ok(EntityKind::Library),
        "tag" => Ok(EntityKind::Tag),
        "lesson" => Ok(EntityKind::Lesson),
        "issue" => Ok(EntityKind::Issue),
        other => anyhow::bail!("unsupported entity kind: {other}"),
    }
}
