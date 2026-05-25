---
name: knowledge-get
description: "Resolve knowledge questions with exact-first retrieval, ranked context matches, and explicit confidence/provenance reporting."
---

# knowledge-get

Use this skill to answer repo/domain/system/entity questions with compact, confidence-scored output.

## Inputs
- Query text or exact identifier.
- Optional `--db` and `--notes-root` overrides.

## Steps
1. Classify query intent:
   - entity resolution
   - boundary/context recall
   - lesson/issue recall
2. Run direct lookup first:
   - `knowledge-cli get <entity> [--db ... --notes-root ... --limit <N> --related-limit <N>]`
   - `get` supports direct normalized variants (`.`, `-`, `_`, `/`, case-insensitive).
3. Use ranked matches from `get` output:
   - `get` always prints `Top matches:` (default 3 ordered rows).
   - increase candidates with `--limit <N>` when broader context is needed.
   - use `list --grep` only when `get` returns no plausible candidates.
4. For parent hits (`domain`/`system`), use `Related` first:
   - `get` prints `Related` rows (`id`, `canonical_name`, `kind`, note marker).
   - increase with `--related-limit <N>` when parent has many children.
   - use a related child canonical name for the next `get` call.
5. Build compact answer from minimal relevant context.
6. Apply lessons/issues inline:
   - if corrective lesson applies, include one concise line
   - if unresolved/high-impact gap remains, create issue
7. If `get` returns `local:` or `git:`, inspect source before speculation.
8. If behavior-level uncertainty remains, recommend source inspection explicitly.

## Output (strict)
- `match`: canonical entity/repo or `none`
- `confidence`: `high|medium|low`
- `provenance`: `exact|alias|semantic|none`
- `answer`: 1-3 lines
- `lessons_issues`: up to 2 bullets
- `action`: `none|applied|issue_created|needs_human`
- `requires_source_inspection`: `yes|no`

## Embedded Apply Policy
- Prefer correction over speculation.
- Do not claim semantic certainty when only partial mappings exist.
- Escalate uncertain/high-impact gaps as issues.
- Prefer local source inspection over decompilation when a local path is available.

## Stop Conditions
- Return `match: none` + low confidence when no reliable mapping exists.
- Set `requires_source_inspection: yes` when knowledge coverage is insufficient for
  implementation-level claims.
