# 06 — Input capture schemas should require every field (explicit-null vs absent-key)

Status: `[planned]`
Effort: medium (schema definitions + validation + `--print-schema` subcommand)
Area: `crates/knowledge-core` (ingest, capture), `crates/knowledge-cli`, `skills/core/*`

## Symptom

The skill's **output contract** in `skills/core/knowledge-get/SKILL.md` is already strict — every field of the response is required, no "drop the key if empty". The skill's **input contract** for ingestion and capture is the opposite: keys can simply be absent and ingestion silently moves on with empty columns. The asymmetry is what lets `aliases` and `namespace` end up empty for 200+ entities without anyone noticing (see issue 04).

Concrete shape of the gap:

```jsonc
// today — both of these are accepted, indistinguishable downstream
{ "canonical_name": "laika-marketplaces-jobs-pricestock", "kind": "library" }
{ "canonical_name": "laika-marketplaces-jobs-pricestock", "kind": "library", "aliases": [] }
```

Neither agents nor humans are ever prompted to think "should `aliases` actually be empty?" The key just isn't there.

## Why required-with-explicit-null helps

Prior art for the pattern:

- TypeScript `exactOptionalPropertyTypes` — distinguishes `key: undefined` from absent key.
- GraphQL — `null` vs missing means "explicitly unset" vs "unknown / don't apply".
- LLM JSON generation — when a schema enumerates every field with "use null if unknown", models hallucinate fewer defaults and omit fewer fields than when fields can simply be absent. The schema is a checklist the model walks down.

The argument is symmetry: if the *output* contract is strict, the *input* contract should be too. Same shape, opposite direction.

Two-state distinction worth modeling:

- `[]` — known to be empty (we checked; this entity has no aliases).
- `null` — unknown (we have not yet derived aliases). The ingestion pipeline can flag `null` rows for follow-up while leaving `[]` rows alone. This information is lost today.

## Proposed fix

### 1. Define a JSON Schema for each input type

One schema file per capture/import shape, checked into `config/knowledge/schemas/`:

- `entity.schema.json` — for `init` source records and any future `capture-entity`.
- `lesson.schema.json` — for `capture-lesson`.
- `issue.schema.json` — for `capture-issue`.
- `pipeline-payload.schema.json` — for `pipeline-enqueue`.

Every property declared in `required` (no exceptions). Optional values modeled as nullable, e.g.:

```jsonc
{
  "type": "object",
  "additionalProperties": false,
  "required": ["canonical_name", "kind", "summary", "namespace",
               "package_name", "repo_name", "aliases", "location", "notes"],
  "properties": {
    "canonical_name": { "type": "string", "pattern": "^[a-z0-9][a-z0-9-]*$" },
    "kind":           { "enum": ["domain", "system", "library", "project", "lesson"] },
    "summary":        { "type": ["string", "null"] },
    "namespace":      { "type": ["string", "null"] },
    "package_name":   { "type": ["string", "null"] },
    "repo_name":      { "type": ["string", "null"] },
    "aliases":        { "type": ["array", "null"], "items": { "type": "string" } },
    "location":       { "$ref": "#/definitions/location_or_null" },
    "notes":          { "type": ["array", "null"], "items": { "type": "string" } }
  }
}
```

### 2. Validate on every write path

- `knowledge-cli init` rejects source records that violate the schema with `field 'aliases' is required (use null if unknown, [] if known empty)`.
- `capture-lesson`, `capture-issue`, `pipeline-enqueue` validate the JSON payload before writing.
- Validation error messages name **every** missing field, not just the first — agents fixing the document benefit from seeing the full list.

### 3. Surface the schema to agents and humans

Add a `--print-schema` flag to every command that accepts JSON input:

```console
$ knowledge-cli init --print-schema
$ knowledge-cli capture-lesson --print-schema
$ knowledge-cli pipeline-enqueue --print-schema
```

Emit the JSON Schema to stdout. SKILL.md files instruct agents to run this once before authoring a capture, so the schema acts as the form to fill in.

### 4. Distinguish `null` from `[]` downstream

In `knowledge-core::store`, store `null` aliases/notes as a sentinel separate from `[]` (e.g. a `state` column on `aliases` per-entity, or a dedicated `entities.aliases_state ENUM('unknown', 'known')`). The `pipeline-status` subcommand reports counts of `unknown`-state rows so we can see "we have 200 entities with unknown aliases" — those become the queue for follow-up enrichment.

### 5. Schema versioning

Add `"$schema": "https://aitoolbox/schemas/entity.v1.json"` to every input document. When the schema changes, bump the version and write a one-shot migration in `knowledge-core/src/migrations.rs`. Avoids the "every new optional field is a breaking change" foot-gun.

## Scope clarification

This issue is about **input format strictness**, not about *what* the fields should be. Issue 04 defines the alias/namespace content for Laika repos. Together:

- 04 says: "when we can auto-derive, fill these specific fields."
- 06 says: "when we can't auto-derive, the format forces an explicit `null` so the gap is visible instead of silent."

Both ship to close the same problem from both ends.

## Acceptance criteria

- JSON Schemas exist for every input type and are checked into `config/knowledge/schemas/`.
- All write paths (`init`, `capture-*`, `pipeline-enqueue`) validate against the relevant schema before committing.
- Validation errors enumerate **all** missing fields with a one-line hint each.
- `<command> --print-schema` emits the schema to stdout for every JSON-accepting command.
- `null` and `[]` are stored distinctly for nullable collection fields; `pipeline-status` reports counts of `null` ("unknown") collections per kind.
- A new test file `crates/knowledge-core/tests/schema_validation.rs` covers: missing-required rejection, explicit-null accepted, `additionalProperties: false` rejection, schema-version mismatch rejection.
- SKILL.md files for `knowledge-update` and `knowledge-refresh` reference `--print-schema` as the first step before authoring captures.

## Trade-offs accepted

- **Boilerplate** in input documents. Mitigated by `--print-schema` + good templates in `knowledge/templates/`.
- **Schema evolution cost.** Mitigated by `$schema` versioning and one-shot migrations.
- **Bulk auto-ingestion noise** (e.g. pulling NuGet metadata). The auto-deriver fills every field; for those paths the strictness is harmless. The win is concentrated in human / agent capture, which is exactly the place where forgetting fields matters most.
