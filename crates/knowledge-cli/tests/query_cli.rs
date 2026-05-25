use assert_cmd::Command;
use predicates::str::contains;
use rusqlite::{params, Connection};
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
          "$schema": "https://aitoolbox/schemas/entity.v1.json",
          "entities": [
            {
              "canonical_name": "MyCompanyName.Ebay.Custom.Client",
              "kind": "library",
              "summary": null,
              "namespace": "MyCompanyName.Ebay.Custom.Client",
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
    let db = temp.path().join("homebase").join("knowledge.sqlite3");
    let notes = temp.path().join("homebase").join("notes");

    fs::create_dir_all(&notes).unwrap();
    fs::write(
        &source,
        r#"{
          "$schema": "https://aitoolbox/schemas/entity.v1.json",
          "entities": [
            {
              "canonical_name": "MyCompanyName.Ebay.Custom.Client",
              "kind": "library",
              "summary": null,
              "namespace": "MyCompanyName.Ebay.Custom.Client",
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
        .env("KNOWLEDGE_CLI_DB", db.to_str().unwrap())
        .env("KNOWLEDGE_CLI_NOTES_ROOT", notes.to_str().unwrap())
        .args(["init", "--source-file", source.to_str().unwrap()])
        .assert()
        .success();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .env("KNOWLEDGE_CLI_DB", db.to_str().unwrap())
        .env("KNOWLEDGE_CLI_NOTES_ROOT", notes.to_str().unwrap())
        .args(["get", "MyCompanyName.Ebay.Custom.Client"])
        .assert()
        .success()
        .stdout(contains("MyCompanyName.Ebay.Custom.Client"))
        .stdout(contains("No note summary stored"));
}

#[test]
fn quickstart_creates_defaults_and_initializes_db() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("quick").join("knowledge.sqlite3");
    let notes = temp.path().join("quick").join("notes");
    let source = temp.path().join("quick").join("sources.example.json");

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .env("KNOWLEDGE_CLI_DB", db.to_str().unwrap())
        .env("KNOWLEDGE_CLI_NOTES_ROOT", notes.to_str().unwrap())
        .env("KNOWLEDGE_CLI_SOURCE_FILE", source.to_str().unwrap())
        .args(["quickstart"])
        .assert()
        .success()
        .stdout(contains(format!(
            "Database ready: {}",
            db.to_str().unwrap()
        )))
        .stdout(contains(format!(
            "Notes root ready: {}",
            notes.to_str().unwrap()
        )))
        .stdout(contains(format!(
            "Source file ready: {}",
            source.to_str().unwrap()
        )));

    assert!(db.exists());
    assert!(notes.exists());
    assert!(source.exists());
}

#[test]
fn init_reads_source_json_from_source_json_flag() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let notes = temp.path().join("notes");
    let source_json = r#"{
      "$schema": "https://aitoolbox/schemas/entity.v1.json",
      "entities": [
        {
          "canonical_name": "MyCompanyName.Ebay.Custom.Client",
          "kind": "library",
          "summary": null,
          "namespace": "MyCompanyName.Ebay.Custom.Client",
          "package_name": null,
          "repo_name": null,
          "aliases": [],
          "location": null,
          "notes": []
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
    fs::write(
        &source,
        r#"{"$schema":"https://aitoolbox/schemas/entity.v1.json","entities":[]}"#,
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
            "--source-json",
            r#"{"$schema":"https://aitoolbox/schemas/entity.v1.json","entities":[]}"#,
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

    fs::write(
        &source,
        r#"{"$schema":"https://aitoolbox/schemas/entity.v1.json","entities":[]}"#,
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
            r#"{"entity":"missing.entity"}"#,
        ])
        .assert()
        .success()
        .stdout(contains("No exact entity match found for missing.entity"));
}

#[test]
fn list_supports_grep_hit_and_miss() {
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
              "namespace": "MyCompanyName.Ebay.Custom.Client",
              "repo_name": "CustomRepo"
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
        .args(["list", "--db", db.to_str().unwrap(), "--grep", "custom"])
        .assert()
        .success()
        .stdout(contains(
            "MyCompanyName.Ebay.Custom.Client\tlibrary\tCustomRepo",
        ));

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "list",
            "--db",
            db.to_str().unwrap(),
            "--grep",
            "does-not-exist",
        ])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn list_filters_by_kind_and_applies_limit() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let source = temp.path().join("sources.json");

    fs::write(
        &source,
        r#"{
          "entities": [
            {"canonical_name": "Alpha.Core", "kind": "library", "repo_name": "alpha"},
            {"canonical_name": "Beta.Core", "kind": "library", "repo_name": "beta"},
            {"canonical_name": "Ops.Service", "kind": "project", "repo_name": "ops"}
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

    let library_output = Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args(["list", "--db", db.to_str().unwrap(), "--kind", "library"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let library_output = String::from_utf8(library_output).unwrap();
    assert!(library_output.contains("Alpha.Core\tlibrary\talpha"));
    assert!(library_output.contains("Beta.Core\tlibrary\tbeta"));
    assert!(!library_output.contains("Ops.Service"));

    let limited_output = Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "list",
            "--db",
            db.to_str().unwrap(),
            "--kind",
            "library",
            "--limit",
            "1",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let limited_output = String::from_utf8(limited_output).unwrap();
    assert_eq!(limited_output.lines().count(), 1);
}

#[test]
fn list_matches_aliases() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let source = temp.path().join("sources.json");

    fs::write(
        &source,
        r#"{
          "entities": [
            {"canonical_name": "MyCompanyName.Ebay.Custom.Client", "kind": "library"}
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

    let conn = Connection::open(&db).unwrap();
    let entity_id = conn
        .query_row(
            "SELECT id FROM entities WHERE canonical_name = ?1",
            ["MyCompanyName.Ebay.Custom.Client"],
            |row| row.get::<_, i64>(0),
        )
        .unwrap();
    conn.execute(
        "INSERT INTO aliases (entity_id, alias) VALUES (?1, ?2)",
        params![entity_id, "PriceStock"],
    )
    .unwrap();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args(["list", "--db", db.to_str().unwrap(), "--grep", "pricestock"])
        .assert()
        .success()
        .stdout(contains("MyCompanyName.Ebay.Custom.Client\tlibrary\t"));
}

#[test]
fn get_command_prints_local_and_git_when_both_locations_exist() {
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
              "kind": "library",
              "local_path": "/workspace/MyCompanyName.Ebay.Custom.Client",
              "git_url": "https://example.com/repo.git"
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

    let output = Command::cargo_bin("knowledge-cli")
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

    let text = String::from_utf8(output).unwrap();
    assert_eq!(
        text,
        "MyCompanyName.Ebay.Custom.Client\nNo note summary stored\nlocal: /workspace/MyCompanyName.Ebay.Custom.Client\ngit:   https://example.com/repo.git\n"
    );
}

#[test]
fn get_command_prints_only_git_when_location_is_partial() {
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
              "kind": "library",
              "git_url": "https://example.com/repo.git"
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

    let output = Command::cargo_bin("knowledge-cli")
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

    let text = String::from_utf8(output).unwrap();
    assert_eq!(
        text,
        "MyCompanyName.Ebay.Custom.Client\nNo note summary stored\ngit:   https://example.com/repo.git\n"
    );
}

#[test]
fn get_command_keeps_two_line_output_when_location_absent() {
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

    let output = Command::cargo_bin("knowledge-cli")
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

    let text = String::from_utf8(output).unwrap();
    assert_eq!(
        text,
        "MyCompanyName.Ebay.Custom.Client\nNo note summary stored\n"
    );
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
        .stderr(contains("failed to parse input JSON payload"));
}

#[test]
fn get_input_json_parse_errors_include_command_context() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("nested").join("knowledge.db");
    let source = temp.path().join("sources.json");
    let notes = temp.path().join("notes");

    fs::write(
        &source,
        r#"{"$schema":"https://aitoolbox/schemas/entity.v1.json","entities":[]}"#,
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

#[test]
fn version_prints_semver() {
    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args(["version"])
        .assert()
        .success()
        .stdout(contains("0.2.0"));
}
