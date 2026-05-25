# 04 — Ingestion under-populates aliases, namespace, package_name

Status: `[planned]`
Effort: medium (touches ingestion sources + pipeline)
Area: `crates/knowledge-core` (ingest), `config/knowledge/*`

## Symptom

The store's lookup is actually well-designed. `find_primary_entity` (`crates/knowledge-core/src/store.rs:331-364`) matches on **any** of: `canonical_name`, `namespace`, `package_name`, `repo_name`, or `aliases.alias`. So in theory:

```console
$ knowledge-cli get Relaxdays.Laika.Marketplaces.Jobs.PriceStock   # nuget package
$ knowledge-cli get PriceStock                                      # repo name
$ knowledge-cli get PriceStockExportJob                             # class name (as alias)
```

…should all resolve to `laika-marketplaces-jobs-pricestock`. But in practice:

```
sqlite> SELECT id, canonical_name, namespace, package_name, repo_name
        FROM entities WHERE canonical_name = 'laika-marketplaces-jobs-pricestock';
210|laika-marketplaces-jobs-pricestock|<empty>|<empty>|PriceStock
sqlite> SELECT * FROM aliases WHERE entity_id = 210;
(no rows)
```

Only `canonical_name` and `repo_name` are filled. `namespace`, `package_name`, and `aliases` are empty for most entities — so the lookup paths the store offers are never reachable.

`PriceStock` (bare repo name) does match, but no agent will guess that — they will try the full `Relaxdays.*` namespace or the class name.

## Root cause

The ingestion pipeline (`crates/knowledge-core/src/ingest.rs`, sources in `config/knowledge/*.json`) writes the canonical name and repo name but does not derive:

- `namespace`: e.g. `Relaxdays.Laika.Marketplaces.Jobs.PriceStock` (the .NET root namespace inferred from `repo_name` and the `laika/` group convention).
- `package_name`: the published NuGet package id, identical or near-identical to `namespace`.
- `aliases`: common alternative spellings — dot-PascalCase, slash-path (`laika/Marketplaces/Jobs/PriceStock`), and (where cheap to extract) public type names from the repo's `*.cs` files.

## Proposed fix

In ingestion, for each `laika-*` entity:

1. Derive `namespace` deterministically from canonical name: `laika-marketplaces-jobs-pricestock` → `Relaxdays.Laika.Marketplaces.Jobs.PriceStock`. (Hyphen → dot, segments title-cased; `laika` → `Relaxdays.Laika`.)
2. Set `package_name = namespace` for Laika repos (they share the convention).
3. Emit aliases for at least:
   - the slash-path form (`laika/Marketplaces/Jobs/PriceStock`)
   - the bare last segment (`PriceStock`) — already covered by `repo_name` match but adding it as an alias is harmless and explicit.
   - (stretch) public type names harvested from `.cs` files in the repo's `src/` directory, capped at N per entity to bound size.

Make the namespace prefix mapping (e.g. `laika` → `Relaxdays.Laika`) a per-source config field in `config/knowledge/sources.example.json`, so other organisations using the same tool can supply their own mapping.

## Acceptance criteria

- After re-running `knowledge-cli init`, the `entities` row for `laika-marketplaces-jobs-pricestock` has populated `namespace` and `package_name`.
- The `aliases` table has at least 2 rows for that entity.
- `knowledge-cli get Relaxdays.Laika.Marketplaces.Jobs.PriceStock` resolves correctly.
- Re-ingestion is idempotent: running `init` twice does not duplicate aliases.
- `crates/knowledge-core/tests/import_sources.rs` covers namespace derivation, alias dedup, and the configurable prefix mapping.

## Open question

Whether to harvest type names from the repo source. Pro: enables `knowledge-cli get PriceStockExportJob` directly. Con: increases ingestion time, requires the repo to be locally cloned at ingest time, and risks alias explosion. Suggest: gated behind an opt-in `--harvest-types` flag on `init`.
