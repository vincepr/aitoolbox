use knowledge_core::ingest::{
    enqueue_job, run_once, DisabledProvider, IngestProvider, ProviderError,
};
use knowledge_core::schema::bootstrap;
use rusqlite::Connection;

#[derive(Debug)]
struct StaticProvider;

impl IngestProvider for StaticProvider {
    fn classify(&self, _normalized_payload: &str) -> std::result::Result<String, ProviderError> {
        Ok("ingested.entity".to_string())
    }
}

#[derive(Debug)]
struct TransientFailProvider;

impl IngestProvider for TransientFailProvider {
    fn classify(&self, _normalized_payload: &str) -> std::result::Result<String, ProviderError> {
        Err(ProviderError::Transient("temporary".to_string()))
    }
}

#[test]
fn job_is_persisted_before_processing() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let result = enqueue_job(&conn, "payload", "dedupe-a", 3).unwrap();
    assert_eq!(
        result.initial_state,
        knowledge_core::ingest::IngestState::Queued
    );
    assert!(!result.deduped);
}

#[test]
fn dedupe_key_prevents_duplicate_jobs() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let first = enqueue_job(&conn, "payload", "dedupe-a", 3).unwrap();
    let second = enqueue_job(&conn, "payload", "dedupe-a", 3).unwrap();

    assert_eq!(first.job_id, second.job_id);
    assert!(second.deduped);

    let count: u32 = conn
        .query_row("SELECT COUNT(*) FROM ingestion_jobs", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn provider_failure_is_recorded_and_retry_is_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    enqueue_job(&conn, "payload", "dedupe-a", 2).unwrap();

    let first = run_once(&conn, &TransientFailProvider).unwrap().unwrap();
    assert_eq!(first.state, knowledge_core::ingest::IngestState::Failed);
    assert_eq!(first.phase, knowledge_core::ingest::IngestPhase::Classify);

    let second = run_once(&conn, &TransientFailProvider).unwrap().unwrap();
    assert_eq!(second.state, knowledge_core::ingest::IngestState::Failed);

    let attempts: u32 = conn
        .query_row(
            "SELECT attempt_count FROM ingestion_jobs WHERE dedupe_key = 'dedupe-a'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(attempts, 2);

    let third = run_once(&conn, &TransientFailProvider).unwrap();
    assert!(third.is_none());
}

#[test]
fn provider_success_persists_once() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    enqueue_job(&conn, "payload", "dedupe-a", 3).unwrap();

    let outcome = run_once(&conn, &StaticProvider).unwrap().unwrap();
    assert_eq!(
        outcome.state,
        knowledge_core::ingest::IngestState::Succeeded
    );

    let entities: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM entities WHERE canonical_name = 'ingested.entity'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(entities, 1);

    let writes: u32 = conn
        .query_row("SELECT COUNT(*) FROM ingestion_domain_writes", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(writes, 1);
}

#[test]
fn provider_off_is_structured_failure() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    enqueue_job(&conn, "payload", "dedupe-a", 3).unwrap();
    let outcome = run_once(&conn, &DisabledProvider).unwrap().unwrap();

    assert_eq!(outcome.state, knowledge_core::ingest::IngestState::Failed);
    assert_eq!(outcome.phase, knowledge_core::ingest::IngestPhase::Classify);
    assert!(outcome.error.unwrap().contains("disabled"));
}
