# Reconciled TODO Roadmap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the reconciled roadmap in safe dependency order, with daemon/service migration last.

**Architecture:** Keep `knowledge-core` as the single domain/storage authority and evolve it first (migrations, auditability, contracts, config, retrieval, ingestion). Add daemon mode only after these invariants are proven by tests. Preserve CLI semantics while introducing new surfaces incrementally.

**Tech Stack:** Rust, `rusqlite`, `clap`, `serde`, `serde_json`, `thiserror`, `anyhow`, `tracing`, `axum`, `tokio`, `tower`, `assert_cmd`, `tempfile`

---

## Ordered Specs
1. `docs/superpowers/specs/2026-05-25-reconciled-01-schema-migrations-design.md`
2. `docs/superpowers/specs/2026-05-25-reconciled-02-provenance-audit-design.md`
3. `docs/superpowers/specs/2026-05-25-reconciled-03-unified-contract-fixtures-design.md`
4. `docs/superpowers/specs/2026-05-25-reconciled-04-config-authority-design.md`
5. `docs/superpowers/specs/2026-05-25-reconciled-05-hybrid-recall-eval-design.md`
6. `docs/superpowers/specs/2026-05-25-reconciled-06-ingestion-pipeline-design.md`
7. `docs/superpowers/specs/2026-05-25-reconciled-07-docs-code-audit-design.md`
8. `docs/superpowers/specs/2026-05-25-reconciled-08-daemon-service-last-design.md`
9. `docs/superpowers/specs/2026-05-25-reconciled-09-embedding-provider-abstraction-design.md`
10. `docs/superpowers/specs/2026-05-25-reconciled-10-vector-storage-and-indexing-design.md`
11. `docs/superpowers/specs/2026-05-25-reconciled-11-cli-embedding-workflows-design.md`
12. `docs/superpowers/specs/2026-05-25-reconciled-12-daemon-parity-for-embeddings-design.md`

## File Structure Map
- `crates/knowledge-core/src/schema.rs`: versioned migration engine, schema verify API.
- `crates/knowledge-core/src/migrations.rs` (create): migration definitions and ordering.
- `crates/knowledge-core/src/audit.rs` (create): audit event write/read helpers.
- `crates/knowledge-core/src/config.rs` (create): typed config and precedence resolution.
- `crates/knowledge-core/src/recall.rs` (create): hybrid retrieval scoring.
- `crates/knowledge-core/src/ingest.rs` (create): ingestion job model + worker primitives.
- `crates/knowledge-cli/src/main.rs`: new commands (`migrate`, `history`, `recall`, `eval`, `pipeline-*`), config wiring.
- `crates/knowledge-cli/tests/*`: CLI contracts and behavior checks.
- `crates/knowledge-daemon/*` (create in Task 8): service runtime and endpoints.
- `docs/CONTRACT_TESTING.md` (create), `docs/audit-workflow.md` (create): policy docs.

### Task 1: Implement schema migrations and startup verify gate

**Files:**
- Modify: `crates/knowledge-core/src/schema.rs`
- Create: `crates/knowledge-core/src/migrations.rs`
- Modify: `crates/knowledge-core/src/lib.rs`
- Modify: `crates/knowledge-cli/src/main.rs`
- Test: `crates/knowledge-core/tests/schema_bootstrap.rs`
- Test: `crates/knowledge-core/tests/schema_migrate_verify.rs` (create)

- [x] **Step 1: Write failing migration verify tests**
```rust
#[test]
fn verify_fails_when_db_version_is_behind_latest() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    // Simulate old schema: no migration ledger.
    conn.execute("CREATE TABLE entities (id INTEGER PRIMARY KEY)", []).unwrap();
    let err = knowledge_core::schema::verify_schema(&conn).unwrap_err();
    assert!(err.to_string().contains("schema version mismatch"));
}
```

- [x] **Step 2: Run tests to confirm failure**
Run: `cargo test -p knowledge-core schema_migrate_verify -v`
Expected: FAIL because `verify_schema` is missing.

- [x] **Step 3: Implement minimal migration engine and verify API**
```rust
pub fn verify_schema(conn: &Connection) -> Result<()> {
    let current = current_schema_version(conn)?;
    let latest = latest_migration_version();
    if current != latest {
        anyhow::bail!("schema version mismatch: current={current} latest={latest}");
    }
    Ok(())
}
```

- [x] **Step 4: Re-run targeted tests**
Run: `cargo test -p knowledge-core schema_migrate_verify -v`
Expected: PASS.

