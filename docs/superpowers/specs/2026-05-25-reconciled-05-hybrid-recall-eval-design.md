# Reconciled 05: Hybrid Recall with Evaluation Harness Design
**DONE:** `2026-05-25-reconciled-05-hybrid-recall-eval-design.md`

**Goal:** Add deterministic ranked recall and measurable retrieval quality gates.

## Scope
- `knowledge-cli recall`.
- Hybrid scoring (exact + alias + FTS + relationship).
- Telemetry table.
- `knowledge-cli eval --dataset`.

## Requirements
1. Ranking must be deterministic with stable tie-breaks.
2. Existing `get` exact behavior must remain unchanged.

## Testing
- Ranking stability tests.
- Eval metric output tests.

## Done Criteria
- CI can detect retrieval regressions from fixed datasets.
