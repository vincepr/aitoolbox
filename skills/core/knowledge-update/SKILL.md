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
2. Print and review input schemas before authoring any payloads:
   - `knowledge-cli init --print-schema`
   - `knowledge-cli capture-lesson --print-schema`
   - `knowledge-cli capture-issue --print-schema`
   - `knowledge-cli pipeline-enqueue --print-schema`
3. Run migration gate:
   - `bash scripts/migrate-knowledge-db.sh`
   - If migration is required and fails: return `blocked` with exact failure reason.
4. Assert config authority and startup validity:
   - run `knowledge-cli quickstart [overrides...]`
   - respect typed config precedence (CLI overrides > env/config > defaults).
5. Upsert mappings from source:
   - `knowledge-cli init --source-file <path> [--db ...]`
6. Detect stale/missing mappings from current workspace hints.
7. Apply only low-risk local corrections.
8. Record unresolved uncertainty as issue/lesson:
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