- [x] **Step 5: Commit**
```bash
git add crates/knowledge-core/src/schema.rs crates/knowledge-core/src/migrations.rs crates/knowledge-core/src/lib.rs crates/knowledge-core/tests/schema_migrate_verify.rs crates/knowledge-cli/src/main.rs
git commit -m "feat: add versioned schema migration and verify gate"
```

### Task 2: Add provenance, audit tables, and idempotency-safe writes

**Files:**
- Create: `crates/knowledge-core/src/audit.rs`
- Modify: `crates/knowledge-core/src/import.rs`
- Modify: `crates/knowledge-core/src/capture.rs`
- Modify: `crates/knowledge-core/src/store.rs`
- Modify: `crates/knowledge-cli/src/main.rs`
- Test: `crates/knowledge-core/tests/audit_history.rs` (create)

- [x] **Step 1: Write failing audit emission test**
```rust
#[test]
fn capture_lesson_writes_mutation_event() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    knowledge_core::schema::bootstrap(&conn).unwrap();
    let note_store = knowledge_core::notes::NoteStore::new(tempfile::tempdir().unwrap().path().into()).unwrap();
    knowledge_core::capture::capture_lesson(&conn, &note_store, "slug", "body").unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM mutation_events", [], |r| r.get(0)).unwrap();
    assert!(count >= 1);
}
```

- [x] **Step 2: Run test to verify failure**
Run: `cargo test -p knowledge-core audit_history -v`
Expected: FAIL due to missing table or write path.

- [x] **Step 3: Implement audit writes and idempotency checks**
```rust
pub fn record_mutation_event(conn: &Connection, event: &MutationEvent) -> Result<()> {
    conn.execute(
        "INSERT INTO mutation_events (event_id, operation, actor, input_hash, created_at) VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)",
        rusqlite::params![event.event_id, event.operation, event.actor, event.input_hash],
    )?;
    Ok(())
}
```

- [x] **Step 4: Run tests**
Run: `cargo test -p knowledge-core audit_history capture import_sources -v`
Expected: PASS.

- [x] **Step 5: Commit**
```bash
git add crates/knowledge-core/src/audit.rs crates/knowledge-core/src/import.rs crates/knowledge-core/src/capture.rs crates/knowledge-core/src/store.rs crates/knowledge-cli/src/main.rs crates/knowledge-core/tests/audit_history.rs
git commit -m "feat: add mutation provenance and idempotency safeguards"
```

### Task 3: Unify CLI/SDK contract fixtures

**Files:**
- Create: `crates/knowledge-core/tests/fixtures/contracts/v1/*.json`
- Modify: `crates/knowledge-cli/tests/help.rs`
- Modify: `crates/knowledge-cli/tests/query_cli.rs`
- Create: `docs/CONTRACT_TESTING.md`

- [x] **Step 1: Add failing contract fixture test**
```rust
#[test]
fn get_output_matches_contract_v1() {
    let expected: serde_json::Value = serde_json::from_str(include_str!("../../knowledge-core/tests/fixtures/contracts/v1/get_success.json")).unwrap();
    let actual = serde_json::json!({"entity":"Example.Entity","status":"ok"});
    assert_eq!(actual, expected);
}
```

- [x] **Step 2: Run test to confirm fixture mismatch**
Run: `cargo test -p knowledge-cli get_output_matches_contract_v1 -v`
Expected: FAIL until CLI output is aligned.

- [x] **Step 3: Align output shape + fixture schema**
```rust
#[derive(serde::Serialize)]
struct GetResponseV1 {
    entity: String,
    status: String,
    summary: String,
}
```

- [x] **Step 4: Run tests**
Run: `cargo test -p knowledge-cli --tests -v`
Expected: PASS.

- [x] **Step 5: Commit**
```bash
git add crates/knowledge-core/tests/fixtures/contracts/v1 crates/knowledge-cli/tests/help.rs crates/knowledge-cli/tests/query_cli.rs docs/CONTRACT_TESTING.md
git commit -m "test: unify cli contracts with versioned fixtures"
```

### Task 4: Add typed config authority and guardrails

**Files:**
- Create: `crates/knowledge-core/src/config.rs`
- Modify: `crates/knowledge-core/src/lib.rs`
- Modify: `crates/knowledge-cli/src/main.rs`
- Modify: `config/README.md`
- Test: `crates/knowledge-core/tests/config_resolution.rs` (create)

- [x] **Step 1: Write failing precedence test**
```rust
#[test]
fn precedence_is_file_then_env_then_cli() {
    let file_cfg = r#"{\"recall\":{\"top_k\":5}}"#;
    let env_top_k = Some(10_u32);
    let cli_top_k = Some(20_u32);
    let effective = knowledge_core::config::resolve_for_test(file_cfg, env_top_k, cli_top_k).unwrap();
    assert_eq!(effective.recall.top_k, 20);
}
```

