---
name: knowledge-get
description: "Resolve knowledge questions with exact-first retrieval, bounded fallback, and explicit confidence/provenance reporting."
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
   - `knowledge-cli get <entity> [--db ... --notes-root ...]`
   - `get` supports direct normalized variants (`.`, `-`, `_`, `/`, case-insensitive).
3. If no match, run bounded fallback once:
   - `knowledge-cli list --grep <token> --limit 20`
   - retry one plausible `get` candidate
   - no broad scans unless user asks
4. Build compact answer from minimal relevant context.
5. Apply lessons/issues inline:
   - if corrective lesson applies, include one concise line
   - if unresolved/high-impact gap remains, create issue
6. If `get` returns `local:` or `git:`, inspect source before speculation.
7. If behavior-level uncertainty remains, recommend source inspection explicitly.

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
