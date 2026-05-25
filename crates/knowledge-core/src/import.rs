use anyhow::{Context, Result};
use camino::Utf8Path;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::hash::{Hash, Hasher};

use crate::audit::{has_idempotency_key, record_mutation_event, MutationEvent};
use crate::input_schema::{validate_payload, InputSchemaKind};
use crate::model::EntityKind;

/// Top-level JSON source document used for batch import.
#[derive(Debug, Deserialize)]
pub struct SourceFile {
    /// Input schema URI for versioned validation.
    #[serde(rename = "$schema")]
    pub schema: String,
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
    /// Optional short summary for exact lookup responses.
    pub summary: Option<String>,
    /// Optional aliases; `null` means unknown and `[]` means known empty.
    pub aliases: Option<Vec<String>>,
    /// Optional notes; `null` means unknown and `[]` means known empty.
    pub notes: Option<Vec<String>>,
    /// Optional nested location object.
    pub location: Option<SourceLocation>,
    /// Optional local filesystem location.
    #[serde(default)]
    pub local_path: Option<String>,
    /// Optional remote Git URL location.
    #[serde(default)]
    pub git_url: Option<String>,
}

/// Optional source location fields.
#[derive(Debug, Deserialize)]
pub struct SourceLocation {
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

    validate_payload(json, InputSchemaKind::Entity)
        .with_context(|| format!("source file failed schema validation: {source_label}"))?;
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
    summary: String,
    aliases: Option<Vec<String>>,
    notes: Option<Vec<String>>,
    local_path: Option<String>,
    git_url: Option<String>,
}

impl TryFrom<SourceEntity> for ValidatedSourceEntity {
    type Error = anyhow::Error;

    fn try_from(entity: SourceEntity) -> Result<Self> {
        Ok(Self {
            canonical_name: entity.canonical_name,
            kind: parse_entity_kind(&entity.kind)?,
            summary: entity.summary.unwrap_or_default(),
            repo_name: entity.repo_name,
            namespace: entity.namespace,
            package_name: entity.package_name,
            aliases: entity.aliases,
            notes: entity.notes,
            local_path: entity
                .location
                .as_ref()
                .and_then(|location| location.local_path.clone())
                .or(entity.local_path),
            git_url: entity
                .location
                .as_ref()
                .and_then(|location| location.git_url.clone())
                .or(entity.git_url),
        })
    }
}

fn apply_validated_source(conn: &Connection, entities: Vec<ValidatedSourceEntity>) -> Result<()> {
    for entity in entities {
        upsert_entity_row(conn, &entity)?;
        let id = fetch_entity_id(conn, &entity.canonical_name)?;
        upsert_location_row(conn, id, &entity)?;
        let aliases_known = entity.aliases.is_some();
        let notes_known = entity.notes.is_some();
        replace_aliases(conn, id, entity.aliases)?;
        upsert_collection_states(conn, id, notes_known, aliases_known)?;
    }

    Ok(())
}

fn upsert_entity_row(conn: &Connection, entity: &ValidatedSourceEntity) -> Result<()> {
    conn.execute(
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
            entity.canonical_name,
            entity.kind.as_str(),
            entity.summary,
            entity.namespace,
            entity.package_name,
            entity.repo_name,
        ],
    )?;
    Ok(())
}

fn replace_aliases(conn: &Connection, entity_id: i64, aliases: Option<Vec<String>>) -> Result<()> {
    let Some(aliases) = aliases else {
        return Ok(());
    };
    conn.execute("DELETE FROM aliases WHERE entity_id = ?1", [entity_id])?;
    for alias in aliases {
        conn.execute(
            "INSERT OR IGNORE INTO aliases (entity_id, alias) VALUES (?1, ?2)",
            params![entity_id, alias],
        )?;
    }
    Ok(())
}

fn upsert_collection_states(
    conn: &Connection,
    entity_id: i64,
    notes_known: bool,
    aliases_known: bool,
) -> Result<()> {
    let notes_state = if notes_known { "known" } else { "unknown" };
    let aliases_state = if aliases_known { "known" } else { "unknown" };
    conn.execute(
        "
        UPDATE entities
        SET notes_state = ?1,
            aliases_state = ?2,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?3
        ",
        params![notes_state, aliases_state, entity_id],
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
