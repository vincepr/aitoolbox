# 01 — CLI help example uses a name format that does not exist in the index

Status: `[planned]`
Effort: trivial (one-line change)
Area: `crates/knowledge-cli`

## Symptom

`knowledge-cli --help` and `knowledge-cli get --help` both advertise:

```
knowledge-cli get MyCompanyName.Ebay.Custom.Client
```

Following the example, an agent tries dot-separated PascalCase names like `Relaxdays.Laika.Marketplaces.Jobs.PriceStock` — and never gets a match, because the index stores canonical names in kebab-lowercase, e.g. `laika-marketplaces-jobs-pricestock`.

Reproduction:

```console
$ knowledge-cli get Relaxdays.Laika.Marketplaces.Jobs.PriceStock
No exact entity match found for Relaxdays.Laika.Marketplaces.Jobs.PriceStock

$ knowledge-cli get laika-marketplaces-jobs-pricestock
laika-marketplaces-jobs-pricestock
No note summary stored
```

## Root cause

`crates/knowledge-cli/src/main.rs:89`

```rust
help = "Canonical entity name for exact lookup (for example MyCompanyName.Ebay.Custom.Client)"
```

The example was chosen as a generic placeholder. It does not match any naming convention used by the ingestion pipeline.

The same string is reproduced in the after-help examples block (`knowledge-cli --help` output).

## Proposed fix

Replace the placeholder with a name shape that matches what `init`/`quickstart` actually produces (kebab-lowercase, dash-separated):

```rust
help = "Canonical entity name for exact lookup (for example laika-marketplaces-jobs-pricestock)"
```

Also update the after-help examples in the same file so:

```
knowledge-cli get MyCompanyName.Ebay.Custom.Client
```

becomes something like:

```
knowledge-cli get laika-marketplaces-jobs-pricestock
```

## Acceptance criteria

- `knowledge-cli get --help` shows an example whose format matches the rows produced by the default ingestion sources.
- `knowledge-cli --help` after-help examples match the same format.
- A help-output snapshot test (under `crates/knowledge-cli/tests/help.rs`) is updated to lock the corrected example.

## Related

- [05-skill-doesnt-teach-naming.md](05-skill-doesnt-teach-naming.md) — the skill SKILL.md repeats this gap on the agent-facing side.
