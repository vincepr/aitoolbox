# Reconciled 02: Provenance, Audit, and Idempotency Design
**DONE:** `2026-05-25-reconciled-02-provenance-audit-design.md`

**Goal:** Record immutable lineage for all writes and prevent duplicate writes under retries.

## Scope
- Add `mutation_events` and `source_evidence`.
- Add idempotency keys for import/capture paths.
- Add `knowledge-cli history <entity>`.

## Requirements
1. Every successful write emits at least one immutable audit event.
2. Retried idempotent calls do not duplicate rows.
3. Rollbacks leave no partial rows.

## Testing
- Write-path tests covering init/import/capture/history.
- Transaction rollback tests.

## Done Criteria
- Audit and domain state remain consistent under retries and failures.
