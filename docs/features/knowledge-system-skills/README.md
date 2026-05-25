# Knowledge System Skills

## Status
- `Implemented (v2)`
- Current canonical skills:
  - `knowledge-update`
  - `knowledge-get`
  - `knowledge-refresh`
- Design source: `docs/superpowers/specs/2026-05-24-knowledge-cli-skills-design.md`

## Summary
This feature defines the behavior layer on top of `knowledge-cli` so agents query and refresh knowledge with minimal context and deterministic outputs.

## Implemented Scope
- Three-skill model (`knowledge-update`, `knowledge-get`, `knowledge-refresh`)
- Embedded lesson/issue actions in query/update flows
- Explicit periodic governance checks via `knowledge-refresh`
- Exact-first retrieval posture with bounded fallback
- Concise, structured output contracts

## Skill Contracts

### `knowledge-update`
Purpose:
- initialize/refresh mappings and notes with conservative, safe updates

Includes:
- major-version migration gate via `scripts/migrate-knowledge-db.sh`
- config authority and startup validity checks via `knowledge-cli quickstart`
- bootstrap/update via `knowledge-cli init`
- concise issue/lesson capture when uncertain data is detected

### `knowledge-get`
Purpose:
- answer entity/repo/domain/system questions with compact confidence-scored output

Includes:
- exact-first lookup via `knowledge-cli get`
- compact answer + confidence + provenance + next action
- short lessons/issues impact note when relevant

### `knowledge-refresh`
Purpose:
- run periodic governance and drift checks without routine data mutation

Includes:
- migration posture validation
- contract test verification
- docs-to-code audit checks
- skill/docs sync checks with prioritized remediation actions

## Notes
- Older naming (`knowledge-query`, `knowledge-init-update`, `knowledge-capture`) is superseded.
- Keep this doc synchronized with `skills/core/*` and `plugins/aitoolbox-knowledge-skills/skills/*`.
