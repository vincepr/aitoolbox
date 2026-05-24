use anyhow::Result;
use camino::Utf8Path;
use rusqlite::{params, Connection};
use serde::Deserialize;

use crate::model::EntityKind;
use crate::store::{EntityInput, KnowledgeStore};

#[derive(Debug, Deserialize)]
pub struct SourceFile {
    pub entities: Vec<SourceEntity>,
}

#[derive(Debug, Deserialize)]
pub struct SourceEntity {
    pub canonical_name: String,
    pub kind: String,
    pub repo_name: Option<String>,
    pub namespace: Option<String>,
    pub package_name: Option<String>,
    pub local_path: Option<String>,
    pub git_url: Option<String>,
}

pub fn apply_source_file(conn: &Connection, path: &Utf8Path) -> Result<()> {
    let source: SourceFile = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let store = KnowledgeStore::new(conn);

    for entity in source.entities {
        let kind = match entity.kind.as_str() {
            "domain" => EntityKind::Domain,
            "system" => EntityKind::System,
            "project" => EntityKind::Project,
            "library" => EntityKind::Library,
            "tag" => EntityKind::Tag,
            "lesson" => EntityKind::Lesson,
            "issue" => EntityKind::Issue,
            other => anyhow::bail!("unsupported entity kind: {other}"),
        };

        let id = store.upsert_entity(EntityInput {
            canonical_name: entity.canonical_name,
            kind,
            summary: String::new(),
            namespace: entity.namespace,
            package_name: entity.package_name,
            repo_name: entity.repo_name,
        })?;

        conn.execute(
            "
            INSERT INTO locations (entity_id, local_path, git_url)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(entity_id) DO UPDATE SET
                local_path = excluded.local_path,
                git_url = excluded.git_url
            ",
            params![id, entity.local_path, entity.git_url],
        )?;
    }

    Ok(())
}
