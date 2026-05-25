---
name: knowledge-update
description: "Refresh local knowledge-cli state with explicit migration/config authority checks, conservative updates, and deterministic compact output."
---

# knowledge-update

Use this skill to initialize or refresh local knowledge mappings safely and deterministically.

## Inputs
- Optional `--db`, `--notes-root`, `--source-file` overrides.
- Optional human anchors only if required and undiscoverable.

## Preconditions
- Prefer local/offline-safe execution.
- Do not require network calls for normal refresh.

## Steps
1. Verify CLI availability:
   - `knowledge-cli version`
   - If missing: return `blocked` with install command.
2. Run migration gate:
   - `bash scripts/migrate-knowledge-db.sh`
   - If migration is required and fails: return `blocked` with exact failure reason.
3. Assert config authority and startup validity:
   - run `knowledge-cli quickstart [overrides...]`
   - respect typed config precedence (CLI overrides > env/config > defaults).
4. Upsert mappings from source:
   - `knowledge-cli init --source-file <path> [--db ...]`
5. Detect stale/missing mappings from current workspace hints.
6. Apply only low-risk local corrections.
7. Record unresolved uncertainty as issue/lesson:
   - `knowledge-cli capture-issue ...`
   - `knowledge-cli capture-lesson ...`

## Embedded Apply Policy
- Never invent domain/system facts.
- Never mutate uncertain mappings without strong evidence.
- Prefer issue creation over speculative correction.

## Output (strict)
- `status`: `ok|partial|blocked`
- `migration`: `not_needed|applied|failed|unknown`
- `config_authority`: `ok|warning|failed`
- `changes`: `repos_added,repos_updated,anchors_added,lessons_applied,issues_created`
- `missing_inputs`: short list
- `next_step`: one line

## Stop Conditions
Return `blocked` when:
- `knowledge-cli` is unavailable and cannot be installed in-session
- migration required but unsuccessful
- mandatory source input is unavailable
