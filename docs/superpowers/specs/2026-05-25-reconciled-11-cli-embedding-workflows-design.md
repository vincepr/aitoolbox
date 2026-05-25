# Reconciled 11: CLI Embedding Workflows and Hybrid Recall Design

**Goal:** Expose embedding index/update and hybrid retrieval through `knowledge-cli` commands with no daemon requirement.

## Scope
- Add CLI commands for embedding lifecycle (for example: `embed-index`, `embed-status`, `embed-clear`).
- Add `get`/`recall` hybrid mode using deterministic + semantic ranking.
- Add config flags for provider/model/runtime overrides.
- Add explicit fallback behavior when provider/index is unavailable.

## Requirements
1. Default CLI behavior stays deterministic exact-first.
2. Hybrid mode is explicit and bounded (`top_k`, score thresholds, deterministic tie-breaks).
3. Embedding operations support batched execution and resume-safe progress.
4. Provider and index failures are visible and do not silently degrade correctness.
5. Output contracts include provenance/source fields that distinguish exact vs semantic hits.

## Testing
- CLI contract tests for embedding commands.
- Hybrid ranking tests using frozen fixtures.
- Failure-path tests (provider unavailable, stale vectors, timeout) with expected fallback output.

## Done Criteria
- Users can build/use embeddings fully from CLI.
- Hybrid retrieval is optional, explicit, and test-verified.
- No daemon process is required for embedding workflows.
