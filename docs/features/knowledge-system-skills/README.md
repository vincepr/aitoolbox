# Knowledge System Skills

## Status
- `Implemented (v1)`
- Current canonical skills:
  - `knowledge-update`
  - `knowledge-get`
- Design source: `docs/superpowers/specs/2026-05-24-knowledge-cli-skills-design.md`

## Summary
This feature defines the behavior layer on top of `knowledge-cli` so agents query and refresh knowledge with minimal context and deterministic outputs.

## Implemented Scope
- Two-skill model (`knowledge-update`, `knowledge-get`)
- Embedded lesson/issue actions in both skills (no standalone apply skill)
- Exact-first retrieval posture
- Concise, structured output contracts

## Skill Contracts

### `knowledge-update`
Purpose:
- initialize/refresh mappings and notes with conservative, safe updates

Includes:
- version check + upgrade guidance
- major-version migration gate via `scripts/migrate-knowledge-db.sh`
- bootstrap/update via `knowledge-cli quickstart` and `knowledge-cli init`
- concise issue/lesson capture when uncertain data is detected

### `knowledge-get`
Purpose:
- answer entity/repo/domain/system questions with compact confidence-scored output

Includes:
- exact-first lookup via `knowledge-cli get`
- compact answer + confidence + next action
- short lessons/issues impact note when relevant

## Notes
- Older naming (`knowledge-query`, `knowledge-init-update`, `knowledge-capture`) is superseded.
- Keep this doc synchronized with `skills/core/*` and `plugins/aitoolbox-knowledge-skills/skills/*`.
