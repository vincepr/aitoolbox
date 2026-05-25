# Issues Implementation Plans

This directory contains ordered implementation plans for the issues in `docs/issues`.

## Recommended execution order

1. `01-cli-help-example-misleads.md` -> `docs/issues/01-cli-help-example-misleads.md`
2. `02-get-hides-repo-location.md` -> `docs/issues/02-get-hides-repo-location.md`
3. `03-no-discovery-subcommand.md` -> `docs/issues/03-no-discovery-subcommand.md`
4. `04-ingestion-under-populates-aliases.md` -> `docs/issues/04-ingestion-under-populates-aliases.md`
5. `06-required-explicit-null-input-schema.md` -> `docs/issues/06-required-explicit-null-input-schema.md`
6. `05-skill-doesnt-teach-naming.md` -> `docs/issues/05-skill-doesnt-teach-naming.md`

## Why this order

- `01` removes a misleading example immediately.
- `02` unlocks immediate practical value from existing data by exposing source locations.
- `03` adds discovery so exact lookup is no longer brittle.
- `04` makes natural queries resolvable by populating namespace/package/alias data.
- `06` enforces strict input contracts so missing fields are explicit and observable.
- `05` is sequenced last so skill docs describe shipped behavior (`list`, location output, and canonical/fallback flow) accurately.

## Verification baseline (run after each issue)

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
