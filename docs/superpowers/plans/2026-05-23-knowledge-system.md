# Knowledge System Implementation Plan

> **ARCHIVE NOTE:** Historical planning document. Not the current source of truth for shipped behavior.


> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first useful local knowledge system for `aitoolbox`: a Rust CLI plus library that stores structured entity metadata in SQLite, keeps compact notes on disk, supports exact lookup first, and captures lessons and issues without external services.

**Architecture:** Create a small Rust workspace with one reusable library crate for the knowledge model, schema, storage, retrieval, and import logic, plus one CLI crate that exposes `knowledge init`, `knowledge query`, and `knowledge capture` commands. Store noisy retrieval metadata in SQLite and keep agent-facing notes as short Markdown files under `.local/knowledge/notes/`, referenced by database rows so exact lookup and graph expansion stay cheap.

**Tech Stack:** Rust, `clap`, `rusqlite`, `serde`, `serde_json`, `anyhow`, `thiserror`, `tracing`, `tracing-subscriber`, `camino`, `walkdir`, `assert_cmd`, `tempfile`

---

## File Structure

- `Cargo.toml`
  Rust workspace manifest for the new crates.
- `crates/knowledge-core/Cargo.toml`
  Library dependencies and crate metadata.
- `crates/knowledge-core/src/lib.rs`
  Public module exports.
- `crates/knowledge-core/src/model.rs`
  Entity kinds, relationship kinds, input DTOs, and query result types.
- `crates/knowledge-core/src/schema.rs`
  SQLite schema bootstrap and migration entrypoint.
- `crates/knowledge-core/src/store.rs`
  Insert, update, lookup, and graph expansion logic.
- `crates/knowledge-core/src/notes.rs`
  Short note file creation and loading.
- `crates/knowledge-core/src/import.rs`
  Config-driven initialization and conservative refresh logic.
- `crates/knowledge-core/src/capture.rs`
  Lesson and issue capture helpers.
- `crates/knowledge-core/tests/*.rs`
  Focused integration tests for schema, query, import, and capture behavior.
- `crates/knowledge-cli/Cargo.toml`
  CLI dependencies and binary metadata.
- `crates/knowledge-cli/src/main.rs`
  `clap` command definitions and output formatting.
- `config/knowledge/sources.example.json`
  Example source config for repo roots and curated entities.
- `.local/knowledge/notes/.gitkeep`
  Keeps the note directory in git before real notes exist.
- `config/knowledge/templates/domain.md`
  Short authoring template for domain notes.
- `config/knowledge/templates/system.md`
  Short authoring template for system notes.
- `config/knowledge/templates/project.md`
  Short authoring template for project notes.
- `config/knowledge/templates/library.md`
  Short authoring template for library notes.
- `config/knowledge/templates/lesson.md`
  Short authoring template for lesson notes.
- `config/knowledge/templates/issue.md`
  Short authoring template for issue notes.
- `docs/features/knowledge-system/implementation-plan.md`
  Replace the current phase-only outline with a short pointer to this executable plan.
- `README.md`
  Add one short section describing how to run the knowledge CLI locally.

### Task 1: Bootstrap The Rust Workspace And CLI Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `crates/knowledge-cli/Cargo.toml`
- Create: `crates/knowledge-cli/src/main.rs`
- Create: `crates/knowledge-cli/tests/help.rs`

- [ ] **Step 1: Write the failing test**

```rust
use assert_cmd::Command;

#[test]
fn help_lists_knowledge_subcommands() {
    let mut cmd = Command::cargo_bin("knowledge-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("knowledge"))
        .stdout(predicates::str::contains("query"))
        .stdout(predicates::str::contains("init"))
        .stdout(predicates::str::contains("capture"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p knowledge-cli help_lists_knowledge_subcommands -- --exact`
Expected: FAIL with a workspace or package resolution error because the Rust manifests do not exist yet.

- [ ] **Step 3: Write minimal workspace and CLI implementation**

`Cargo.toml`

```toml
[workspace]
members = ["crates/knowledge-core", "crates/knowledge-cli"]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT"
version = "0.1.0"

[workspace.dependencies]
anyhow = "1"
assert_cmd = "2"
camino = "1"
clap = { version = "4", features = ["derive"] }
predicates = "3"
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tempfile = "3"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt"] }
walkdir = "2"
```

`crates/knowledge-cli/Cargo.toml`

```toml
[package]
name = "knowledge-cli"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
anyhow.workspace = true
clap.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

[dev-dependencies]
assert_cmd.workspace = true
predicates.workspace = true
```

