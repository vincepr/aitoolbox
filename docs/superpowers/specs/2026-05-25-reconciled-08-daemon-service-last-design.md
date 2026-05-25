# Reconciled 08: Daemon/Service Migration (Final Step) Design

**Goal:** Add Axum service mode only after core safety and contracts are stable.

## Scope
- Add `knowledge-daemon` crate.
- Shared core service layer usage.
- CLI `--daemon-url` mode.
- Middleware: timeout, tracing, compression.
- Reuse core provider interfaces (including embeddings) with no daemon-specific provider logic.

## Requirements
1. No schema authority in HTTP handlers.
2. Startup must verify schema compatibility before accepting writes.
3. CLI local and daemon paths preserve output semantics.
4. Daemon mode remains optional; CLI local mode must provide full deterministic and hybrid recall
   behavior when configured.

## Testing
- Endpoint integration tests.
- CLI parity tests local vs daemon.
- Provider parity tests local vs daemon for identical inputs/config.

## Done Criteria
- Daemon migration is production-safe and last in sequence.
- No core retrieval feature requires daemon-only architecture.
