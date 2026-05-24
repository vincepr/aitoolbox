# Rust Codebase Remediation Plan

## Summary
Bring the Rust codebase into full compliance with `AGENTS.md` quality rules without changing behavior.

Primary gaps to remediate are:
- missing Rust doc comments on public APIs in `knowledge-core`
- inconsistent runtime error-reporting style in `knowledge-cli` output paths

Success criteria:
- all public API items in `crates/knowledge-core/src` are documented with Rustdoc
- CLI error/failure output policy is explicit and consistently implemented
- `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` remain green

## Implementation Changes

### 1. Public API Documentation (knowledge-core)
- Add doc comments for every public item in `crates/knowledge-core/src`:
  - modules exposed via `lib.rs`
  - public structs/enums (including field docs where externally relevant)
  - public functions and methods
- For each public function, include:
  - purpose/behavior
  - arguments
  - return value
  - error conditions
- Add short usage examples only for non-obvious functions (`apply_source_json`, `first_paragraph`, path-validation helpers).
- Keep docs concise and behavior-accurate; no aspirational or stale wording.

### 2. CLI Output/Error Policy (knowledge-cli)
- Define command-output contract:
  - success data stays on stdout
  - user-correctable errors and operational failures go through structured error flow
- For “no match found” in `get/query`:
  - decide and document whether it is a valid empty result (exit 0) or an error (non-zero)
  - implement consistently in command handler
- Replace ad-hoc failure-style messaging with centralized contextual errors (`anyhow::Context`) where command execution can fail.
- Keep human-readable terminal UX, but ensure internal failures are surfaced with actionable context.

### 3. Consistency and Hygiene
- Ensure no `unwrap()`/`expect()` is introduced in non-test runtime/library paths.
- Confirm formatting and line-length consistency through rustfmt.
- Ensure comments/docs do not contradict actual defaults and CLI alias behavior.

## Test Plan
- Existing checks (must pass):
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo fmt --check`
- Add/adjust tests as needed for CLI behavior contract:
  - `get/query` no-match exit code + message behavior
  - error-path assertions for malformed `--source-json` / `--input-json` with clear context
- Optional quality gate:
  - run `cargo doc --no-deps` to validate Rustdoc generation and catch doc issues early

## Deliverables
- New document: `docs/remediation-plan.md` containing this finalized plan and acceptance checklist
- Code updates in:
  - `crates/knowledge-core/src/*` (Rustdoc coverage)
  - `crates/knowledge-cli/src/main.rs` (error/output policy alignment)
  - `crates/knowledge-cli/tests/*` (behavior assertions)
- Final verification summary captured in MR description

## Assumptions and Defaults
- Scope is Rust code only (`crates/knowledge-core`, `crates/knowledge-cli`), excluding docs/plan archives in `docs/superpowers/plans/*`.
- Test-only `unwrap()` remains acceptable unless you later choose stricter test hygiene.
- Preferred default for `get/query` no-match is **non-error informational output with exit code 0** unless product direction changes.

## Acceptance Checklist
- [ ] Rustdoc coverage added to all public `knowledge-core` APIs
- [ ] `knowledge-cli` uses contextual error propagation for operational failures
- [ ] `get` no-match behavior is explicitly documented and tested as exit code 0
- [ ] Malformed JSON errors include actionable context and are tested
- [ ] `cargo fmt --check` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
