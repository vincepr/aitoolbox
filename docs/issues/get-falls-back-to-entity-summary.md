# `knowledge-cli get` ignores `entities.summary` when no `note_refs` row exists

## Problem

`knowledge-cli get <entity>` resolves an exact match but reports `No note summary stored` whenever the entity has no row in `note_refs` — even when the `entities.summary` column is populated with usable content. The data is in the DB; the renderer just never looks at it.

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

But the row is not empty:

```console
$ sqlite3 "$KNOWLEDGE_CLI_DB" \
    "SELECT length(summary), substr(summary,1,80) FROM entities WHERE canonical_name='marketplaces';"
382|Relaxdays Marketplaces-Integrationen (Amazon, eBay, Kaufland, Otto, ManoMano, Bo
```

And `note_refs` has no row for `entity_id = 1` (the `marketplaces` domain entity), so the note-based path returns nothing.

## Why it matters

- Anchor-level entities (`kind = domain`) are exactly the entries most likely to have a hand-curated `summary` but no dedicated note file — they aggregate downstream notes rather than carry their own.
- Callers reading the CLI output reasonably conclude "this entry is empty" and either ignore it or try to re-populate it, when in fact the summary is already there.
- The third column in the `Top matches` table is `repo_name` (e.g. `Catalog` for `laika-marketplaces-catalog`), not `summary`, so even the ranked fallback doesn't expose what's stored.

## Proposed fix

When `get` resolves an exact match:

1. If `note_refs` has a row → render the linked note as today.
2. Else if `entities.summary` is non-empty → render it (clearly marked as the column summary, not a note).
3. Else → emit the current `No note summary stored` message.

Optional: add the `summary` column (truncated) to the `Top matches` table when `repo_name` is empty, so domain-level hits aren't blank.

## Out of scope

- Backfilling `note_refs` for existing domain entities. That's a separate data question (see `get-returns-children-on-parent-match.md` for the related navigation gap).
