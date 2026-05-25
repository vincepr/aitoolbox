use knowledge_core::import::apply_source_json;
use knowledge_core::input_schema::{validate_payload, InputSchemaKind};
use knowledge_core::schema::bootstrap;
use rusqlite::Connection;

#[test]
fn missing_required_field_is_rejected() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let json = r#"{
      "$schema": "https://aitoolbox/schemas/entity.v1.json",
      "entities": [
        {
          "canonical_name": "ebay-common",
          "kind": "project",
          "summary": null,
          "namespace": null,
          "package_name": null,
          "repo_name": null,
          "location": null,
          "notes": null
        }
      ]
    }"#;

    let err = apply_source_json(&conn, json, "test")
        .unwrap_err()
        .to_string();
    assert!(err.contains("schema validation"));
}

#[test]
fn explicit_null_is_accepted() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let json = r#"{
      "$schema": "https://aitoolbox/schemas/entity.v1.json",
      "entities": [
        {
          "canonical_name": "ebay-common",
          "kind": "project",
          "summary": null,
          "namespace": null,
          "package_name": null,
          "repo_name": null,
          "aliases": null,
          "location": null,
          "notes": null
        }
      ]
    }"#;

    apply_source_json(&conn, json, "test").unwrap();
}

#[test]
fn additional_properties_are_rejected() {
    let payload = r#"{
      "$schema": "https://aitoolbox/schemas/lesson.v1.json",
      "slug": "a",
      "body": "b",
      "extra": "unexpected"
    }"#;

    let err = validate_payload(payload, InputSchemaKind::Lesson)
        .unwrap_err()
        .to_string();
    assert!(err.contains("input schema validation failed"));
    assert!(err.contains("extra"));
}

#[test]
fn schema_version_mismatch_is_rejected() {
    let payload = r#"{
      "$schema": "https://aitoolbox/schemas/lesson.v9.json",
      "slug": "a",
      "body": "b"
    }"#;

    let err = validate_payload(payload, InputSchemaKind::Lesson)
        .unwrap_err()
        .to_string();
    assert!(err.contains("input schema validation failed"));
    assert!(err.contains("lesson.v1.json"));
}
