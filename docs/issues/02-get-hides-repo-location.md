# 02 — `get` hides the entity's local clone path and git URL

Status: `[planned]`
Effort: small (~30 LOC across `knowledge-core` and `knowledge-cli`)
Area: `crates/knowledge-core`, `crates/knowledge-cli`

## Symptom

The whole purpose of the skill is to spare agents from decompiling NuGet DLLs or scanning `~/.nuget/packages` — by handing them the canonical source location. The index stores exactly that, in the `locations` table:

```
sqlite> SELECT * FROM locations WHERE entity_id =
        (SELECT id FROM entities WHERE canonical_name = 'laika-marketplaces-jobs-pricestock');
210|/Users/vincent.probst/RiderProjects/laika/Marketplaces/Jobs/PriceStock|https://gitlab.relaxdays.de/laika/Marketplaces/Jobs/PriceStock.git
```

But `knowledge-cli get` prints only:

```
laika-marketplaces-jobs-pricestock
No note summary stored
```

The single most useful field for an agent (a path to readable source-with-comments) is in the database but never reaches stdout. The agent then falls back to `ls ~/.nuget/...` and `ilspycmd`.

## Root cause

Three places conspire:

**a) `QueryAnswer` struct has no location fields** — `crates/knowledge-core/src/store.rs:83-90`

```rust
pub struct QueryAnswer {
    pub canonical_name: String,
    pub summary: String,
    pub navigation_hints: Vec<String>,  // always empty, "reserved" per doc-comment
}
```

**b) `query_exact` never reads the `locations` table** — `crates/knowledge-core/src/store.rs:304-329`
Reads `note_refs` but not `locations`. No JOIN, no second query.

**c) `print_get_result` only formats `canonical_name` + `summary`** — `crates/knowledge-cli/src/main.rs:618-630`
Even if (a) and (b) were fixed, the CLI surface would still drop the location.

## Proposed fix

1. Add a location field to `QueryAnswer`:

   ```rust
   pub struct QueryAnswer {
       pub canonical_name: String,
       pub summary: String,
       pub location: Option<EntityLocation>,
       pub navigation_hints: Vec<String>,
   }

   pub struct EntityLocation {
       pub local_path: Option<String>,
       pub git_url: Option<String>,
   }
   ```

2. Extend `query_exact` to read the `locations` row for the matched entity (one extra `query_row().optional()` call, same pattern already used for `note_refs`).

3. Extend `print_get_result` to emit the location after the summary, e.g.:

   ```
   laika-marketplaces-jobs-pricestock
   No note summary stored
   local: /Users/vincent.probst/RiderProjects/laika/Marketplaces/Jobs/PriceStock
   git:   https://gitlab.relaxdays.de/laika/Marketplaces/Jobs/PriceStock.git
   ```

   Hide a line if its field is `None`.

4. For the `--input-json` / structured-output path, include the location as a nested JSON object so machine consumers can use it without parsing free text.

## Acceptance criteria

- `knowledge-cli get laika-marketplaces-jobs-pricestock` prints both the local path and the git URL when present.
- When neither is populated, the existing two-line output is preserved (no spurious empty lines).
- A query-CLI test in `crates/knowledge-cli/tests/query_cli.rs` covers: location-present, location-partial (only git), location-absent.
- `crates/knowledge-core/tests/exact_lookup.rs` asserts that `query_exact` returns the `EntityLocation` when populated.

## Why this is the single biggest-impact fix

For class- or behavior-level questions, the index will rarely have summary text in time. But the location *is* already populated by `init` for every entity. Exposing it means the skill becomes useful immediately — even before issues 3 and 4 are in.
