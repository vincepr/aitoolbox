# 04 — Ingestion under-populates aliases, namespace, package_name

Status: `[planned]`
Effort: medium (touches ingestion sources + pipeline)
Area: `crates/knowledge-core` (ingest), `config/knowledge/*`

## Symptom

The store's lookup is well-designed. `find_primary_entity` (`crates/knowledge-core/src/store.rs:331-364`) matches on **any** of: `canonical_name`, `namespace`, `package_name`, `repo_name`, or `aliases.alias`. In theory:

```console
$ knowledge-cli get Relaxdays.Laika.Marketplaces.Jobs.PriceStock   # nuget package
$ knowledge-cli get laika/Marketplaces/Jobs/PriceStock              # gitlab path
$ knowledge-cli get PriceStockExportJob                             # class name (as alias)
```

…should all resolve to `laika-marketplaces-jobs-pricestock`. In practice:

```
sqlite> SELECT id, canonical_name, namespace, package_name, repo_name
        FROM entities WHERE canonical_name = 'laika-marketplaces-jobs-pricestock';
210|laika-marketplaces-jobs-pricestock|<empty>|<empty>|PriceStock
sqlite> SELECT * FROM aliases WHERE entity_id = 210;
(no rows)
```

Only `canonical_name` and `repo_name` are filled. `namespace`, `package_name`, and `aliases` are empty for most entities — so the lookup paths the store offers are never reachable. `PriceStock` (bare repo name) *does* match, but no agent will guess that — they will try the full `Relaxdays.*` namespace or the class name.

## Design choice: canonical stays, queries land via aliases

The canonical kebab form (`laika-marketplaces-jobs-pricestock`) is a sensible **internal storage ID**: stable, ASCII-safe, free of casing ambiguity. Agents should not be asked to know it. Instead, every natural query form is exposed as an **alias** that resolves to the canonical.

So: **do not change the canonical-name format**. Fill the empty `namespace` / `package_name` columns deterministically, and write enough alias rows that the queries an agent actually issues will hit.

## Decision: granularity — repo-level only

There are two granularities we could index at:

| Granularity     | Example entity                                                       | "PriceStockExportJob" query | Cost |
|-----------------|----------------------------------------------------------------------|-----------------------------|------|
| Repo / package  | one entity per Laika repo (current model)                            | misses; agent must truncate to namespace, get repo path, then `grep`. | low — derivable from repo metadata only. |
| Repo + type     | one entity per repo *plus* one per public class, related via FK      | hits directly; location is the `.cs` file. | high — requires parsing `*.cs` at ingest, sized alias table, type-rename churn. |

**Decision: repo-level only for now.** Once the agent has the repo path (issue 02) and the right aliases (this issue), `grep -r PriceStockExportJob` inside that one repo is trivial and zero-maintenance. Type-level granularity is documented as a future option, not built yet.

## Proposed fix

For each `laika-*` entity, ingestion should populate:

**1. `namespace` (derived from canonical name)**
Hyphen → dot, segments title-cased, `laika` prefix mapped to `Relaxdays.Laika`:
- `laika-marketplaces-jobs-pricestock` → `Relaxdays.Laika.Marketplaces.Jobs.PriceStock`
- `laika-companyapis-clients-pricestock` → `Relaxdays.Laika.CompanyApis.Clients.PriceStock`

The `laika` → `Relaxdays.Laika` prefix mapping is a per-source config field in `config/knowledge/sources.example.json`, so other organisations can supply their own.

**2. `package_name = namespace`**
Laika repos publish NuGet packages where the package id equals the .NET root namespace. Set them equal at ingest.

**3. `aliases` (concrete set per Laika repo entity)**

| Alias                                                       | Why                              |
|-------------------------------------------------------------|----------------------------------|
| `Relaxdays.Laika.Marketplaces.Jobs.PriceStock`              | full .NET / NuGet form           |
| `laika/Marketplaces/Jobs/PriceStock`                        | GitLab path form (per AGENTS.md) |
| `Laika.Marketplaces.Jobs.PriceStock`                        | namespace without `Relaxdays.` prefix (some code uses this) |
| `PriceStock`                                                | bare repo name (redundant with `repo_name` column but cheap and explicit) |

Aliases are case-insensitive at lookup time — implement by storing lowercased in `aliases.alias` and lowercasing the query in `find_primary_entity`.

**4. Idempotent re-ingestion**
`knowledge-cli init` must be safely re-runnable: aliases dedup'd, no orphan rows after a source removal.

## Future option: type-level entities

If `grep -r ClassName <repo-path>` proves too coarse, revisit type-level entities behind an opt-in `--harvest-types` flag on `init`. Cost: parse `*.cs` headers, write one entity per public top-level type, add `relationships(from=type, to=repo, kind='contained-in')`. Alias each type by its bare name and FQN. Out of scope for this issue.

## Acceptance criteria

- After re-running `knowledge-cli init`, the `entities` row for `laika-marketplaces-jobs-pricestock` has populated `namespace = "Relaxdays.Laika.Marketplaces.Jobs.PriceStock"` and `package_name = "Relaxdays.Laika.Marketplaces.Jobs.PriceStock"`.
- The `aliases` table has at least the four rows listed above for that entity.
- All four natural queries (`Relaxdays.Laika.Marketplaces.Jobs.PriceStock`, `laika/Marketplaces/Jobs/PriceStock`, `Laika.Marketplaces.Jobs.PriceStock`, `PriceStock`) resolve to `laika-marketplaces-jobs-pricestock` via `knowledge-cli get`.
- Re-running `init` twice does not duplicate aliases.
- The prefix mapping (`laika` → `Relaxdays.Laika`) is configurable per source, not hard-coded in `ingest.rs`.
- `crates/knowledge-core/tests/import_sources.rs` covers: namespace derivation, alias generation, alias dedup, case-insensitive matching, configurable prefix mapping.

## Related

- [02-get-hides-repo-location.md](02-get-hides-repo-location.md) — without the location being printed, the alias matching has no useful payoff.
- [06-required-explicit-null-input-schema.md](06-required-explicit-null-input-schema.md) — complements this fix from the schema side: even when ingestion can't auto-derive, the input format forces capture authors to acknowledge the fields exist.
