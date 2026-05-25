use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use thiserror::Error;

/// Processing phase for a pipeline run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestPhase {
    Parse,
    Normalize,
    Classify,
    Persist,
}

impl IngestPhase {
    fn as_str(self) -> &'static str {
        match self {
            IngestPhase::Parse => "parse",
            IngestPhase::Normalize => "normalize",
            IngestPhase::Classify => "classify",
            IngestPhase::Persist => "persist",
        }
    }
}

/// Persisted job state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestState {
    Queued,
    Processing,
    Succeeded,
    Failed,
}

impl IngestState {
    fn as_str(self) -> &'static str {
        match self {
            IngestState::Queued => "queued",
            IngestState::Processing => "processing",
            IngestState::Succeeded => "succeeded",
            IngestState::Failed => "failed",
        }
    }
}

/// Result of enqueueing a raw payload into the ingestion queue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnqueueResult {
    pub job_id: i64,
    pub initial_state: IngestState,
    pub deduped: bool,
}

/// Processing outcome for one pipeline run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub job_id: i64,
    pub state: IngestState,
    pub phase: IngestPhase,
    pub attempts: u32,
    pub error: Option<String>,
}

/// Queue-state counters plus unknown collection coverage counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueStatus {
    pub queued: u64,
    pub processing: u64,
    pub failed: u64,
    pub unknown_aliases: u64,
    pub unknown_notes: u64,
}

/// Provider abstraction used during classification phase.
pub trait IngestProvider {
    fn classify(&self, normalized_payload: &str) -> std::result::Result<String, ProviderError>;
}

/// Provider used by default when classification is disabled.
#[derive(Debug, Default)]
pub struct DisabledProvider;

impl IngestProvider for DisabledProvider {
    fn classify(&self, _normalized_payload: &str) -> std::result::Result<String, ProviderError> {
        Err(ProviderError::Disabled)
    }
}

/// Provider error that keeps failures structured for retry policy.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProviderError {
    #[error("provider is disabled")]
    Disabled,
    #[error("transient provider failure: {0}")]
    Transient(String),
    #[error("permanent provider failure: {0}")]
    Permanent(String),
}

#[derive(Debug)]
struct JobRow {
    id: i64,
    raw_payload: String,
    attempt_count: u32,
    max_attempts: u32,
}

/// Enqueues a raw payload for asynchronous ingestion.
///
/// # Arguments
///
/// * `conn` - SQLite connection.
/// * `raw_payload` - Raw input payload.
/// * `dedupe_key` - Stable deduplication key.
/// * `max_attempts` - Maximum retries before terminal failure.
///
/// # Returns
///
/// Queue insertion metadata.
///
/// # Errors
///
/// Returns an error when writes fail or `max_attempts` is invalid.
pub fn enqueue_job(
    conn: &Connection,
    raw_payload: &str,
    dedupe_key: &str,
    max_attempts: u32,
) -> Result<EnqueueResult> {
    if max_attempts == 0 {
        anyhow::bail!("max_attempts must be >= 1");
    }

    conn.execute(
        "
        INSERT OR IGNORE INTO ingestion_jobs (dedupe_key, raw_payload, state, attempt_count, max_attempts)
        VALUES (?1, ?2, ?3, 0, ?4)
        ",
        params![dedupe_key, raw_payload, IngestState::Queued.as_str(), max_attempts],
    )?;

    let deduped = conn.changes() == 0;
    let job_id = conn.query_row(
        "SELECT id FROM ingestion_jobs WHERE dedupe_key = ?1",
        [dedupe_key],
        |row| row.get::<_, i64>(0),
    )?;

    Ok(EnqueueResult {
        job_id,
        initial_state: IngestState::Queued,
        deduped,
    })
}

/// Runs exactly one pending ingestion job.
///
/// # Arguments
///
/// * `conn` - SQLite connection.
/// * `provider` - Classification provider hook.
///
/// # Returns
///
/// `Some(outcome)` when work was processed, otherwise `None` when queue is empty.
///
/// # Errors
///
/// Returns an error when SQL operations fail.
pub fn run_once(conn: &Connection, provider: &dyn IngestProvider) -> Result<Option<RunOutcome>> {
    let job = next_job(conn)?;
    let Some(job) = job else {
        return Ok(None);
    };

    transition_state(conn, job.id, IngestState::Processing, None)?;

    let parsed = match parse_phase(&job.raw_payload) {
        Ok(parsed) => parsed,
        Err(err) => {
            return mark_failed(conn, &job, IngestPhase::Parse, err.to_string(), false);
        }
    };

    let normalized = normalize_phase(&parsed);

    let classified = match provider.classify(&normalized) {
        Ok(classified) => classified,
        Err(ProviderError::Disabled) => {
            return mark_failed(
                conn,
                &job,
                IngestPhase::Classify,
                ProviderError::Disabled.to_string(),
                true,
            );
        }
        Err(ProviderError::Transient(msg)) => {
            return mark_failed(conn, &job, IngestPhase::Classify, msg, false);
        }
        Err(ProviderError::Permanent(msg)) => {
            return mark_failed(conn, &job, IngestPhase::Classify, msg, true);
        }
    };

    persist_phase(conn, job.id, &classified)?;
    transition_state(conn, job.id, IngestState::Succeeded, None)?;
    write_result(
        conn,
        job.id,
        IngestPhase::Persist,
        IngestState::Succeeded,
        None,
    )?;

    Ok(Some(RunOutcome {
        job_id: job.id,
        state: IngestState::Succeeded,
        phase: IngestPhase::Persist,
        attempts: job.attempt_count.saturating_add(1),
        error: None,
    }))
}

