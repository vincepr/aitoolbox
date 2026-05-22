# Knowledge System

## Summary

Add a local knowledge system that helps the CLI resolve company-specific architecture, repos, libraries, domains, lessons, and workflow issues without polluting normal prompt context.

The system should separate hidden retrieval metadata from compact agent-readable knowledge so that skills can answer targeted questions cheaply, explain failures clearly, and inspect source code only when needed.

## Problem

The codebase and adjacent company systems contain important knowledge that is hard to re-discover repeatedly:

- business domains such as `marketplaces`
- system boundaries such as `Amazon`, `Ebay`, and `LeroyMerlin`
- repo and macro-repo layout differences across systems
- reusable libraries and published packages such as `MyCompanyName.Ebay.Custom.Client`
- shared patterns such as `Laika`
- short lessons about critical mistakes to avoid next time
- workflow issues that need larger architecture or process improvements

Today this information is scattered across memory, local folders, git hosting, package references, and source code. Loading it directly into prompts is expensive and noisy. Putting aliases, tags, paths, URLs, and cross-references inline in Markdown also pollutes what the agent has to read.

## Goal

Create a query-oriented knowledge system that:

- resolves entities such as domains, systems, projects, libraries, and tags
- maps those entities to local folders, git repos, packages, and related entities
- stores compact human-facing notes separately from structured retrieval metadata
- supports exact lookup first and semantic lookup second
- allows skills to retrieve only the minimum relevant context per task
- captures reusable lessons and workflow issues in a form that is easy to query later

## Non-Goals

- replacing source-code inspection for implementation-specific behavior
- building a general-purpose documentation platform
- storing large tutorials or long version-sensitive notes
- depending on external infrastructure in the first iteration
- using vector search as the primary resolution mechanism for exact identifiers

## Primary Use Cases

### Entity Resolution

Given a query such as `using MyCompanyName.Ebay.Custom.Client`, the system should be able to return:

- the matched library or namespace entity
- its related project, system, and domain
- local repo path such as `C:/repos/Ebay/`
- connected git repo such as `https://MyCompanyName-gitlab.de/marketplaces/ebay/Common.git`
- a very short summary of what the entity is for
- one or two compact navigation hints for where to start in the repo

### Domain And System Understanding

Given a question about `marketplaces`, `Amazon`, or `Ebay`, the system should explain:

- where the boundary sits in the business model
- what shared structure exists across marketplaces
- whether the implementation is macro-repo based, split-repo based, or mixed
- which shared/common projects or libraries are important entry points

### Lesson Recall

Given a task that resembles a previous mistake, the system should surface short critical lessons that reduce repeat failures without dumping lots of historical context.

### Workflow Improvement Recall

Given recurring operational pain, the system should surface issues that describe gaps in the skill or AI workflow itself, similar to compact internal GitHub issues.

## Knowledge Model

The first-class entity types should be:

- `domain`
- `system`
- `project`
- `library`
- `tag`
- `lesson`
- `issue`

Examples:

- `marketplaces` is a `domain`
- `amazon`, `ebay`, and `leroy-merlin` are `system`s
- `ebay-common` and `ebay-prices-api` are `project`s
- `MyCompanyName.Ebay.Custom.Client` is a `library`
- `laika-framework` is a `tag`

These entities should form a graph. A domain contains systems, systems relate to projects, projects may publish libraries, and tags provide cross-cutting query hints.

## Storage Model

Use SQLite as the primary local knowledge store.

The database should hold hidden retrieval metadata such as:

- canonical names and aliases
- namespaces and package names
- local paths and git URLs
- entity relationships and cross-references
- tags
- links to compact note documents
- optional vector embeddings for semantic retrieval

Agent-readable notes should remain short and separate from this metadata. They can be stored as compact documents referenced by the database, but they should not inline noisy lookup fields like aliases, paths, URLs, and cross-reference lists.

This keeps the retrieval layer efficient while keeping agent-facing content readable.

## Technology Choices

The recommended implementation stack for this repository is:

- Rust for the CLI and core knowledge engine
- `clap` for command-line parsing
- `rusqlite` for SQLite access
- SQLite `FTS5` for local full-text search over compact note content
- Markdown for short human-authored note documents
- `serde` and `serde_json` for typed config and import or export shapes
- `thiserror` and `anyhow` for error handling
- `tracing` and `tracing-subscriber` for logs and diagnostics
- `camino` for path handling
- `ignore` or `walkdir` for local repo and manifest scanning