- [x] **Step 2: Run test and verify failure**
Run: `cargo test -p knowledge-core config_resolution -v`
Expected: FAIL until resolver exists.

- [x] **Step 3: Implement typed config and validation**
```rust
#[derive(Debug, Clone, serde::Deserialize)]
pub struct EffectiveConfig {
    pub recall: RecallConfig,
}
```

- [x] **Step 4: Run tests**
Run: `cargo test -p knowledge-core config_resolution -v`
Expected: PASS.

- [x] **Step 5: Commit**
```bash
git add crates/knowledge-core/src/config.rs crates/knowledge-core/src/lib.rs crates/knowledge-cli/src/main.rs crates/knowledge-core/tests/config_resolution.rs config/README.md
git commit -m "feat: add typed config precedence and startup validation"
```

### Task 5: Implement hybrid recall and evaluation harness

**Files:**
- Create: `crates/knowledge-core/src/recall.rs`
- Modify: `crates/knowledge-core/src/store.rs`
- Modify: `crates/knowledge-cli/src/main.rs`
- Test: `crates/knowledge-core/tests/recall_ranking.rs` (create)
- Test: `crates/knowledge-cli/tests/eval_cli.rs` (create)

- [ ] **Step 1: Write failing deterministic ranking test**
```rust
#[test]
fn recall_order_is_stable_for_same_input() {
    let first = run_recall("custom client");
    let second = run_recall("custom client");
    assert_eq!(first, second);
}
```

- [ ] **Step 2: Run test to confirm failure**
Run: `cargo test -p knowledge-core recall_ranking -v`
Expected: FAIL until ranking implementation.

- [ ] **Step 3: Implement scoring and tie-breaks**
```rust
fn total_score(s: &ScoreParts) -> i64 {
    s.exact * 1000 + s.alias * 500 + s.fts * 100 + s.graph * 10 - s.name_penalty
}
```

- [ ] **Step 4: Run retrieval and eval tests**
Run: `cargo test -p knowledge-core recall_ranking -v && cargo test -p knowledge-cli eval_cli -v`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/knowledge-core/src/recall.rs crates/knowledge-core/src/store.rs crates/knowledge-cli/src/main.rs crates/knowledge-core/tests/recall_ranking.rs crates/knowledge-cli/tests/eval_cli.rs
git commit -m "feat: add deterministic hybrid recall and eval harness"
```

### Task 6: Add config-validated ingestion pipeline

**Files:**
- Create: `crates/knowledge-core/src/ingest.rs`
- Modify: `crates/knowledge-core/src/schema.rs`
- Modify: `crates/knowledge-cli/src/main.rs`
- Test: `crates/knowledge-core/tests/ingest_pipeline.rs` (create)

- [ ] **Step 1: Write failing raw-first job persistence test**
```rust
#[test]
fn job_is_persisted_before_processing() {
    let result = enqueue_and_run_once("payload");
    assert_eq!(result.initial_state, "queued");
}
```

- [ ] **Step 2: Run test to confirm failure**
Run: `cargo test -p knowledge-core ingest_pipeline -v`
Expected: FAIL until ingest tables/worker exist.

- [ ] **Step 3: Implement worker phases and idempotent retry**
```rust
pub enum IngestPhase { Parse, Normalize, Classify, Persist }
```

- [ ] **Step 4: Run tests**
Run: `cargo test -p knowledge-core ingest_pipeline -v`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/knowledge-core/src/ingest.rs crates/knowledge-core/src/schema.rs crates/knowledge-cli/src/main.rs crates/knowledge-core/tests/ingest_pipeline.rs
git commit -m "feat: add raw-first ingestion pipeline with retry safety"
```

### Task 7: Add docs-to-code audit workflow

**Files:**
- Create: `docs/audit-workflow.md`
- Create: `docs/.audit-state.json`
- Create: `scripts/audit-docs.sh`
- Test: `tests/audit_docs.bats` (create) or shell assertion script

- [x] **Step 1: Write failing script test for first-audit/skip/re-audit states**
```bash
#!/usr/bin/env bash
set -euo pipefail
./scripts/audit-docs.sh --mode test-first | grep -q "first audit"
```

- [x] **Step 2: Run test to verify failure**
Run: `bash tests/audit_docs.sh`
Expected: FAIL until script exists.

- [x] **Step 3: Implement script + policy doc**
```bash
if [[ ! -f docs/.audit-state.json ]]; then
  echo "first audit"
fi
```

