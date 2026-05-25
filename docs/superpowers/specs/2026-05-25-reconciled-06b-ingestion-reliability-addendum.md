# Reconciled 06b: Ingestion Reliability Addendum

**Context:** Added during spec 6 implementation after comparing with external operational
queue and deterministic pipeline patterns.

## Why this addendum exists
Spec 6 introduces a deterministic raw-first queue, but production-grade ingestion should also include worker-lease and dead-letter semantics.

## Follow-up hardening
1. Add lease ownership fields (`lease_owner`, `lease_expires_at`) to avoid duplicate workers processing the same job.
2. Add max retry ceiling with `dead_letter` terminal state.
3. Add retention policy for succeeded/dead-letter rows to cap table growth.
4. Add per-phase duration metrics (`parse_ms`, `normalize_ms`, `classify_ms`, `persist_ms`) for debugging throughput regressions.

## Reference-informed notes
- Mature queue designs emphasize retry ceilings, dead-lettering, and retention windows.
- Deterministic pipelines benefit from explicit phase observability.
