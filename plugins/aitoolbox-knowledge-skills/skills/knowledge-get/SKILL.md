---
name: knowledge-get
description: "Resolve package/library/namespace/repo knowledge via knowledge-cli using exact-first lookup, compact answers, and embedded lesson/issue actions."
---

# knowledge-get

Use this skill to resolve package, library, namespace, repo, domain, system, and entity questions with concise confidence-scored output.

## When To Use
- Resolve which repo/path owns a package or namespace.
- Map a library identifier to canonical entity context.
- Retrieve prior lessons/issues relevant to a package-level task.

## Inputs
- Query text or exact identifier.
- Optional `--db` and `--notes-root` overrides.

## Steps
1. Run exact-first lookup:
   - `knowledge-cli get <entity> [--db ... --notes-root ...]`
2. If exact lookup is weak, retry with closest canonical candidate(s) only.
3. Return compact answer with confidence.
4. Apply lessons/issues inline:
   - If a known corrective lesson applies, state it in one short line.
   - If unresolved risk/staleness is detected, create a concise issue:
     - `knowledge-cli capture-issue --slug <slug> --body <short-body> [--db ... --notes-root ...]`

## Embedded Apply Policy
- Prefer correction over speculation.
- Do not expand into broad scans unless lookup fails.
- Escalate uncertain/high-impact gaps as issues.

## Output (keep compact)
- `match`: canonical entity/repo or `none`
- `confidence`: `high|medium|low`
- `answer`: 1-3 lines
- `lessons_issues`: up to 2 bullets
- `action`: `none|applied|issue_created|needs_human`
