# Spec 08 Draft TODOs

Status: `Draft / incomplete`
Branch: `feat/spec-08-daemon-main`

This branch is intentionally paused and opened as draft.

## Implemented in this draft
- Added workspace wiring for a new `knowledge-daemon` crate.
- Added `crates/knowledge-daemon/Cargo.toml`.
- Added daemon entrypoint skeleton (`src/main.rs`) with startup schema verification.
- Added HTTP router skeleton (`src/http.rs`) with:
  - `/health`
  - `/entity/:name`
  - middleware: timeout, tracing, compression.

## TODOs before Spec 08 can be considered complete
- [ ] Add daemon endpoint integration tests in `crates/knowledge-daemon/tests/endpoints.rs`.
- [ ] Add CLI daemon parity tests in `crates/knowledge-cli/tests/daemon_parity.rs`.
- [ ] Wire `knowledge-cli --daemon-url` mode and preserve local output semantics.
- [ ] Ensure handlers do not own schema authority (startup gate only, no handler-side migration/bootstrap).
- [ ] Add explicit daemon error response contract and parity assertions.
- [ ] Verify middleware behavior (timeouts/compression/tracing) with tests.
- [ ] Run full verification gate:
  - `cargo fmt --check`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test --workspace`
  - `cargo build --workspace`

## Notes
- This checkpoint is meant to preserve progress while stopping implementation work.
- No claim of Spec 08 completion is made by this draft branch.
