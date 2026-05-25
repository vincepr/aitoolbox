use anyhow::Result;
use rusqlite::{params, Connection};

/// Immutable mutation event row for auditing write paths.
#[derive(Debug, Clone)]
pub struct MutationEvent {
    /// Stable event identifier.
    pub event_id: String,
    /// Operation label such as `capture_lesson`.
    pub operation: String,
    /// Actor string, usually the command name.
    pub actor: String,
    /// Optional target entity id.
    pub target_entity_id: Option<i64>,
    /// Optional idempotency key for replay-safe mutations.
    pub idempotency_key: Option<String>,
    /// Content hash used to deduplicate replay attempts.
    pub input_hash: String,
}

/// Lightweight history row returned by history queries.
#[derive(Debug, Clone)]
pub struct MutationHistoryRow {
    /// Operation label.
    pub operation: String,
    /// Actor label.
    pub actor: String,
    /// Timestamp in SQLite text format.
    pub created_at: String,
}

/// Records one immutable mutation event row.
pub fn record_mutation_event(conn: &Connection, event: &MutationEvent) -> Result<()> {
    conn.execute(
        "
        INSERT INTO mutation_events (
            event_id,
            operation,
            actor,
            target_entity_id,
            idempotency_key,
            input_hash
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ",
        params![
            event.event_id,
            event.operation,
            event.actor,
            event.target_entity_id,
            event.idempotency_key,
            event.input_hash,
        ],
    )?;

    Ok(())
}

/// Returns true when the idempotency key has already been seen.
pub fn has_idempotency_key(conn: &Connection, key: &str) -> Result<bool> {
    let seen = conn
        .query_row(
            "SELECT 1 FROM mutation_events WHERE idempotency_key = ?1 LIMIT 1",
            [key],
            |_| Ok(()),
        )
        .is_ok();
    Ok(seen)
}

/// Lists recent mutation history rows for one entity.
pub fn list_entity_history(
    conn: &Connection,
    entity_id: i64,
    limit: u32,
) -> Result<Vec<MutationHistoryRow>> {
    let mut stmt = conn.prepare(
        "
        SELECT operation, actor, created_at
        FROM mutation_events
        WHERE target_entity_id = ?1
        ORDER BY id DESC
        LIMIT ?2
        ",
    )?;

    let rows = stmt
        .query_map(params![entity_id, i64::from(limit)], |row| {
            Ok(MutationHistoryRow {
                operation: row.get(0)?,
                actor: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}
