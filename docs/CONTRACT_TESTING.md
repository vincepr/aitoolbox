# Contract Testing

Versioned contract fixtures live in `crates/knowledge-core/tests/fixtures/contracts/`.

Rules:
- Increment fixture version for intentional breaking changes.
- Keep tests focused on machine-relevant output fields.
- Update CLI and SDK tests to consume the same fixture version.
