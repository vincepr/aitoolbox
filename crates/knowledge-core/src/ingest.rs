use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Pipeline status values persisted in `ingest_jobs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IngestStatus {
    Queued,
    Processing,
    Succeeded,
    Failed,
}

impl IngestStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Processing => "processing",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

/// One row returned by pipeline status listings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestJobRow {
    pub id: i64,
    pub status: String,
    pub attempts: i64,
    pub last_error: Option<String>,
}

/// Result of processing a single queued job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOnceResult {
    pub initial_state: String,
    pub final_state: String,
    pub job_id: i64,
}

/// Enqueues one raw payload using hash-based dedupe.
///
/// Returns the existing or newly created job id.
pub fn enqueue_job(conn: &Connection, payload: &str) -> Result<i64> {
    let hash = payload_hash(payload);

    conn.execute(
        "
        INSERT INTO ingest_jobs (payload, payload_hash, status)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(payload_hash) DO NOTHING
        ",
        params![payload, hash, IngestStatus::Queued.as_str()],
    )?;

    let id = conn.query_row(
        "SELECT id FROM ingest_jobs WHERE payload_hash = ?1",
        [hash],
        |row| row.get::<_, i64>(0),
    )?;
    Ok(id)
}

/// Processes one queued job through parse->normalize->classify->persist.
pub fn run_once(conn: &Connection) -> Result<Option<RunOnceResult>> {
    let queued = conn
        .query_row(
            "
            SELECT id, payload
            FROM ingest_jobs
            WHERE status = 'queued'
            ORDER BY id
            LIMIT 1
            ",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;

    let Some((job_id, payload)) = queued else {
        return Ok(None);
    };

    conn.execute(
        "
        UPDATE ingest_jobs
        SET status = 'processing', attempts = attempts + 1, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        ",
        [job_id],
    )?;

    let result = process_payload(&payload).and_then(|(normalized_text, classification)| {
        persist_result(conn, job_id, &normalized_text, &classification)
    });

    match result {
        Ok(()) => {
            conn.execute(
                "
                UPDATE ingest_jobs
                SET status = 'succeeded', last_error = NULL, updated_at = CURRENT_TIMESTAMP
                WHERE id = ?1
                ",
                [job_id],
            )?;
            Ok(Some(RunOnceResult {
                initial_state: IngestStatus::Queued.as_str().to_string(),
                final_state: IngestStatus::Succeeded.as_str().to_string(),
                job_id,
            }))
        }
        Err(error) => {
            conn.execute(
                "
                UPDATE ingest_jobs
                SET status = 'failed', last_error = ?2, updated_at = CURRENT_TIMESTAMP
                WHERE id = ?1
                ",
                params![job_id, error.to_string()],
            )?;
            Ok(Some(RunOnceResult {
                initial_state: IngestStatus::Queued.as_str().to_string(),
                final_state: IngestStatus::Failed.as_str().to_string(),
                job_id,
            }))
        }
    }
}

/// Moves failed jobs back to queued state up to `limit`.
pub fn retry_failed(conn: &Connection, limit: u32) -> Result<u32> {
    let mut stmt = conn.prepare(
        "
        SELECT id
        FROM ingest_jobs
        WHERE status = 'failed'
        ORDER BY id
        LIMIT ?1
        ",
    )?;
    let ids = stmt
        .query_map([i64::from(limit)], |row| row.get::<_, i64>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    for id in &ids {
        conn.execute(
            "
            UPDATE ingest_jobs
            SET status = 'queued', last_error = NULL, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?1
            ",
            [id],
        )?;
    }

    Ok(ids.len() as u32)
}

/// Lists most recent ingestion jobs.
pub fn list_jobs(conn: &Connection, limit: u32) -> Result<Vec<IngestJobRow>> {
    let mut stmt = conn.prepare(
        "
        SELECT id, status, attempts, last_error
        FROM ingest_jobs
        ORDER BY id DESC
        LIMIT ?1
        ",
    )?;

    let rows = stmt
        .query_map([i64::from(limit)], |row| {
            Ok(IngestJobRow {
                id: row.get(0)?,
                status: row.get(1)?,
                attempts: row.get(2)?,
                last_error: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(rows)
}

fn process_payload(payload: &str) -> Result<(String, String)> {
    // Deterministic fail trigger for transient/permanent failure-path tests.
    if payload.contains("__fail_parse__") {
        anyhow::bail!("parse failed for payload marker");
    }

    let normalized = payload.trim().to_lowercase();
    if normalized.is_empty() {
        anyhow::bail!("normalized payload is empty");
    }

    let classification = if normalized.contains("library") {
        "library"
    } else if normalized.contains("issue") {
        "issue"
    } else {
        "note"
    };

    Ok((normalized, classification.to_string()))
}

fn persist_result(
    conn: &Connection,
    job_id: i64,
    normalized_text: &str,
    classification: &str,
) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "
        INSERT INTO ingest_results (job_id, normalized_text, classification)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(job_id) DO UPDATE SET
            normalized_text = excluded.normalized_text,
            classification = excluded.classification
        ",
        params![job_id, normalized_text, classification],
    )?;
    tx.commit()?;
    Ok(())
}

fn payload_hash(payload: &str) -> String {
    let mut hasher = DefaultHasher::new();
    payload.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