`crates/knowledge-cli/src/main.rs`

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "knowledge-cli")]
#[command(about = "Local knowledge system tooling for aitoolbox")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Query,
    Init,
    Capture,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().without_time().init();
    let _ = Cli::parse();
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p knowledge-cli help_lists_knowledge_subcommands -- --exact`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/knowledge-cli/Cargo.toml crates/knowledge-cli/src/main.rs crates/knowledge-cli/tests/help.rs
git commit -m "feat: bootstrap knowledge cli workspace"
```

### Task 2: Add The Core Knowledge Model And SQLite Schema

**Files:**
- Create: `crates/knowledge-core/Cargo.toml`
- Create: `crates/knowledge-core/src/lib.rs`
- Create: `crates/knowledge-core/src/model.rs`
- Create: `crates/knowledge-core/src/schema.rs`
- Create: `crates/knowledge-core/tests/schema_bootstrap.rs`

- [ ] **Step 1: Write the failing test**

```rust
use knowledge_core::schema::bootstrap;
use rusqlite::Connection;

#[test]
fn bootstrap_creates_core_tables() {
    let conn = Connection::open_in_memory().unwrap();

    bootstrap(&conn).unwrap();

    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .unwrap();
    let names = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert!(names.contains(&"entities".to_string()));
    assert!(names.contains(&"aliases".to_string()));
    assert!(names.contains(&"relationships".to_string()));
    assert!(names.contains(&"locations".to_string()));
    assert!(names.contains(&"note_refs".to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p knowledge-core bootstrap_creates_core_tables -- --exact`
Expected: FAIL because `knowledge-core` and `bootstrap` do not exist yet.

- [ ] **Step 3: Write minimal model and schema implementation**

`crates/knowledge-core/Cargo.toml`

```toml
[package]
name = "knowledge-core"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
anyhow.workspace = true
camino.workspace = true
rusqlite.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

`crates/knowledge-core/src/lib.rs`

```rust
pub mod model;
pub mod schema;
```

`crates/knowledge-core/src/model.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityKind {
    Domain,
    System,
    Project,
    Library,
    Tag,
    Lesson,
    Issue,
}

impl EntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EntityKind::Domain => "domain",
            EntityKind::System => "system",
            EntityKind::Project => "project",
            EntityKind::Library => "library",
            EntityKind::Tag => "tag",
            EntityKind::Lesson => "lesson",
            EntityKind::Issue => "issue",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipKind {
    Contains,
    Owns,
    Publishes,
    TaggedAs,
    RelatedTo,
}

impl RelationshipKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RelationshipKind::Contains => "contains",
            RelationshipKind::Owns => "owns",
            RelationshipKind::Publishes => "publishes",
            RelationshipKind::TaggedAs => "tagged_as",
            RelationshipKind::RelatedTo => "related_to",
        }
    }
}
```

`crates/knowledge-core/src/schema.rs`

```rust
use anyhow::Result;
use rusqlite::Connection;

