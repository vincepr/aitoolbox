# Reconciled 06: Background Ingestion Pipeline Design

**Goal:** Introduce raw-first asynchronous ingestion with safe retry/dedupe semantics.

## Scope
- Add job/result tables and states.
- Worker phases: parse->normalize->classify->persist.
- Config-gated startup and optional provider hooks off by default.

## Requirements
1. Persist job before processing.
2. Retries remain idempotent.
3. Deduplication prevents duplicate domain writes.

## Testing
- Success/transient/permanent failure tests.

## Done Criteria
- Failed jobs can be retried safely.