- [x] **Step 4: Run test**
Run: `bash tests/audit_docs.sh`
Expected: PASS.

- [x] **Step 5: Commit**
```bash
git add docs/audit-workflow.md docs/.audit-state.json scripts/audit-docs.sh tests/audit_docs.sh
git commit -m "docs: add docs-to-code audit workflow and drift checks"
```

### Task 8: Migrate to daemon/service mode as final step

**Files:**
- Create: `crates/knowledge-daemon/Cargo.toml`
- Create: `crates/knowledge-daemon/src/main.rs`
- Create: `crates/knowledge-daemon/src/http.rs`
- Modify: `Cargo.toml`
- Modify: `crates/knowledge-cli/src/main.rs`
- Test: `crates/knowledge-daemon/tests/endpoints.rs`
- Test: `crates/knowledge-cli/tests/daemon_parity.rs` (create)

- [ ] **Step 1: Write failing daemon health endpoint test**
```rust
#[tokio::test]
async fn health_returns_ok() {
    let app = knowledge_daemon::http::router(test_state());
    let response = tower::ServiceExt::oneshot(app, http::Request::get("/health").body(axum::body::Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), http::StatusCode::OK);
}
```

- [ ] **Step 2: Run tests to verify failure**
Run: `cargo test -p knowledge-daemon endpoints -v`
Expected: FAIL until crate/router exists.

- [ ] **Step 3: Implement daemon endpoints and CLI daemon mode**
```rust
Router::new()
  .route("/health", get(health))
  .route("/entity/:name", get(get_entity))
  .layer(tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(5)))
```

- [ ] **Step 4: Run tests and parity checks**
Run: `cargo test -p knowledge-daemon -v && cargo test -p knowledge-cli daemon_parity -v`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/knowledge-daemon Cargo.toml crates/knowledge-cli/src/main.rs crates/knowledge-daemon/tests/endpoints.rs crates/knowledge-cli/tests/daemon_parity.rs
git commit -m "feat: add axum daemon mode as final migration step"
```

### Task 9: Add embedding provider abstraction in core (CLI-first, replaceable)

**Files:**
- Create: `crates/knowledge-core/src/embed/provider.rs`
- Create: `crates/knowledge-core/src/embed/mod.rs`
- Modify: `crates/knowledge-core/src/config.rs`
- Modify: `crates/knowledge-core/src/lib.rs`
- Test: `crates/knowledge-core/tests/embed_provider.rs` (create)

- [ ] **Step 1: Write failing provider contract test**
```rust
#[test]
fn provider_none_returns_disabled_error() {
    let provider = knowledge_core::embed::provider::provider_none();
    let err = provider.embed_texts(&["hello".to_string()]).unwrap_err();
    assert!(err.to_string().contains("disabled"));
}
```

- [ ] **Step 2: Run test to confirm failure**
Run: `cargo test -p knowledge-core embed_provider -v`
Expected: FAIL until provider interfaces are implemented.

- [ ] **Step 3: Implement trait + typed config selection**
```rust
pub trait EmbeddingProvider: Send + Sync {
    fn embed_texts(&self, input: &[String]) -> Result<Vec<Vec<f32>>, EmbedError>;
}
```

- [ ] **Step 4: Run tests**
Run: `cargo test -p knowledge-core embed_provider config_resolution -v`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/knowledge-core/src/embed crates/knowledge-core/src/config.rs crates/knowledge-core/src/lib.rs crates/knowledge-core/tests/embed_provider.rs
git commit -m "feat: add replaceable embedding provider abstraction in core"
```

### Task 10: Add SQLite vector storage and deterministic semantic candidate API

**Files:**
- Modify: `crates/knowledge-core/src/schema.rs`
- Create: `crates/knowledge-core/src/vector_store.rs`
- Modify: `crates/knowledge-core/src/store.rs`
- Modify: `crates/knowledge-core/src/lib.rs`
- Test: `crates/knowledge-core/tests/vector_store.rs` (create)
- Test: `crates/knowledge-core/tests/schema_migrate_verify.rs`

- [ ] **Step 1: Write failing vector schema migration test**
```rust
#[test]
fn migration_creates_embeddings_table() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    knowledge_core::schema::bootstrap(&conn).unwrap();
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='entity_embeddings'",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 1);
}
```

- [ ] **Step 2: Run tests to confirm failure**
Run: `cargo test -p knowledge-core vector_store schema_migrate_verify -v`
Expected: FAIL until schema/store are added.

