use anyhow::Result;
use rusqlite::Connection;

pub fn bootstrap(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS entities (
            id INTEGER PRIMARY KEY,
            canonical_name TEXT NOT NULL UNIQUE,
            kind TEXT NOT NULL,
            summary TEXT NOT NULL DEFAULT '',
            namespace TEXT,
            package_name TEXT,
            repo_name TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS aliases (
            entity_id INTEGER NOT NULL,
            alias TEXT NOT NULL,
            UNIQUE(entity_id, alias),
            FOREIGN KEY(entity_id) REFERENCES entities(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS relationships (
            from_entity_id INTEGER NOT NULL,
            to_entity_id INTEGER NOT NULL,
            kind TEXT NOT NULL,
            UNIQUE(from_entity_id, to_entity_id, kind),
            FOREIGN KEY(from_entity_id) REFERENCES entities(id) ON DELETE CASCADE,
            FOREIGN KEY(to_entity_id) REFERENCES entities(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS locations (
            entity_id INTEGER NOT NULL,
            local_path TEXT,
            git_url TEXT,
            UNIQUE(entity_id),
            FOREIGN KEY(entity_id) REFERENCES entities(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS note_refs (
            entity_id INTEGER NOT NULL,
            note_path TEXT NOT NULL,
            UNIQUE(entity_id),
            FOREIGN KEY(entity_id) REFERENCES entities(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_entities_canonical_name ON entities(canonical_name);
        CREATE INDEX IF NOT EXISTS idx_entities_namespace ON entities(namespace);
        CREATE INDEX IF NOT EXISTS idx_entities_package_name ON entities(package_name);
        CREATE INDEX IF NOT EXISTS idx_entities_repo_name ON entities(repo_name);
        CREATE INDEX IF NOT EXISTS idx_aliases_alias ON aliases(alias);
        ",
    )?;

    Ok(())
}
