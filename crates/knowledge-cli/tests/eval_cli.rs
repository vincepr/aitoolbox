use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn eval_emits_machine_readable_metrics_with_dataset_metadata() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("knowledge.db");
    let source = temp.path().join("sources.json");
    let dataset = temp.path().join("dataset.json");

    fs::write(
        &source,
        r#"{
          "entities": [
            {
              "canonical_name": "MyCompanyName.Ebay.Custom.Client",
              "kind": "library"
            }
          ]
        }"#,
    )
    .unwrap();

    fs::write(
        &dataset,
        r#"{
          "dataset_id": "smoke-v1",
          "version": "1.0.0",
          "generated_at": "2026-05-25T00:00:00Z",
          "cases": [
            {
              "query": "custom",
              "expected_entity": "MyCompanyName.Ebay.Custom.Client"
            }
          ]
        }"#,
    )
    .unwrap();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "init",
            "--db",
            db.to_str().unwrap(),
            "--source-file",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "eval",
            "--db",
            db.to_str().unwrap(),
            "--dataset",
            dataset.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("\"dataset_id\":\"smoke-v1\""))
        .stdout(contains("\"exact_match_rate\":"));
}

#[test]
fn eval_fails_when_threshold_is_not_met() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("knowledge.db");
    let source = temp.path().join("sources.json");
    let dataset = temp.path().join("dataset.json");

    fs::write(&source, r#"{"entities":[]}"#).unwrap();
    fs::write(
        &dataset,
        r#"{
          "dataset_id": "smoke-v1",
          "version": "1.0.0",
          "generated_at": "2026-05-25T00:00:00Z",
          "cases": [
            {
              "query": "missing",
              "expected_entity": "Nope"
            }
          ]
        }"#,
    )
    .unwrap();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "init",
            "--db",
            db.to_str().unwrap(),
            "--source-file",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "eval",
            "--db",
            db.to_str().unwrap(),
            "--dataset",
            dataset.to_str().unwrap(),
            "--fail-below-exact-match-rate",
            "0.5",
        ])
        .assert()
        .failure()
        .stderr(contains("below threshold"));
}
