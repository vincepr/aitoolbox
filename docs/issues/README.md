# Issues

Engineering issues for `knowledge-cli`, `knowledge-core`, and the `knowledge-get` skill. Each file is self-contained: symptom, root cause with file/line citations, proposed fix, acceptance criteria.

Status legend:
- `[planned]` not started
- `[in-progress]` actively being worked
- `[done]` merged

## Theme: "skill recommends a path the CLI cannot deliver"

The `knowledge-get` skill is meant to be the first step before agents decompile NuGet DLLs or grep `~/.nuget`. In practice, the skill misses on natural queries (`Relaxdays.Laika.Marketplaces.Jobs.PriceStock`, `PriceStockExportJob`) and even when it hits, the CLI hides the most useful field (the local clone path / git URL). Agents fall through to decompilation. These issues, taken together, close that gap.

| # | File | Status | Effort | Impact |
|---|------|--------|--------|--------|
| 1 | [01-cli-help-example-misleads.md](01-cli-help-example-misleads.md) | `[planned]` | trivial | Stops agents trying the wrong name format |
| 2 | [02-get-hides-repo-location.md](02-get-hides-repo-location.md) | `[planned]` | small | When entity matches, agent reads real source instead of decompiling |
| 3 | [03-no-discovery-subcommand.md](03-no-discovery-subcommand.md) | `[planned]` | medium | Lookup works without prior knowledge of canonical names |
| 4 | [04-ingestion-under-populates-aliases.md](04-ingestion-under-populates-aliases.md) | `[planned]` | medium | Natural queries (full .NET namespace, class names) start matching |
| 5 | [05-skill-doesnt-teach-naming.md](05-skill-doesnt-teach-naming.md) | `[planned]` | trivial | Agents stop blindly trying dot-PascalCase names |

Issues 1 and 5 unblock agents using the tool as-is. Issues 2, 3, 4 are the durable fixes.
