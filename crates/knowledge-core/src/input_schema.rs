use anyhow::{Context, Result};
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;

/// Input schema identifiers for JSON-accepting write paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputSchemaKind {
    Entity,
    Lesson,
    Issue,
    PipelinePayload,
}

impl InputSchemaKind {
    /// Returns the expected `$schema` URI for this payload.
    pub fn uri(self) -> &'static str {
        match self {
            Self::Entity => "https://aitoolbox/schemas/entity.v1.json",
            Self::Lesson => "https://aitoolbox/schemas/lesson.v1.json",
            Self::Issue => "https://aitoolbox/schemas/issue.v1.json",
            Self::PipelinePayload => "https://aitoolbox/schemas/pipeline-payload.v1.json",
        }
    }

    /// Returns the raw JSON Schema document.
    pub fn schema_text(self) -> &'static str {
        match self {
            Self::Entity => include_str!("../../../config/knowledge/schemas/entity.schema.json"),
            Self::Lesson => include_str!("../../../config/knowledge/schemas/lesson.schema.json"),
            Self::Issue => include_str!("../../../config/knowledge/schemas/issue.schema.json"),
            Self::PipelinePayload => {
                include_str!("../../../config/knowledge/schemas/pipeline-payload.schema.json")
            }
        }
    }
}

/// Validates a raw JSON payload against one of the shipped input schemas.
///
/// Returns the parsed JSON value when validation succeeds.
pub fn validate_payload(raw_json: &str, kind: InputSchemaKind) -> Result<Value> {
    let value: Value =
        serde_json::from_str(raw_json).with_context(|| "failed to parse input JSON payload")?;
    let schema_value: Value = serde_json::from_str(kind.schema_text())
        .with_context(|| "failed to parse embedded JSON Schema")?;
    let schema_value: &'static Value = Box::leak(Box::new(schema_value));
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(schema_value)
        .with_context(|| "failed to compile embedded JSON Schema")?;

    if let Err(errors) = compiled.validate(&value) {
        let mut missing_hints = Vec::new();
        let mut details = Vec::new();
        for error in errors {
            let detail = format!("{}: {}", error.instance_path, error);
            if detail.contains("is a required property") {
                if let Some(start) = detail.find("\"") {
                    if let Some(end) = detail[start + 1..].find("\"") {
                        let field = &detail[start + 1..start + 1 + end];
                        missing_hints.push(format!(
                            "field '{field}' is required (use null if unknown, [] if known empty)"
                        ));
                    }
                }
            }
            details.push(detail);
        }

        missing_hints.sort();
        missing_hints.dedup();
        details.sort();
        details.dedup();

        let mut message = String::from("input schema validation failed");
        if !missing_hints.is_empty() {
            message.push('\n');
            message.push_str(&missing_hints.join("\n"));
        }
        if !details.is_empty() {
            message.push_str("\nvalidation details:\n");
            message.push_str(&details.join("\n"));
        }
        anyhow::bail!(message);
    }

    Ok(value)
}
