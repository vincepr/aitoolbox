use knowledge_core::ingest::{enqueue_job, list_jobs, retry_failed, run_once};
use knowledge_core::schema::bootstrap;
use rusqlite::Connection;

#[test]
fn job_is_persisted_before_processing() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let job_id = enqueue_job(&conn, "library payload").unwrap();
    let jobs = list_jobs(&conn, 10).unwrap();
    let created = jobs.iter().find(|j| j.id == job_id).unwrap();
    assert_eq!(created.status, "queued");

    let outcome = run_once(&conn).unwrap().unwrap();
    assert_eq!(outcome.initial_state, "queued");
}

#[test]
fn dedupe_reuses_existing_job_for_same_payload() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let first = enqueue_job(&conn, "same payload").unwrap();
    let second = enqueue_job(&conn, "same payload").unwrap();
    assert_eq!(first, second);

    let jobs = list_jobs(&conn, 10).unwrap();
    assert_eq!(jobs.len(), 1);
}

#[test]
fn failed_job_can_be_retried() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let job_id = enqueue_job(&conn, "__fail_parse__ payload").unwrap();
    let first_run = run_once(&conn).unwrap().unwrap();
    assert_eq!(first_run.final_state, "failed");

    let retried = retry_failed(&conn, 10).unwrap();
    assert_eq!(retried, 1);

    let jobs = list_jobs(&conn, 10).unwrap();
    let row = jobs.iter().find(|j| j.id == job_id).unwrap();
    assert_eq!(row.status, "queued");
}

#[test]
fn successful_job_persists_result_row() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    enqueue_job(&conn, "library text").unwrap();
    let run = run_once(&conn).unwrap().unwrap();
    assert_eq!(run.final_state, "succeeded");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ingest_results", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}
