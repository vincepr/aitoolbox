# Knowledge CLI Skills Design (knowledge-update + knowledge-get)

## Summary
Define exactly two compact skills for knowledge-cli integration:

- `knowledge-update`
- `knowledge-get`

No standalone `knowledge-apply` skill. Lessons/issues application is embedded in both skills to minimize skill surface area and token usage.

## Goals
- Keep behavior deterministic and concise.
- Avoid assumptions from stale docs; derive from current workspace state and explicit user input.
- Support first-run bootstrap and ongoing repository drift updates.
- Surface and operationalize lessons/issues without extra workflow overhead.

## Non-Goals
- Broad autonomous crawling beyond configured scope.
- Long narrative output or raw metadata dumps.
- Separate skill for lessons/issues application.

## Skill 1: knowledge-update

### Purpose
Initialize and refresh structured knowledge from folder structure and configured sources, while safely applying actionable lessons/issues.

### Triggers
Use when:
- first-time setup is needed
- repo/folder structure changed
- mappings are stale or incomplete
- user requests refresh/upsert

### Workflow
1. Discover current folder/repo structure from configured roots.
2. Upsert known entities and mappings.
3. Detect missing top-level anchors (domain/system) only when undiscoverable.
4. Prompt user only for missing required anchors or source pointers.
5. Apply lesson/issue actions that are safe and local (for example: missing governance files, stale mapping markers).
6. Create concise issues for unresolved or risky items that need follow-up.

### Output Contract
- `status`: `ok | partial | blocked`
- `changes`: `{ repos_added, repos_updated, anchors_added, lessons_applied, issues_created }`
- `missing_inputs`: `[ ... ]`
- `next_step`: short string

## Skill 2: knowledge-get

### Purpose
Answer knowledge queries with exact-first retrieval, compact interpretation, and embedded lessons/issues impact.

### Triggers
Use when:
- user asks for entity/repo/domain/system knowledge
- user needs routing to relevant code or ownership context
- user asks for known pitfalls or prior corrective guidance

### Workflow
1. Classify query type.
2. Resolve exact identifiers first.
3. Fallback to fuzzy/semantic retrieval only if exact match is weak.
4. Return compact answer and confidence.
5. Include relevant lessons/issues impact in minimal form.
6. Apply low-risk corrective action when obvious; otherwise create issue when risk remains unresolved.

### Output Contract
- `match`: canonical entity/repo or `none`
- `confidence`: `high | medium | low`
- `answer`: 1-3 concise lines
- `lessons_issues`: up to 2 compact bullets
- `action`: `none | applied | issue_created | needs_human`

## Embedded Apply Rules (Both Skills)
- Never invent domain/system facts.
- Mark unknowns explicitly.
- Prefer safe local updates; escalate uncertain changes as issues.
- Keep all apply output compact and machine-scannable.

## Error Handling
- `blocked` only when required context is unavailable and cannot be inferred safely.
- Report exact missing input (single-line per item).
- No silent fallbacks that change state ambiguously.

## Testing Expectations
- Unit tests for classification, exact/fuzzy fallback, and output schema stability.
- Integration tests for first-run update, incremental update, and issue creation paths.
- Golden tests for concise output shape to prevent token drift.

## Acceptance Criteria
- Only two skills exist (`knowledge-update`, `knowledge-get`).
- Lessons/issues application is demonstrably integrated into both.
- First-run flow asks only for truly missing anchors.
- Outputs remain short, stable, and actionable.
