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
          "$schema": "https://aitoolbox/schemas/entity.v1.json",
          "entities": [
            {
              "canonical_name": "MyCompanyName.Ebay.Custom.Client",
              "kind": "library",
              "summary": null,
              "namespace": null,
              "package_name": null,
              "repo_name": null,
              "aliases": [],
              "location": null,
              "notes": []
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
    let local_path = lines
        .iter()
        .find_map(|line| line.strip_prefix("local: ").map(str::to_string));
    let git_url = lines
        .iter()
        .find_map(|line| line.strip_prefix("git:   ").map(str::to_string));
    let location = if local_path.is_none() && git_url.is_none() {
        Value::Null
    } else {
        serde_json::json!({
            "local_path": local_path,
            "git_url": git_url
        })
    };
    let actual = serde_json::json!({
        "entity": lines.first().copied().unwrap_or(""),
        "status": "ok",
        "summary": lines.get(1).copied().unwrap_or(""),
        "location": location
    });

    let expected: Value = serde_json::from_str(include_str!(
        "../../knowledge-core/tests/fixtures/contracts/v1/get_success.json"
    ))
    .unwrap();
    assert_eq!(actual, expected);
}