/// Returns queue status counts.
pub fn queue_status(conn: &Connection) -> Result<QueueStatus> {
    let queued = count_by_state(conn, IngestState::Queued)?;
    let processing = count_by_state(conn, IngestState::Processing)?;
    let failed = count_by_state(conn, IngestState::Failed)?;
    let unknown_aliases = count_unknown_collection_state(conn, "aliases_state")?;
    let unknown_notes = count_unknown_collection_state(conn, "notes_state")?;
    Ok(QueueStatus {
        queued,
        processing,
        failed,
        unknown_aliases,
        unknown_notes,
    })
}

fn parse_phase(raw_payload: &str) -> Result<String> {
    let parsed = raw_payload.trim();
    if parsed.is_empty() {
        anyhow::bail!("raw payload is empty")
    }
    Ok(parsed.to_string())
}

fn normalize_phase(parsed: &str) -> String {
    parsed.to_ascii_lowercase()
}

fn persist_phase(conn: &Connection, job_id: i64, entity_name: &str) -> Result<()> {
    conn.execute(
        "
        INSERT INTO entities (canonical_name, kind, summary)
        VALUES (?1, 'tag', 'ingested from pipeline')
        ON CONFLICT(canonical_name) DO UPDATE SET
            updated_at = CURRENT_TIMESTAMP
        ",
        [entity_name],
    )
    .context("failed to persist ingested entity")?;

    conn.execute(
        "
        INSERT OR IGNORE INTO ingestion_domain_writes (job_id, entity_canonical_name)
        VALUES (?1, ?2)
        ",
        params![job_id, entity_name],
    )?;
    Ok(())
}

fn next_job(conn: &Connection) -> Result<Option<JobRow>> {
    let job = conn
        .query_row(
            "
            SELECT id, raw_payload, attempt_count, max_attempts
            FROM ingestion_jobs
            WHERE state IN ('queued', 'failed')
              AND attempt_count < max_attempts
            ORDER BY id
            LIMIT 1
            ",
            [],
            |row| {
                Ok(JobRow {
                    id: row.get(0)?,
                    raw_payload: row.get(1)?,
                    attempt_count: row.get::<_, u32>(2)?,
                    max_attempts: row.get::<_, u32>(3)?,
                })
            },
        )
        .optional()?;
    Ok(job)
}

fn transition_state(
    conn: &Connection,
    job_id: i64,
    state: IngestState,
    last_error: Option<&str>,
) -> Result<()> {
    conn.execute(
        "
        UPDATE ingestion_jobs
        SET state = ?2,
            last_error = ?3,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        ",
        params![job_id, state.as_str(), last_error],
    )?;
    Ok(())
}

fn increment_attempt(conn: &Connection, job_id: i64) -> Result<u32> {
    conn.execute(
        "
        UPDATE ingestion_jobs
        SET attempt_count = attempt_count + 1,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?1
        ",
        [job_id],
    )?;

    let attempts = conn.query_row(
        "SELECT attempt_count FROM ingestion_jobs WHERE id = ?1",
        [job_id],
        |row| row.get::<_, u32>(0),
    )?;
    Ok(attempts)
}

fn mark_failed(
    conn: &Connection,
    job: &JobRow,
    phase: IngestPhase,
    error: String,
    permanent: bool,
) -> Result<Option<RunOutcome>> {
    let attempts = increment_attempt(conn, job.id)?;
    let terminal = permanent || attempts >= job.max_attempts;
    let next_state = if terminal {
        IngestState::Failed
    } else {
        IngestState::Queued
    };

    transition_state(conn, job.id, next_state, Some(&error))?;
    write_result(conn, job.id, phase, IngestState::Failed, Some(&error))?;

    Ok(Some(RunOutcome {
        job_id: job.id,
        state: IngestState::Failed,
        phase,
        attempts,
        error: Some(error),
    }))
}

fn write_result(
    conn: &Connection,
    job_id: i64,
    phase: IngestPhase,
    state: IngestState,
    error: Option<&str>,
) -> Result<()> {
    conn.execute(
        "
        INSERT INTO ingestion_results (job_id, phase, state, error)
        VALUES (?1, ?2, ?3, ?4)
        ",
        params![job_id, phase.as_str(), state.as_str(), error],
    )?;
    Ok(())
}

fn count_by_state(conn: &Connection, state: IngestState) -> Result<u64> {
    let count = conn.query_row(
        "SELECT COUNT(*) FROM ingestion_jobs WHERE state = ?1",
        [state.as_str()],
        |row| row.get::<_, u64>(0),
    )?;
    Ok(count)
}

fn count_unknown_collection_state(conn: &Connection, column: &str) -> Result<u64> {
    let sql = format!("SELECT COUNT(*) FROM entities WHERE {column} = 'unknown'");
    let count = conn.query_row(&sql, [], |row| row.get::<_, u64>(0))?;
    Ok(count)
}
