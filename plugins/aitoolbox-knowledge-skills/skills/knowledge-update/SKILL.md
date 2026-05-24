---
name: knowledge-update
description: "Upsert local knowledge-cli state from repo structure/config and apply concise lesson/issue actions during refresh."
---

# knowledge-update

Use this skill to initialize or refresh the local knowledge store with minimal output and safe corrective actions.

## Inputs
- Optional `--db`, `--notes-root`, `--source-file` overrides.
- Optional human anchors only when missing and undiscoverable: top-level `domain`/`system` context or pointer to source JSON.

## Steps
1. Ensure baseline files exist:
   - `knowledge-cli quickstart [--db ... --notes-root ... --source-file ...]`
2. Upsert entity mappings from source:
   - `knowledge-cli init --source-file <path> [--db ...]`
3. Detect missing or stale mappings from current folder/repo structure.
4. Apply low-risk corrections directly (safe local updates only).
5. If uncertainty remains, create concise tracked records:
   - `knowledge-cli capture-issue --slug <slug> --body <short-body> [--db ... --notes-root ...]`
   - `knowledge-cli capture-lesson --slug <slug> --body <short-body> [--db ... --notes-root ...]`

## Embedded Apply Policy
- Never invent domain/system facts.
- If evidence is weak: record issue instead of mutating mappings.
- Keep lesson/issue bodies short and action-oriented.

## Output (keep compact)
- `status`: `ok|partial|blocked`
- `changes`: `repos_added,repos_updated,anchors_added,lessons_applied,issues_created`
- `missing_inputs`: short list
- `next_step`: one line
