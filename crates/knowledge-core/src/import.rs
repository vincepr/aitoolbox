use anyhow::{Context, Result};
use camino::Utf8Path;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::hash::{Hash, Hasher};

use crate::audit::{has_idempotency_key, record_mutation_event, MutationEvent};
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
pub fn apply_source_json(conn: &Connection, json: &str, source_label: &str) -> Result<()> {
    let idempotency_key = format!("import:{}:{}", source_label, stable_hash(json));
    if has_idempotency_key(conn, &idempotency_key)? {
        return Ok(());
    }

    let source: SourceFile = serde_json::from_str(json)
        .with_context(|| format!("failed to parse source file: {source_label}"))?;
    let entities = source
        .entities
        .into_iter()
        .map(ValidatedSourceEntity::try_from)
        .collect::<Result<Vec<_>>>()?;

    let tx = conn.unchecked_transaction()?;
    apply_validated_source(&tx, entities)?;
    record_mutation_event(
        &tx,
        &MutationEvent {
            event_id: format!("import:{source_label}:{}", stable_hash(json)),
            operation: "import_source_json".to_string(),
            actor: "knowledge-cli".to_string(),
            target_entity_id: None,
            idempotency_key: Some(idempotency_key),
            input_hash: stable_hash(json),
        },
    )?;
    tx.commit()?;

    Ok(())
}

/// Reads a source JSON file from disk and applies it to the connected database.
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
        upsert_entity_row(conn, &entity)?;
        let id = fetch_entity_id(conn, &entity.canonical_name)?;
        upsert_location_row(conn, id, &entity)?;
    }

    Ok(())
}

fn upsert_entity_row(conn: &Connection, entity: &ValidatedSourceEntity) -> Result<()> {
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
    Ok(())
}

fn fetch_entity_id(conn: &Connection, canonical_name: &str) -> Result<i64> {
    let id = conn.query_row(
        "SELECT id FROM entities WHERE canonical_name = ?1",
        [canonical_name],
        |row| row.get::<_, i64>(0),
    )?;
    Ok(id)
}

fn upsert_location_row(
    conn: &Connection,
    entity_id: i64,
    entity: &ValidatedSourceEntity,
) -> Result<()> {
    conn.execute(
        "
        INSERT INTO locations (entity_id, local_path, git_url)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(entity_id) DO UPDATE SET
            local_path = COALESCE(excluded.local_path, locations.local_path),
            git_url = COALESCE(excluded.git_url, locations.git_url)
        ",
        params![entity_id, entity.local_path, entity.git_url],
    )?;
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

fn stable_hash(input: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
