# Plan 06: Enforce Required Input Schema with Explicit Null Semantics

## Issue reference
- `docs/issues/06-required-explicit-null-input-schema.md`

## Scope
- Add JSON schemas for write paths, validation, `--print-schema`, and null-vs-empty tracking.

## Files
- Create: `config/knowledge/schemas/entity.schema.json`
- Create: `config/knowledge/schemas/lesson.schema.json`
- Create: `config/knowledge/schemas/issue.schema.json`
- Create: `config/knowledge/schemas/pipeline-payload.schema.json`
- Modify: `crates/knowledge-core/src/ingest.rs`
- Modify: `crates/knowledge-core/src/capture.rs` (or equivalent capture entrypoints)
- Modify: `crates/knowledge-cli/src/main.rs`
- Modify: `crates/knowledge-core/src/store.rs`
- Modify: `crates/knowledge-core/src/migrations.rs`
- Modify: `crates/knowledge-core/tests/schema_validation.rs` (create if missing)
- Modify: `skills/core/knowledge-update/SKILL.md`
- Modify: `skills/core/knowledge-refresh/SKILL.md`

## Tasks
1. Define required-field schemas (nullable where allowed) with `additionalProperties: false`.
2. Wire schema validation into all write paths before persistence.
3. Return aggregated missing-field validation errors with explicit-null hints.
4. Add `--print-schema` to each JSON-input command.
5. Persist null-vs-empty state distinctly for nullable collections.
6. Surface unknown/null counts in pipeline status output.
7. Add migration/version handling for schema evolution.
8. Add tests for missing-required, explicit-null acceptance, extra-field rejection, and version mismatch.
9. Update relevant skill docs to require `--print-schema` before authoring payloads.
10. Run verification baseline.

## Acceptance checks
- Missing keys are rejected with full missing-field list.
- Explicit `null` is accepted where schema allows.
- `--print-schema` available on all JSON-input commands.
