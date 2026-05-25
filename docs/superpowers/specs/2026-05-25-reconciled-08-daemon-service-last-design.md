# Reconciled 08: Daemon/Service Migration (Final Step) Design

**Goal:** Add Axum service mode only after core safety and contracts are stable.

## Scope
- Add `knowledge-daemon` crate.
- Shared core service layer usage.
- CLI `--daemon-url` mode.
- Middleware: timeout, tracing, compression.

## Requirements
1. No schema authority in HTTP handlers.
2. Startup must verify schema compatibility before accepting writes.
3. CLI local and daemon paths preserve output semantics.

## Testing
- Endpoint integration tests.
- CLI parity tests local vs daemon.

## Done Criteria
- Daemon migration is production-safe and last in sequence.
