# 05 — `knowledge-get` SKILL.md does not teach the canonical-name convention or the fallback flow

Status: `[planned]`
Effort: trivial (SKILL.md edit)
Area: `skills/core/knowledge-get`, `plugins/aitoolbox-knowledge-skills/skills/knowledge-get`

## Symptom

The skill is intended to be the first tool an agent reaches for when looking up `Relaxdays.*` source. In practice an agent invokes it and immediately fails because:

1. The SKILL.md instructs `knowledge-cli get <entity>` with no example showing what `<entity>` looks like.
2. The agent guesses dot-PascalCase (`Relaxdays.Laika.Marketplaces.Jobs.PriceStock`), gets `No exact entity match`, tries variants, then gives up and decompiles.
3. The SKILL.md's "bounded fallback" step (`nearest canonical candidate(s) only`) has no concrete recipe — no example of how to actually produce a candidate when exact misses.

## Root cause

`skills/core/knowledge-get/SKILL.md` lines 14-28:

```
2. Run exact-first lookup:
   - knowledge-cli get <entity> [--db ... --notes-root ...]
3. If exact is weak, run bounded fallback:
   - nearest canonical candidate(s) only
   - no broad scans unless user asks
```

No example, no naming convention, no concrete fallback command. The CLI side reinforces the gap (see [01](01-cli-help-example-misleads.md)).

## Proposed fix

Replace step 2-3 with concrete worked examples and a naming-convention note:

```markdown
2. Run exact-first lookup. Canonical names are kebab-lowercase-dash, e.g.
   `laika-marketplaces-jobs-pricestock` (NOT `Relaxdays.Laika.Marketplaces.Jobs.PriceStock`).
   - knowledge-cli get laika-marketplaces-jobs-pricestock

3. If exact misses, run bounded fallback before giving up:
   - knowledge-cli list --grep <one or two relevant words>
     (issue #03 — `list` subcommand to add)
   - Re-run `get` against any plausible canonical name from the result.
   - At most one fallback round; do not iterate.

4. Read the `local:` path printed by `get` (issue #02) and inspect the
   real source. Do NOT fall back to decompilation, `~/.nuget` scanning, or
   `ilspycmd` while a local path is available.
```

Once [01](01-cli-help-example-misleads.md), [02](02-get-hides-repo-location.md) and [03](03-no-discovery-subcommand.md) ship, this SKILL.md update locks in the contract from the agent side.

## Acceptance criteria

- SKILL.md shows at least one full worked example end-to-end (`get` succeeds → location printed → source read).
- SKILL.md explicitly states the canonical-name format and that dot-PascalCase will not match (until [04](04-ingestion-under-populates-aliases.md) is in).
- SKILL.md tells the agent the explicit "do not decompile while a local path is available" rule, mirroring the rule in user CLAUDE.md files.

## Related

- [01-cli-help-example-misleads.md](01-cli-help-example-misleads.md) — the matching CLI-side fix.
- [02-get-hides-repo-location.md](02-get-hides-repo-location.md) — without the `local:` line being printed, the skill's "read the path" step has nothing to read.
- [03-no-discovery-subcommand.md](03-no-discovery-subcommand.md) — without `list`, the fallback recipe has nothing to call.
