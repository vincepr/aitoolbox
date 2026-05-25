# Reconciled 10: Vector Storage and Indexing (SQLite) Design

**Goal:** Introduce local vector persistence and retrieval primitives in SQLite while keeping exact lookup first and semantic recall optional.

## Scope
- Add schema tables for embeddings (document/entity vectors, metadata, provider/model/version hash).
- Add idempotent upsert/update flow for embeddings.
- Add retrieval query API for top-k semantic candidates.
- Keep vector path optional and off by default.

## Requirements
1. Vector schema is migration-controlled and backward-compatible.
2. Embedding rows are versioned by provider/model/config fingerprint.
3. Re-indexing is incremental and idempotent.
4. Vector retrieval API returns scored candidates with stable tie-breaks.
5. Missing vector index does not block deterministic exact lookup.

## Testing
- Migration tests for new vector schema.
- Upsert idempotency tests.
- Retrieval determinism tests with fixed test vectors.
- Failure-path tests for stale/missing vectors.

## Done Criteria
- SQLite vector storage exists with safe migration path.
- Semantic candidates can be retrieved locally from CLI mode.
- Deterministic lookup remains available and unaffected.
