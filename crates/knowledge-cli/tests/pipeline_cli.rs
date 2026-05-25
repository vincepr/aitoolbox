use assert_cmd::Command;
use tempfile::tempdir;

#[test]
fn pipeline_enqueue_and_status_work() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("knowledge.sqlite3");
    let db_arg = db_path.to_string_lossy().to_string();

    let mut enqueue = Command::cargo_bin("knowledge-cli").unwrap();
    enqueue
        .args([
            "pipeline-enqueue",
            "--db",
            &db_arg,
            "--dedupe-key",
            "job-a",
            "--payload",
            "{\"$schema\":\"https://aitoolbox/schemas/pipeline-payload.v1.json\",\"payload\":\"hello\"}",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("state=queued"));

    let mut status = Command::cargo_bin("knowledge-cli").unwrap();
    status
        .args(["pipeline-status", "--db", &db_arg])
        .assert()
        .success()
        .stdout(predicates::str::contains("queued=1"))
        .stdout(predicates::str::contains("unknown_aliases=0"))
        .stdout(predicates::str::contains("unknown_notes=0"));
}
