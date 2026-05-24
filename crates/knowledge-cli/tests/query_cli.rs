use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn get_command_returns_summary_and_missing_mapping_message() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let notes = temp.path().join("notes");
    let source = temp.path().join("sources.json");

    fs::write(
        &source,
        r#"{
          "entities": [
            {
              "canonical_name": "MyCompanyName.Ebay.Custom.Client",
              "kind": "library",
              "namespace": "MyCompanyName.Ebay.Custom.Client"
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
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "get",
            "MyCompanyName.Ebay.Custom.Client",
            "--db",
            db.to_str().unwrap(),
            "--notes-root",
            notes.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("MyCompanyName.Ebay.Custom.Client"))
        .stdout(contains("No note summary stored"));
}