pub fn bootstrap(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS entities (
            id INTEGER PRIMARY KEY,
            canonical_name TEXT NOT NULL UNIQUE,
            kind TEXT NOT NULL,
            summary TEXT NOT NULL DEFAULT '',
            namespace TEXT,
            package_name TEXT,
            repo_name TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS aliases (
            entity_id INTEGER NOT NULL,
            alias TEXT NOT NULL,
            UNIQUE(entity_id, alias),
            FOREIGN KEY(entity_id) REFERENCES entities(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS relationships (
            from_entity_id INTEGER NOT NULL,
            to_entity_id INTEGER NOT NULL,
            kind TEXT NOT NULL,
            UNIQUE(from_entity_id, to_entity_id, kind),
            FOREIGN KEY(from_entity_id) REFERENCES entities(id) ON DELETE CASCADE,
            FOREIGN KEY(to_entity_id) REFERENCES entities(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS locations (
            entity_id INTEGER NOT NULL,
            local_path TEXT,
            git_url TEXT,
            UNIQUE(entity_id),
            FOREIGN KEY(entity_id) REFERENCES entities(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS note_refs (
            entity_id INTEGER NOT NULL,
            note_path TEXT NOT NULL,
            UNIQUE(entity_id),
            FOREIGN KEY(entity_id) REFERENCES entities(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_entities_canonical_name ON entities(canonical_name);
        CREATE INDEX IF NOT EXISTS idx_entities_namespace ON entities(namespace);
        CREATE INDEX IF NOT EXISTS idx_entities_package_name ON entities(package_name);
        CREATE INDEX IF NOT EXISTS idx_entities_repo_name ON entities(repo_name);
        CREATE INDEX IF NOT EXISTS idx_aliases_alias ON aliases(alias);
        ",
    )?;

    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p knowledge-core bootstrap_creates_core_tables -- --exact`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/knowledge-core/Cargo.toml crates/knowledge-core/src/lib.rs crates/knowledge-core/src/model.rs crates/knowledge-core/src/schema.rs crates/knowledge-core/tests/schema_bootstrap.rs Cargo.toml
git commit -m "feat: add knowledge schema bootstrap"
```

### Task 3: Implement Exact Lookup, Upserts, And Graph Expansion

**Files:**
- Modify: `crates/knowledge-core/src/lib.rs`
- Create: `crates/knowledge-core/src/store.rs`
- Create: `crates/knowledge-core/tests/exact_lookup.rs`

- [ ] **Step 1: Write the failing test**

```rust
use knowledge_core::model::{EntityKind, RelationshipKind};
use knowledge_core::schema::bootstrap;
use knowledge_core::store::{EntityInput, KnowledgeStore};
use rusqlite::Connection;

#[test]
fn lookup_by_namespace_expands_to_project_system_and_domain() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);

    let domain_id = store
        .upsert_entity(EntityInput::new("marketplaces", EntityKind::Domain))
        .unwrap();
    let system_id = store
        .upsert_entity(EntityInput::new("ebay", EntityKind::System))
        .unwrap();
    let project_id = store
        .upsert_entity(EntityInput::new("ebay-common", EntityKind::Project))
        .unwrap();
    let library_id = store
        .upsert_entity(
            EntityInput::new("MyCompanyName.Ebay.Custom.Client", EntityKind::Library)
                .with_namespace("MyCompanyName.Ebay.Custom.Client"),
        )
        .unwrap();

    store.link(domain_id, system_id, RelationshipKind::Contains).unwrap();
    store.link(system_id, project_id, RelationshipKind::Contains).unwrap();
    store.link(project_id, library_id, RelationshipKind::Publishes).unwrap();

    let result = store
        .lookup_exact("MyCompanyName.Ebay.Custom.Client")
        .unwrap()
        .expect("library match");

    assert_eq!(result.entity.canonical_name, "MyCompanyName.Ebay.Custom.Client");
    assert!(result.related.iter().any(|entity| entity.canonical_name == "ebay-common"));
    assert!(result.related.iter().any(|entity| entity.canonical_name == "ebay"));
    assert!(result.related.iter().any(|entity| entity.canonical_name == "marketplaces"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p knowledge-core lookup_by_namespace_expands_to_project_system_and_domain -- --exact`
Expected: FAIL because `store` APIs do not exist yet.

- [ ] **Step 3: Write the minimal exact lookup implementation**

`crates/knowledge-core/src/lib.rs`

```rust
pub mod model;
pub mod schema;
pub mod store;
```

`crates/knowledge-core/src/store.rs`

```rust
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

use crate::model::{EntityKind, RelationshipKind};

#[derive(Debug, Clone)]
pub struct EntityInput {
    pub canonical_name: String,
    pub kind: EntityKind,
    pub summary: String,
    pub namespace: Option<String>,
    pub package_name: Option<String>,
    pub repo_name: Option<String>,
}

impl EntityInput {
    pub fn new(name: &str, kind: EntityKind) -> Self {
        Self {
            canonical_name: name.to_string(),
            kind,
            summary: String::new(),
            namespace: None,
            package_name: None,
            repo_name: None,
        }
    }

    pub fn with_namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }
}

#[derive(Debug, Clone)]
pub struct EntityRecord {
    pub id: i64,
    pub canonical_name: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct ExactLookup {
    pub entity: EntityRecord,
    pub related: Vec<EntityRecord>,
}

pub struct KnowledgeStore<'a> {
    conn: &'a Connection,
}

impl<'a> KnowledgeStore<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn upsert_entity(&self, input: EntityInput) -> Result<i64> {
        self.conn.execute(
            "
            INSERT INTO entities (canonical_name, kind, summary, namespace, package_name, repo_name)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(canonical_name) DO UPDATE SET
                kind = excluded.kind,
                summary = excluded.summary,
                namespace = excluded.namespace,
                package_name = excluded.package_name,
                repo_name = excluded.repo_name,
                updated_at = CURRENT_TIMESTAMP
            ",
            params![
                input.canonical_name,
                input.kind.as_str(),
                input.summary,
                input.namespace,
                input.package_name,
                input.repo_name
            ],
        )?;

        let id = self.conn.query_row(
            "SELECT id FROM entities WHERE canonical_name = ?1",
            [input.canonical_name],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    pub fn link(&self, from_id: i64, to_id: i64, kind: RelationshipKind) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO relationships (from_entity_id, to_entity_id, kind) VALUES (?1, ?2, ?3)",
            params![from_id, to_id, kind.as_str()],
        )?;
        Ok(())
    }

    pub fn lookup_exact(&self, query: &str) -> Result<Option<ExactLookup>> {
        let entity = self
            .conn
            .query_row(
                "
                SELECT id, canonical_name, kind
                FROM entities
                WHERE canonical_name = ?1 OR namespace = ?1 OR package_name = ?1 OR repo_name = ?1
                OR id IN (SELECT entity_id FROM aliases WHERE alias = ?1)
                LIMIT 1
                ",
                [query],
                |row| {
                    Ok(EntityRecord {
                        id: row.get(0)?,
                        canonical_name: row.get(1)?,
                        kind: row.get(2)?,
                    })
                },
            )
            .optional()?;

        let Some(entity) = entity else {
            return Ok(None);
        };

        let mut stmt = self.conn.prepare(
            "
            WITH RECURSIVE related(id, canonical_name, kind) AS (
                SELECT e.id, e.canonical_name, e.kind
                FROM entities e
                WHERE e.id = ?1
                UNION
                SELECT e.id, e.canonical_name, e.kind
                FROM relationships r
                JOIN related current ON r.from_entity_id = current.id OR r.to_entity_id = current.id
                JOIN entities e ON e.id = CASE
                    WHEN r.from_entity_id = current.id THEN r.to_entity_id
                    ELSE r.from_entity_id
                END
            )
            SELECT id, canonical_name, kind FROM related WHERE id != ?1
            ",
        )?;

        let related = stmt
            .query_map([entity.id], |row| {
                Ok(EntityRecord {
                    id: row.get(0)?,
                    canonical_name: row.get(1)?,
                    kind: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(ExactLookup { entity, related }))
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p knowledge-core lookup_by_namespace_expands_to_project_system_and_domain -- --exact`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/knowledge-core/src/lib.rs crates/knowledge-core/src/store.rs crates/knowledge-core/tests/exact_lookup.rs
git commit -m "feat: implement exact knowledge lookup"
```

### Task 4: Add Compact Notes And Query Output Shaping

**Files:**
- Modify: `crates/knowledge-core/src/lib.rs`
- Create: `crates/knowledge-core/src/notes.rs`
- Modify: `crates/knowledge-core/src/store.rs`
- Create: `crates/knowledge-core/tests/query_output.rs`
- Create: `.local/knowledge/notes/.gitkeep`
- Create: `config/knowledge/templates/domain.md`
- Create: `config/knowledge/templates/system.md`
- Create: `config/knowledge/templates/project.md`
- Create: `config/knowledge/templates/library.md`

- [ ] **Step 1: Write the failing test**

```rust
use camino::Utf8PathBuf;
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::{EntityInput, KnowledgeStore};
use knowledge_core::model::EntityKind;
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn exact_query_loads_only_the_primary_note_summary() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);
    let temp = tempdir().unwrap();
    let notes = NoteStore::new(Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap());

    let library_id = store
        .upsert_entity(EntityInput::new("MyCompanyName.Ebay.Custom.Client", EntityKind::Library))
        .unwrap();
    notes
        .write_note("library", "mycompany-ebay-custom-client.md", "# Client\n\nUsed to call Ebay custom endpoints.")
        .unwrap();
    store
        .attach_note(library_id, "library/mycompany-ebay-custom-client.md")
        .unwrap();

    let answer = store.query_exact("MyCompanyName.Ebay.Custom.Client", &notes).unwrap().unwrap();

    assert_eq!(answer.summary, "Used to call Ebay custom endpoints.");
    assert!(answer.navigation_hints.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p knowledge-core exact_query_loads_only_the_primary_note_summary -- --exact`
Expected: FAIL because `NoteStore`, `attach_note`, and `query_exact` do not exist yet.

- [ ] **Step 3: Write minimal note and query composition code**

`crates/knowledge-core/src/lib.rs`

```rust
pub mod model;
pub mod notes;
pub mod schema;
pub mod store;
```

`crates/knowledge-core/src/notes.rs`

```rust
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;

pub struct NoteStore {
    root: Utf8PathBuf,
}

impl NoteStore {
    pub fn new(root: Utf8PathBuf) -> Self {
        Self { root }
    }

    pub fn write_note(&self, folder: &str, file_name: &str, body: &str) -> Result<Utf8PathBuf> {
        let dir = self.root.join(folder);
        fs::create_dir_all(dir.as_std_path())?;
        let path = dir.join(file_name);
        fs::write(path.as_std_path(), body)?;
        Ok(path)
    }

    pub fn read_note(&self, relative_path: &str) -> Result<String> {
        Ok(fs::read_to_string(self.root.join(relative_path).as_std_path())?)
    }

    pub fn relative_path<'a>(&self, path: &'a Utf8Path) -> &'a str {
        path.strip_prefix(&self.root).unwrap().as_str()
    }
}
```

Add to `crates/knowledge-core/src/store.rs`

```rust
use crate::notes::NoteStore;

#[derive(Debug, Clone)]
pub struct QueryAnswer {
    pub canonical_name: String,
    pub summary: String,
    pub navigation_hints: Vec<String>,
}

pub fn first_paragraph(markdown: &str) -> String {
    markdown
        .lines()
        .skip_while(|line| line.trim().is_empty() || line.starts_with('#'))
        .take_while(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

impl<'a> KnowledgeStore<'a> {
    pub fn attach_note(&self, entity_id: i64, note_path: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO note_refs (entity_id, note_path) VALUES (?1, ?2) ON CONFLICT(entity_id) DO UPDATE SET note_path = excluded.note_path",
            params![entity_id, note_path],
        )?;
        Ok(())
    }

    pub fn query_exact(&self, query: &str, notes: &NoteStore) -> Result<Option<QueryAnswer>> {
        let lookup = match self.lookup_exact(query)? {
            Some(lookup) => lookup,
            None => return Ok(None),
        };

        let note_path = self.conn.query_row(
            "SELECT note_path FROM note_refs WHERE entity_id = ?1",
            [lookup.entity.id],
            |row| row.get::<_, String>(0),
        ).optional()?;

        let summary = match note_path {
            Some(path) => first_paragraph(&notes.read_note(&path)?),
            None => String::new(),
        };

        Ok(Some(QueryAnswer {
            canonical_name: lookup.entity.canonical_name,
            summary,
            navigation_hints: Vec::new(),
        }))
    }
}
```

`config/knowledge/templates/library.md`

```md
# Library Name

One sentence describing what the library is for.

- Start in: `src/` or the main package folder
- Check: the client entrypoint or dependency registration first
```

`config/knowledge/templates/domain.md`

```md
# Domain Name

One sentence describing the business boundary.

- Shared structure:
- Important systems:
```

`config/knowledge/templates/system.md`

```md
# System Name

One sentence describing what this system owns.

- Repo layout:
- Start in:
```

`config/knowledge/templates/project.md`

```md
# Project Name

One sentence describing the project's purpose.

- Entry point:
- Key package or namespace:
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p knowledge-core exact_query_loads_only_the_primary_note_summary -- --exact`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/knowledge-core/src/lib.rs crates/knowledge-core/src/notes.rs crates/knowledge-core/src/store.rs crates/knowledge-core/tests/query_output.rs .local/knowledge/notes/.gitkeep config/knowledge/templates/domain.md config/knowledge/templates/system.md config/knowledge/templates/project.md config/knowledge/templates/library.md
git commit -m "feat: add compact knowledge notes"
```

### Task 5: Implement Config-Driven Init And Conservative Refresh

**Files:**
- Modify: `crates/knowledge-core/src/lib.rs`
- Create: `crates/knowledge-core/src/import.rs`
- Create: `crates/knowledge-core/tests/import_sources.rs`
- Create: `config/knowledge/sources.example.json`

- [ ] **Step 1: Write the failing test**

```rust
use camino::Utf8PathBuf;
use knowledge_core::import::{apply_source_file, SourceFile};
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;

#[test]
fn source_file_import_is_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    let temp = tempdir().unwrap();
    let file = temp.path().join("sources.json");
    fs::write(
        &file,
        r#"{
          "entities": [
            {
              "canonical_name": "ebay-common",
              "kind": "project",
              "repo_name": "Common",
              "local_path": "C:/repos/Ebay/Common",
              "git_url": "https://example.invalid/marketplaces/ebay/Common.git"
            }
          ]
        }"#,
    )
    .unwrap();

    apply_source_file(
        &conn,
        Utf8PathBuf::from_path_buf(file.clone()).unwrap().as_path(),
    )
    .unwrap();
    apply_source_file(
        &conn,
        Utf8PathBuf::from_path_buf(file).unwrap().as_path(),
    )
    .unwrap();

    let store = KnowledgeStore::new(&conn);
    let result = store.lookup_exact("ebay-common").unwrap().unwrap();

    assert_eq!(result.entity.canonical_name, "ebay-common");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p knowledge-core source_file_import_is_idempotent -- --exact`
Expected: FAIL because the import module does not exist yet.

- [ ] **Step 3: Write the minimal importer**

`crates/knowledge-core/src/lib.rs`

```rust
pub mod import;
pub mod model;
pub mod notes;
pub mod schema;
pub mod store;
```

`crates/knowledge-core/src/import.rs`

```rust
use anyhow::Result;
use camino::Utf8Path;
use rusqlite::{params, Connection};
use serde::Deserialize;

use crate::model::EntityKind;
use crate::store::{EntityInput, KnowledgeStore};

#[derive(Debug, Deserialize)]
pub struct SourceFile {
    pub entities: Vec<SourceEntity>,
}

#[derive(Debug, Deserialize)]
pub struct SourceEntity {
    pub canonical_name: String,
    pub kind: String,
    pub repo_name: Option<String>,
    pub namespace: Option<String>,
    pub package_name: Option<String>,
    pub local_path: Option<String>,
    pub git_url: Option<String>,
}

pub fn apply_source_file(conn: &Connection, path: &Utf8Path) -> Result<()> {
    let source: SourceFile = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let store = KnowledgeStore::new(conn);

    for entity in source.entities {
        let kind = match entity.kind.as_str() {
            "domain" => EntityKind::Domain,
            "system" => EntityKind::System,
            "project" => EntityKind::Project,
            "library" => EntityKind::Library,
            "tag" => EntityKind::Tag,
            "lesson" => EntityKind::Lesson,
            "issue" => EntityKind::Issue,
            other => anyhow::bail!("unsupported entity kind: {other}"),
        };

        let id = store.upsert_entity(EntityInput {
            canonical_name: entity.canonical_name,
            kind,
            summary: String::new(),
            namespace: entity.namespace,
            package_name: entity.package_name,
            repo_name: entity.repo_name,
        })?;

        conn.execute(
            "
            INSERT INTO locations (entity_id, local_path, git_url)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(entity_id) DO UPDATE SET
                local_path = excluded.local_path,
                git_url = excluded.git_url
            ",
            params![id, entity.local_path, entity.git_url],
        )?;
    }

    Ok(())
}
```

`config/knowledge/sources.example.json`

```json
{
  "entities": [
    {
      "canonical_name": "marketplaces",
      "kind": "domain"
    },
    {
      "canonical_name": "ebay",
      "kind": "system"
    },
    {
      "canonical_name": "ebay-common",
      "kind": "project",
      "repo_name": "Common",
      "local_path": "C:/repos/Ebay/Common",
      "git_url": "https://MyCompanyName-gitlab.de/marketplaces/ebay/Common.git"
    },
    {
      "canonical_name": "MyCompanyName.Ebay.Custom.Client",
      "kind": "library",
      "namespace": "MyCompanyName.Ebay.Custom.Client"
    }
  ]
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p knowledge-core source_file_import_is_idempotent -- --exact`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/knowledge-core/src/lib.rs crates/knowledge-core/src/import.rs crates/knowledge-core/tests/import_sources.rs config/knowledge/sources.example.json
git commit -m "feat: add knowledge source import"
```

### Task 6: Capture Lessons And Issues As First-Class Knowledge

**Files:**
- Modify: `crates/knowledge-core/src/lib.rs`
- Create: `crates/knowledge-core/src/capture.rs`
- Modify: `crates/knowledge-core/src/store.rs`
- Create: `crates/knowledge-core/tests/capture.rs`
- Create: `config/knowledge/templates/lesson.md`
- Create: `config/knowledge/templates/issue.md`

- [ ] **Step 1: Write the failing test**

```rust
use camino::Utf8PathBuf;
use knowledge_core::capture::{capture_issue, capture_lesson};
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn captured_lesson_is_queryable_by_exact_name() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let notes_root = tempdir().unwrap();
    let notes = NoteStore::new(Utf8PathBuf::from_path_buf(notes_root.path().to_path_buf()).unwrap());

    let lesson_name = capture_lesson(
        &conn,
        &notes,
        "prefer-curated-mappings-over-guesses",
        "Never invent a repo mapping when the configured source is missing.",
    )
    .unwrap();

    let store = KnowledgeStore::new(&conn);
    let answer = store.query_exact(&lesson_name, &notes).unwrap().unwrap();

    assert_eq!(answer.summary, "Never invent a repo mapping when the configured source is missing.");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p knowledge-core captured_lesson_is_queryable_by_exact_name -- --exact`
Expected: FAIL because capture helpers do not exist yet.

- [ ] **Step 3: Write minimal capture helpers**

`crates/knowledge-core/src/lib.rs`

```rust
pub mod capture;
pub mod import;
pub mod model;
pub mod notes;
pub mod schema;
pub mod store;
```

`crates/knowledge-core/src/capture.rs`

```rust
use anyhow::Result;
use crate::model::EntityKind;
use crate::notes::NoteStore;
use crate::store::{EntityInput, KnowledgeStore};
use rusqlite::Connection;

pub fn capture_lesson(conn: &Connection, notes: &NoteStore, slug: &str, body: &str) -> Result<String> {
    capture(conn, notes, slug, body, EntityKind::Lesson, "lesson")
}

pub fn capture_issue(conn: &Connection, notes: &NoteStore, slug: &str, body: &str) -> Result<String> {
    capture(conn, notes, slug, body, EntityKind::Issue, "issue")
}

fn capture(
    conn: &Connection,
    notes: &NoteStore,
    slug: &str,
    body: &str,
    kind: EntityKind,
    folder: &str,
) -> Result<String> {
    let store = KnowledgeStore::new(conn);
    let canonical_name = slug.to_string();
    let id = store.upsert_entity(EntityInput::new(&canonical_name, kind))?;
    let note = format!("# {slug}\n\n{body}\n");
    let path = notes.write_note(folder, &format!("{slug}.md"), &note)?;
    store.attach_note(id, notes.relative_path(&path))?;
    Ok(canonical_name)
}
```

`config/knowledge/templates/lesson.md`

```md
# Lesson Slug

One short corrective rule.

- Trigger:
- Mistake:
- Reminder:
```

`config/knowledge/templates/issue.md`

```md
# Issue Slug

One short problem statement.

- Why it matters:
- Impact:
- Next step:
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p knowledge-core captured_lesson_is_queryable_by_exact_name -- --exact`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/knowledge-core/src/lib.rs crates/knowledge-core/src/capture.rs crates/knowledge-core/tests/capture.rs config/knowledge/templates/lesson.md config/knowledge/templates/issue.md
git commit -m "feat: add lesson and issue capture"
```

### Task 7: Wire The Real CLI Commands And End-To-End Output

**Files:**
- Modify: `crates/knowledge-cli/Cargo.toml`
- Modify: `crates/knowledge-cli/src/main.rs`
- Create: `crates/knowledge-cli/tests/query_cli.rs`
- Modify: `README.md`
- Modify: `docs/features/knowledge-system/implementation-plan.md`

- [ ] **Step 1: Write the failing test**

```rust
use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::tempdir;

#[test]
fn query_command_returns_summary_and_missing_mapping_message() {
    let temp = tempdir().unwrap();
    let db = temp.path().join("knowledge.db");
    let notes = temp.path().join("notes");
    let source = temp.path().join("sources.json");

    fs::write(
        &source,
        r#"{
          "entities": [
            {
              "canonical_name": "MyCompanyName.Ebay.Custom.Client",
              "kind": "library",
              "namespace": "MyCompanyName.Ebay.Custom.Client"
            }
          ]
        }"#,
    )
    .unwrap();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "init",
            "--db",
            db.to_str().unwrap(),
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    Command::cargo_bin("knowledge-cli")
        .unwrap()
        .args([
            "query",
            "MyCompanyName.Ebay.Custom.Client",
            "--db",
            db.to_str().unwrap(),
            "--notes-root",
            notes.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(contains("MyCompanyName.Ebay.Custom.Client"))
        .stdout(contains("No note summary stored"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p knowledge-cli query_command_returns_summary_and_missing_mapping_message -- --exact`
Expected: FAIL because the CLI still only parses empty subcommands.

- [ ] **Step 3: Implement the CLI against the core crate**

`crates/knowledge-cli/Cargo.toml`

```toml
[package]
name = "knowledge-cli"
edition.workspace = true
license.workspace = true
version.workspace = true

[dependencies]
anyhow.workspace = true
camino.workspace = true
clap.workspace = true
knowledge-core = { path = "../knowledge-core" }
rusqlite.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

[dev-dependencies]
assert_cmd.workspace = true
predicates.workspace = true
tempfile.workspace = true
```

`crates/knowledge-cli/src/main.rs`

```rust
use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use knowledge_core::capture::{capture_issue, capture_lesson};
use knowledge_core::import::apply_source_file;
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::KnowledgeStore;
use rusqlite::Connection;

#[derive(Parser)]
#[command(name = "knowledge-cli")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init {
        #[arg(long)]
        db: Utf8PathBuf,
        #[arg(long)]
        source: Utf8PathBuf,
    },
    Query {
        query: String,
        #[arg(long)]
        db: Utf8PathBuf,
        #[arg(long)]
        notes_root: Utf8PathBuf,
    },
    Capture {
        kind: String,
        slug: String,
        body: String,
        #[arg(long)]
        db: Utf8PathBuf,
        #[arg(long)]
        notes_root: Utf8PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().without_time().init();
    match Cli::parse().command {
        Command::Init { db, source } => {
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            apply_source_file(&conn, source.as_path())?;
        }
        Command::Query { query, db, notes_root } => {
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            let store = KnowledgeStore::new(&conn);
            let notes = NoteStore::new(notes_root);
            match store.query_exact(&query, &notes)? {
                Some(answer) if answer.summary.is_empty() => {
                    println!("{}\nNo note summary stored", answer.canonical_name);
                }
                Some(answer) => {
                    println!("{}\n{}", answer.canonical_name, answer.summary);
                }
                None => {
                    println!("No exact entity match found for {query}");
                }
            }
        }
        Command::Capture { kind, slug, body, db, notes_root } => {
            let conn = Connection::open(db.as_std_path())?;
            bootstrap(&conn)?;
            let notes = NoteStore::new(notes_root);
            match kind.as_str() {
                "lesson" => {
                    capture_lesson(&conn, &notes, &slug, &body)?;
                }
                "issue" => {
                    capture_issue(&conn, &notes, &slug, &body)?;
                }
                other => anyhow::bail!("unsupported capture kind: {other}"),
            }
        }
    }

    Ok(())
}
```

`README.md`

```md
## Knowledge CLI

Use the local knowledge tool to initialize a small SQLite store and query exact identifiers:

- `cargo run -p knowledge-cli -- init --db .local/knowledge.db --source config/knowledge/sources.example.json`
- `cargo run -p knowledge-cli -- query MyCompanyName.Ebay.Custom.Client --db .local/knowledge.db --notes-root .local/knowledge/notes`
```

`docs/features/knowledge-system/implementation-plan.md`

```md
# Knowledge System Implementation Plan

This document is superseded by the executable task plan in:

- `docs/superpowers/plans/2026-05-23-knowledge-system.md`

Keep this file as the feature-local pointer and maintain the step-by-step implementation details in the superpowers plan.
```

- [ ] **Step 4: Run verification for the real CLI**

Run: `cargo test -p knowledge-cli -- --nocapture`
Expected: PASS.

Run: `cargo test -p knowledge-core -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/knowledge-cli/Cargo.toml crates/knowledge-cli/src/main.rs crates/knowledge-cli/tests/query_cli.rs README.md docs/features/knowledge-system/implementation-plan.md
git commit -m "feat: expose knowledge init query and capture commands"
```

## Self-Review

### Spec Coverage

- Entity resolution: covered by Tasks 2, 3, and 7.
- Compact metadata versus note separation: covered by Task 4.
- Local SQLite-backed storage: covered by Task 2.
- Exact lookup first: covered by Tasks 3 and 7.
- Session-efficient minimal note loading: the first useful version loads only the primary note in Task 4; session caching is intentionally deferred until the downstream skills feature consumes this CLI.
- Lessons and issues capture: covered by Task 6.
- Initialization and refresh: covered by Task 5.
- Clear failure handling: covered by Task 7 command output.
- Semantic lookup second: intentionally deferred from this first implementation slice. Add it only after these deterministic flows prove useful, using SQLite FTS5 first and optional vector support later.

### Placeholder Scan

- No `TODO`, `TBD`, or "appropriate error handling" placeholders remain.
- Every code-changing step includes concrete file paths, code, and commands.
- The only deliberate deferral is semantic retrieval, which is an explicit scope decision from the spec's recommended progression.

### Type Consistency

- Crate names stay `knowledge-core` and `knowledge-cli` throughout.
- Command names stay `init`, `query`, and `capture` throughout.
- Entity kinds stay `domain`, `system`, `project`, `library`, `tag`, `lesson`, and `issue` throughout.
