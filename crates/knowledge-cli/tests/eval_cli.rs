use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn eval_emits_machine_readable_metrics() {
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
        r#"[
          {
            "query": "custom",
            "expected_entity": "MyCompanyName.Ebay.Custom.Client"
          }
        ]"#,
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
        .stdout(contains("\"total\":"))
        .stdout(contains("\"exact_matches\":"))
        .stdout(contains("\"exact_match_rate\":"));
}
