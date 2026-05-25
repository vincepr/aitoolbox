use anyhow::Result;
use rusqlite::Connection;

/// Ordered migration definitions for the knowledge schema.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// Monotonic migration version.
    pub version: i64,
    /// Human-readable migration name.
    pub name: &'static str,
    /// SQL body executed inside a transaction.
    pub sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "baseline",
        sql: r#"
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

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

        CREATE TABLE IF NOT EXISTS mutation_events (
            id INTEGER PRIMARY KEY,
            event_id TEXT NOT NULL UNIQUE,
            operation TEXT NOT NULL,
            actor TEXT NOT NULL,
            target_entity_id INTEGER,
            idempotency_key TEXT,
            input_hash TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(target_entity_id) REFERENCES entities(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS source_evidence (
            id INTEGER PRIMARY KEY,
            mutation_event_id INTEGER NOT NULL,
            source_label TEXT NOT NULL,
            source_hash TEXT NOT NULL,
            FOREIGN KEY(mutation_event_id) REFERENCES mutation_events(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_entities_canonical_name ON entities(canonical_name);
        CREATE INDEX IF NOT EXISTS idx_entities_namespace ON entities(namespace);
        CREATE INDEX IF NOT EXISTS idx_entities_package_name ON entities(package_name);
        CREATE INDEX IF NOT EXISTS idx_entities_repo_name ON entities(repo_name);
        CREATE INDEX IF NOT EXISTS idx_aliases_alias ON aliases(alias);
        CREATE INDEX IF NOT EXISTS idx_mutation_events_entity_id ON mutation_events(target_entity_id);
        CREATE INDEX IF NOT EXISTS idx_mutation_events_created_at ON mutation_events(created_at);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_mutation_events_idempotency_key ON mutation_events(idempotency_key) WHERE idempotency_key IS NOT NULL;
    "#,
    },
    Migration {
        version: 2,
        name: "retrieval_telemetry",
        sql: r#"
        CREATE TABLE IF NOT EXISTS retrieval_telemetry (
            id INTEGER PRIMARY KEY,
            query TEXT NOT NULL,
            match_source TEXT NOT NULL,
            total_score INTEGER NOT NULL,
            exact_score INTEGER NOT NULL,
            alias_score INTEGER NOT NULL,
            fts_score INTEGER NOT NULL,
            graph_score INTEGER NOT NULL,
            selected_entity TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_retrieval_telemetry_created_at ON retrieval_telemetry(created_at);
    "#,
    },
];

/// Returns the latest supported schema version.
pub fn latest_migration_version() -> i64 {
    MIGRATIONS.last().map_or(0, |m| m.version)
}

fn ensure_ledger_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        ",
    )?;
    Ok(())
}

/// Applies all pending migrations to the connected SQLite database.
pub fn apply_all(conn: &Connection) -> Result<()> {
    ensure_ledger_table(conn)?;

    for migration in MIGRATIONS {
        let already_applied = conn.query_row(
            "SELECT 1 FROM schema_migrations WHERE version = ?1 LIMIT 1",
            [migration.version],
            |_| Ok(()),
        );

        if already_applied.is_ok() {
            continue;
        }

        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name) VALUES (?1, ?2)",
            (migration.version, migration.name),
        )?;
        tx.commit()?;
    }

    Ok(())
}

/// Returns the current migrated schema version for this database.
pub fn current_schema_version(conn: &Connection) -> Result<i64> {
    ensure_ledger_table(conn)?;

    let version = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get::<_, i64>(0),
    )?;
    Ok(version)
}
