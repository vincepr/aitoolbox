# Reconciled 03: Unified Contract Fixtures for CLI and SDK Design

**Goal:** Prevent output/interface drift by introducing one shared fixture/version source.

## Scope
- Define versioned DTO fixture files.
- Use fixtures in CLI and SDK contract tests.
- Add `CONTRACT_TESTING` policy to AGENTS.md (and comment in code in suitable place(s))

## Requirements
1. Fixture schema must be explicit and versioned.
2. Contract tests fail on breaking changes unless version is bumped.

## Testing
- Contract tests for help/validation/output semantics.

## Done Criteria
- One fixture set powers all contract tests.
