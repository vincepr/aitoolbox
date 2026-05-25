use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn get_output_matches_contract_v1() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("knowledge.db");
    let notes = temp.path().join("notes");
    let source = temp.path().join("sources.json");

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

    let out = Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "get",
            "--db",
            db.to_str().unwrap(),
            "--notes-root",
            notes.to_str().unwrap(),
            "MyCompanyName.Ebay.Custom.Client",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let actual = serde_json::json!({
        "entity": lines.first().copied().unwrap_or(""),
        "status": "ok",
        "summary": lines.get(1).copied().unwrap_or("")
    });

    let expected: Value = serde_json::from_str(include_str!(
        "../../knowledge-core/tests/fixtures/contracts/v1/get_success.json"
    ))
    .unwrap();
    assert_eq!(actual, expected);
}
