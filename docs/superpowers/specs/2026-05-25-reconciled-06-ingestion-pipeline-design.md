# Reconciled 06: Background Ingestion Pipeline Design

**Goal:** Introduce raw-first asynchronous ingestion with safe retry/dedupe semantics.

## Scope
- Add job/result tables and states.
- Worker phases: parse->normalize->classify->persist.
- Config-gated startup and optional provider hooks off by default.
- Provider hooks MUST be interface-based (for example embeddings/classifiers) so providers are
  replaceable and usable from CLI flows without daemon dependency.

## Requirements
1. Persist job before processing.
2. Retries remain idempotent.
3. Deduplication prevents duplicate domain writes.
4. Provider failures are isolated from ingestion job state transitions and recorded as structured
   failures with retry policy control.
5. Provider configuration is explicit and typed (provider kind, runtime, model, batch size,
   timeout), with safe defaults that keep providers disabled by default.

## Testing
- Success/transient/permanent failure tests.
- Provider-off, provider-on-success, and provider-on-failure tests.

## Done Criteria
- Failed jobs can be retried safely.
- Provider behavior is pluggable and does not force service/daemon mode.
