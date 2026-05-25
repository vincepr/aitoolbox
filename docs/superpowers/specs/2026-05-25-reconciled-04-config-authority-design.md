# Reconciled 04: Config Authority and Guardrails Design

**Goal:** Establish deterministic typed configuration with future daemon-safe authority boundaries.

## Scope
- File+env+CLI precedence.
- `server_authoritative` vs `client_request_defaults` split.
- Startup validation and typed errors.

## Requirements
1. Invalid config prevents mutations.
2. Effective config resolution is deterministic.

## Testing
- Unit tests for precedence.
- Integration tests for invalid config rejection.

## Done Criteria
- Runtime behavior can be tuned without code changes.
