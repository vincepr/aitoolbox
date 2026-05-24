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
            "--source-file",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "get",
            "--db",
            db.to_str().unwrap(),
            "--notes-root",
            notes.to_str().unwrap(),
            "--input-json",
            r#"{"entity":"MyCompanyName.Ebay.Custom.Client"}"#,
        ])
        .assert()
        .success()
        .stdout(contains("MyCompanyName.Ebay.Custom.Client"))
        .stdout(contains("No note summary stored"));
}

#[test]
fn get_accepts_positional_entity_with_default_paths() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("sources.json");
    let notes = temp.path().join("knowledge").join("notes");

    fs::create_dir_all(&notes).unwrap();
    fs::create_dir_all(temp.path().join(".local")).unwrap();
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
        .current_dir(temp.path())
        .args(["init", "--source-file", source.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .current_dir(temp.path())
        .args(["get", "MyCompanyName.Ebay.Custom.Client"])
        .assert()
        .success()
        .stdout(contains("MyCompanyName.Ebay.Custom.Client"))
        .stdout(contains("No note summary stored"));
}

#[test]
fn quickstart_creates_defaults_and_initializes_db() {
    let temp = tempdir().unwrap();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .current_dir(temp.path())
        .args(["quickstart"])
        .assert()
        .success()
        .stdout(contains("Database ready: .local/knowledge.sqlite3"))
        .stdout(contains("Notes root ready: knowledge/notes"))
        .stdout(contains(
            "Source file ready: config/knowledge/sources.example.json",
        ));

    assert!(temp
        .path()
        .join(".local")
        .join("knowledge.sqlite3")
        .exists());
    assert!(temp.path().join("knowledge").join("notes").exists());
    assert!(temp
        .path()
        .join("config")
        .join("knowledge")
        .join("sources.example.json")
        .exists());
}

#[test]
fn init_reads_source_json_from_source_json_flag() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let notes = temp.path().join("notes");
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
            "--db",
            db.to_str().unwrap(),
            "--notes-root",
            notes.to_str().unwrap(),
            "--input-json",
            r#"{"entity":"MyCompanyName.Ebay.Custom.Client"}"#,
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

#[test]
fn get_command_reports_no_match_as_informational_success() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let source = temp.path().join("sources.json");
    let notes = temp.path().join("notes");

    fs::write(&source, r#"{"entities":[]}"#).unwrap();

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
            "--db",
            db.to_str().unwrap(),
            "--notes-root",
            notes.to_str().unwrap(),
            "--input-json",
            r#"{"entity":"missing.entity"}"#,
        ])
        .assert()
        .success()
        .stdout(contains("No exact entity match found for missing.entity"));
}

#[test]
fn init_source_json_parse_errors_include_flag_context() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "init",
            "--db",
            db.to_str().unwrap(),
            "--source-json",
            "{\"entities\":[",
        ])
        .assert()
        .failure()
        .stderr(contains("failed to parse source file: --source-json"));
}

#[test]
fn get_input_json_parse_errors_include_command_context() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let source = temp.path().join("sources.json");
    let notes = temp.path().join("notes");

    fs::write(&source, r#"{"entities":[]}"#).unwrap();

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
            "--db",
            db.to_str().unwrap(),
            "--notes-root",
            notes.to_str().unwrap(),
            "--input-json",
            "{\"entity\":",
        ])
        .assert()
        .failure()
        .stderr(contains("failed to parse get input JSON"));
}

#[test]
fn capture_lesson_accepts_slug_and_body_flags() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "capture-lesson",
            "--db",
            db.to_str().unwrap(),
            "--slug",
            "avoid-global-singleton",
            "--body",
            "Global state leaked between tests",
        ])
        .assert()
        .success();
}

#[test]
fn capture_lesson_rejects_partial_field_flags() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "capture-lesson",
            "--db",
            db.to_str().unwrap(),
            "--slug",
            "missing-body",
        ])
        .assert()
        .failure()
        .stderr(contains(
            "both --slug and --body are required together for capture-lesson",
        ));
}

#[test]
fn completions_generates_bash_script() {
    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(contains("knowledge-cli"))
        .stdout(contains("complete"));
}

#[test]
fn alias_prints_shell_snippet() {
    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args(["alias", "bash"])
        .assert()
        .success()
        .stdout(contains("alias kno='knowledge-cli'"));
}