This aligns with the repository direction described in the root `README.md`: local tooling implemented primarily in Rust, with strong typing and explicit schemas.

### Vector Support

Vector search should not be part of the first implementation slice.

The recommended progression is:

1. exact SQL lookup
2. graph expansion
3. SQLite `FTS5`
4. optional vector search

If semantic retrieval proves necessary after the structured model is working well, add vector support inside SQLite rather than introducing a separate vector database immediately.

The current preferred direction for that phase is `sqlite-vector`, but it should remain an additive capability rather than the primary retrieval path.

### Technology Constraints

- keep the first version local-first with no mandatory external service dependencies
- avoid a separate Postgres or vector database in the first iteration
- avoid using TypeScript for the core storage and retrieval engine unless a later boundary clearly requires it
- preserve a clear separation between structured metadata in SQLite and agent-facing note content

## Retrieval Strategy

The knowledge system should use layered retrieval in this order:

1. exact structured lookup
2. graph expansion through related entities
3. compact note loading for matched entities
4. optional vector search for fuzzy or conceptual recall
5. source-code inspection when knowledge is insufficient

Exact lookup should handle:

- namespace strings
- package names
- library names
- repo names
- domain and system names
- configured aliases

Vector search should be used only for:

- fuzzy conceptual questions
- lesson recall
- issue recall
- short architectural note recall when exact lookup is weak

## Skill Interaction Model

Skills should treat the knowledge system as a retrieval service, not as a document corpus to read broadly.

The expected interaction pattern is:

1. classify the task
2. query the knowledge store
3. load only the minimum relevant note content
4. answer if confidence is sufficient
5. inspect source code only when needed

For example, a knowledge retrieval skill should:

- identify whether the question is entity lookup, architecture understanding, lesson recall, or workflow issue recall
- resolve entities by exact lookup before any semantic search
- load at most a few compact related notes such as the matched library, its system, and a relevant tag
- return concise navigation guidance and confidence
- explain gaps explicitly when no mapping or incomplete mapping exists

## Session Efficiency

The skill layer should cache what knowledge has already been loaded during the session, for example:

- loaded domains
- loaded systems
- loaded tags
- recently resolved entities

This allows later queries about the same area, such as repeated `Ebay` questions, to avoid reloading the same notes unless needed.

## Knowledge Capture

The system should support two lightweight capture flows.

### Lessons

Lessons are short, high-signal reminders about critical things that went wrong and should be avoided in future work.

A lesson should focus on:

- trigger or context
- the mistake or risk
- the corrective rule or reminder
- optional linked entities or tags

### Issues

Issues are longer-lived problems in the skill, workflow, or architecture itself that likely need deliberate follow-up work.

An issue should focus on:

- the problem description
- why it matters
- impact on quality, speed, or reliability
- possible next step or architectural direction
- optional linked entities or tags

## Initialization And Update Flow

The initial system should support a skill-driven setup and refresh flow that:

- scans configured local repos and relevant project/package manifests
- registers entities, aliases, locations, and relationships
- records or refreshes compact notes conservatively
- detects stale mappings where possible
- updates embeddings only for the compact semantic content that benefits from fuzzy retrieval

It should prefer marking missing or stale data over inventing uncertain mappings.

## Failure Handling

The system should fail clearly and compactly.

Example failure outcomes:

- no exact entity match found for a namespace or package
- entity found, but no local repo path is mapped
- entity found, but related project or system links are incomplete
- knowledge found, but current runtime behavior still requires source inspection

This is important for trust. The system should show what it knows, what it does not know, and why it recommends the next step.

## Expected Outputs

- a local SQLite-backed knowledge store
- a compact entity and relationship model for domains, systems, projects, libraries, tags, lessons, and issues
- a retrieval skill that uses exact lookup first and semantic lookup second
- an initialization/update skill for building and refreshing the store
- a capture skill for creating lessons and issues
- compact agent-facing notes that stay readable because noisy lookup metadata remains hidden in the database

## Open Questions

- should agent-facing notes live inside SQLite, on disk as separate documents, or both
- what is the minimal entity schema needed for the first useful version
- how much of initialization can be automated from repo/package scanning versus manual curation
- how should stale mappings be detected for local folders and git remotes
- what query API should the CLI expose directly versus keeping behind skills only
