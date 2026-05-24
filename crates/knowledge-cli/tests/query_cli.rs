use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn get_command_returns_summary_and_missing_mapping_message() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
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
            "--source-file",
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
        ])
        .assert()
        .success()
        .stdout(contains("MyCompanyName.Ebay.Custom.Client"))
        .stdout(contains("No note summary stored"));
}

#[test]
fn init_reads_source_json_from_source_json_flag() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let source_json = r#"{
      "entities": [
        {
          "canonical_name": "MyCompanyName.Ebay.Custom.Client",
          "kind": "library",
          "namespace": "MyCompanyName.Ebay.Custom.Client"
        }
      ]
    }"#;

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "init",
            "--db",
            db.to_str().unwrap(),
            "--source-json",
            source_json,
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
        ])
        .assert()
        .success()
        .stdout(contains("MyCompanyName.Ebay.Custom.Client"))
        .stdout(contains("No note summary stored"));
}

#[test]
fn query_alias_maps_to_get() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let source_json = r#"{
      "entities": [
        {
          "canonical_name": "MyCompanyName.Ebay.Custom.Client",
          "kind": "library",
          "namespace": "MyCompanyName.Ebay.Custom.Client"
        }
      ]
    }"#;

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "init",
            "--db",
            db.to_str().unwrap(),
            "--source-json",
            source_json,
        ])
        .assert()
        .success();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "query",
            "MyCompanyName.Ebay.Custom.Client",
            "--db",
            db.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("MyCompanyName.Ebay.Custom.Client"))
        .stdout(contains("No note summary stored"));
}

#[test]
fn init_rejects_both_source_file_and_source_json() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let source = temp.path().join("sources.json");
    fs::write(&source, r#"{"entities":[]}"#).unwrap();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "init",
            "--db",
            db.to_str().unwrap(),
            "--source-file",
            source.to_str().unwrap(),
            "--source-json",
            r#"{"entities":[]}"#,
        ])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}
