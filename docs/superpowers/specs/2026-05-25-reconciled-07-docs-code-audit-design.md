# Reconciled 07: Docs-to-Code Audit Workflow Design

**Goal:** Add a lightweight repeatable process to keep docs and todos aligned with behavior.

## Scope
- `docs/audit-workflow.md`.
- Checkpoint metadata.
- Drift detection command/script.

## Requirements
1. Audit status must be explicit and reviewable in PRs.
2. CI should enforce checkpoint updates on architecture doc changes.

## Testing
- Script behavior tests for first-audit/skip/re-audit paths.

## Done Criteria
- Single command reports audit scope reliably.