- [ ] **Step 3: Implement vector table + idempotent upsert/query API**
```rust
pub fn upsert_embedding(
    conn: &rusqlite::Connection,
    entity_id: i64,
    provider_fingerprint: &str,
    embedding: &[f32],
) -> anyhow::Result<()>;
```

- [ ] **Step 4: Run tests**
Run: `cargo test -p knowledge-core vector_store schema_migrate_verify -v`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/knowledge-core/src/schema.rs crates/knowledge-core/src/vector_store.rs crates/knowledge-core/src/store.rs crates/knowledge-core/src/lib.rs crates/knowledge-core/tests/vector_store.rs crates/knowledge-core/tests/schema_migrate_verify.rs
git commit -m "feat: add sqlite vector storage and semantic candidate primitives"
```

### Task 11: Add CLI embedding lifecycle commands and hybrid retrieval path

**Files:**
- Modify: `crates/knowledge-cli/src/main.rs`
- Modify: `crates/knowledge-core/src/recall.rs`
- Modify: `crates/knowledge-core/src/store.rs`
- Test: `crates/knowledge-cli/tests/embed_cli.rs` (create)
- Test: `crates/knowledge-cli/tests/eval_cli.rs`
- Test: `crates/knowledge-core/tests/recall_ranking.rs`

- [ ] **Step 1: Write failing CLI contract test for embed command**
```rust
#[test]
fn embed_status_reports_disabled_by_default() {
    let mut cmd = assert_cmd::Command::cargo_bin("knowledge-cli").unwrap();
    cmd.args(["embed-status"]).assert().success().stdout(predicates::str::contains("disabled"));
}
```

- [ ] **Step 2: Run tests to confirm failure**
Run: `cargo test -p knowledge-cli embed_cli eval_cli -v`
Expected: FAIL until commands and output contracts are implemented.

- [ ] **Step 3: Implement commands and hybrid mode**
```rust
enum Command {
    EmbedIndex { /* ... */ },
    EmbedStatus { /* ... */ },
    EmbedClear { /* ... */ },
    Recall { /* existing + hybrid flags */ },
}
```

- [ ] **Step 4: Run tests**
Run: `cargo test -p knowledge-cli embed_cli eval_cli -v && cargo test -p knowledge-core recall_ranking -v`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/knowledge-cli/src/main.rs crates/knowledge-core/src/recall.rs crates/knowledge-core/src/store.rs crates/knowledge-cli/tests/embed_cli.rs crates/knowledge-cli/tests/eval_cli.rs crates/knowledge-core/tests/recall_ranking.rs
git commit -m "feat: add cli embedding lifecycle commands and hybrid retrieval"
```

### Task 12: Add daemon parity for embeddings and hybrid retrieval (optional mode)

**Files:**
- Modify: `crates/knowledge-daemon/src/http.rs`
- Modify: `crates/knowledge-daemon/src/main.rs`
- Modify: `crates/knowledge-cli/src/main.rs`
- Test: `crates/knowledge-daemon/tests/endpoints.rs`
- Test: `crates/knowledge-cli/tests/daemon_parity.rs`

- [ ] **Step 1: Write failing parity test for semantic provenance fields**
```rust
#[test]
fn daemon_and_local_recall_have_matching_provenance_shape() {
    // arrange same fixture/config, compare JSON shape for provenance fields
    // local result == daemon result (allowing transport metadata differences)
}
```

- [ ] **Step 2: Run tests to confirm failure**
Run: `cargo test -p knowledge-daemon endpoints -v && cargo test -p knowledge-cli daemon_parity -v`
Expected: FAIL until endpoints/parity paths are implemented.

- [ ] **Step 3: Implement daemon endpoints and CLI parity wiring**
```rust
Router::new()
  .route("/embed/status", get(embed_status))
  .route("/embed/index", post(embed_index))
  .route("/recall", post(recall))
```

- [ ] **Step 4: Run tests**
Run: `cargo test -p knowledge-daemon endpoints -v && cargo test -p knowledge-cli daemon_parity -v`
Expected: PASS.

- [ ] **Step 5: Commit**
```bash
git add crates/knowledge-daemon/src/http.rs crates/knowledge-daemon/src/main.rs crates/knowledge-cli/src/main.rs crates/knowledge-daemon/tests/endpoints.rs crates/knowledge-cli/tests/daemon_parity.rs
git commit -m "feat: add embedding and hybrid retrieval parity for daemon mode"
```

## Final Verification Gate
- [ ] Run: `cargo fmt --check`
- [ ] Run: `cargo clippy --workspace -- -D warnings`
- [ ] Run: `cargo test --workspace`
- [ ] Run: `cargo build --workspace`

Expected: all commands pass with no warnings.
