# `knowledge-cli get` should return related child entities when a parent matches

## Problem

When `get` exact-matches a parent-level entity (typically `kind = domain` or `kind = system`), the output gives the caller no way to discover the children that actually carry the detail. The caller has to guess at canonical names or fall back to `list` with no parent context.

## Repro

```console
$ knowledge-cli get marketplaces
marketplaces
No note summary stored

Top matches:
marketplaces            domain
laika-marketplaces      system
laika-marketplaces-catalog  library Catalog
```

The `Top matches` block is a generic ranked-similarity list. It is not a "children of the matched parent" list. For `marketplaces` the actually-useful follow-ups are the eight lesson entities that live under it:

```console
$ sqlite3 "$KNOWLEDGE_CLI_DB" \
    "SELECT e.canonical_name, e.kind, n.note_path
       FROM entities e JOIN note_refs n ON n.entity_id = e.id
      WHERE e.canonical_name LIKE 'marketplaces-%';"
marketplaces-overview      lesson  lesson/marketplaces-overview.md
marketplaces-sql-rules     lesson  lesson/marketplaces-sql-rules.md
marketplaces-fallstricke   lesson  lesson/marketplaces-fallstricke.md
marketplaces-cross-db      lesson  lesson/marketplaces-cross-db.md
marketplaces-zap           lesson  lesson/marketplaces-zap.md
marketplaces-zot           lesson  lesson/marketplaces-zot.md
marketplaces-marc          lesson  lesson/marketplaces-marc.md
marketplaces-identifiers   lesson  lesson/marketplaces-identifiers.md
```

These are exactly the next entries the caller would want to query — but `get marketplaces` does not surface them.

## Proposed behaviour

When `get` resolves an exact match on a parent-shaped entity, also include a `Related` block listing child entities by `id` and `canonical_name`, bounded and ordered so the caller can pick the next query themselves.

### Candidate sources for "child of"

In rough preference order (use whichever the schema supports cleanly):

1. Rows in `relationships` where `to_entity_id` is the matched parent (currently empty for `marketplaces` — would need population, but is the right long-term answer).
2. Entities whose `canonical_name` is a prefix-extension of the parent (`marketplaces-*`, `laika-marketplaces-*`). Cheap heuristic, works today without schema changes.
3. Entities sharing the parent's `namespace` / `package_name` root (`Relaxdays.Laika.Marketplaces.*`).

### Cutoff and ordering

- Cap at e.g. `--related-limit` (default ~10) so domain hits don't dump 50+ rows.
- Order by, in this priority:
  1. Entities with `notes_state = 'known'` first (they have something concrete to read).
  2. Then by `kind` rank (`lesson` > `library` > `system`), so the highest-signal entries float up.
  3. Then alphabetical for stability.
- Print `id` alongside `canonical_name` so callers can pass it back unambiguously.

### Example desired output

```
marketplaces                                            domain
Summary: Relaxdays Marketplaces-Integrationen (Amazon, …)

Related (8 of 28, ordered by notes_state, kind, name):
  8   marketplaces-overview     lesson   has_note
  9   marketplaces-sql-rules    lesson   has_note
 10   marketplaces-fallstricke  lesson   has_note
 11   marketplaces-cross-db     lesson   has_note
 …
```

The caller can then pick `marketplaces-overview` or `marketplaces-sql-rules` as the next `get`, without guessing.

## Why it matters

- Domains are entry points. A discovery flow that dead-ends at the parent forces every caller (human or agent) to fall back to `list` + grep, which is what the canonical-name lookup is supposed to replace.
- The information is already in the DB; this is purely a query-and-render gap.

## Related

- See `get-falls-back-to-entity-summary.md` — the parent's own `summary` column is also currently hidden, which compounds this issue: a `get` on a parent today returns essentially nothing useful, when in fact both the summary and the children are available locally.
