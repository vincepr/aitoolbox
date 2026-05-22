# Knowledge System Implementation Plan

## Summary

Implement the knowledge system in small phases, starting with deterministic structured lookup and a minimal capture model before adding any semantic retrieval.

The first useful version should solve exact internal entity resolution and compact knowledge capture. Vector support should be added only after the structured model proves useful.

## Delivery Strategy

Build the feature in this order:

1. define the core entity and relationship model
2. create the SQLite storage layer and query API
3. support minimal curated note storage and retrieval
4. add initialization and refresh flows for mapped entities
5. add lesson and issue capture
6. add semantic retrieval only where exact lookup is insufficient

This keeps the first iteration focused on high-confidence routing instead of broad fuzzy recall.

## Phase 1: Core Model And Boundaries

### Goal

Lock down the minimum useful knowledge model and avoid overbuilding the first version.

### Work

- define the first-class entity kinds: `domain`, `system`, `project`, `library`, `tag`, `lesson`, `issue`
- define the minimal relationship kinds needed for the first version
- define what belongs in structured metadata versus compact agent-facing notes
- define what exact identifiers must be resolvable in v1

### Expected Output

- concrete entity model
- concrete relationship model
- list of exact-match lookup fields
- decision on how note documents are stored and referenced

### Exit Criteria

- the team can describe how `MyCompanyName.Ebay.Custom.Client` resolves to library, project, system, domain, repo path, and git URL
- the model can represent both macro-repo and split-repo structures
- the model keeps aliases, paths, tags, and URLs out of agent-facing note content

## Phase 2: SQLite Schema And Access Layer

### Goal

Create the local source of truth for structured knowledge.

### Work

- define SQLite tables for entities, aliases, links, locations, tags, and note references
- add indexes for exact lookup fields such as aliases, namespaces, package names, repo names, and canonical names
- define a small internal query interface for the future CLI and skills
- define migration strategy for schema evolution

### Expected Output

- initial database schema
- migrations or schema bootstrap flow
- programmatic query API with clear read operations

### Exit Criteria

- exact entity lookup works for names, aliases, namespaces, and package-like identifiers
- linked entities can be expanded without reading freeform documents first
- the schema is small enough to evolve without immediate complexity pressure

## Phase 3: Compact Notes And Retrieval Composition

### Goal

Attach short human-facing knowledge to structured entities without contaminating retrieval with noisy metadata.

### Work

- define the compact note format for domains, systems, projects, libraries, tags, lessons, and issues
- store or reference notes separately from lookup metadata
- implement a retrieval composition layer that combines structured resolution with minimal note loading
- define output shaping for answers such as summary, navigation hints, confidence, and gaps

### Expected Output

- compact note format
- note loading mechanism
- retrieval composition logic for exact-match cases

### Exit Criteria

- a resolved entity can return a short summary plus one or two navigation hints
- raw aliases, URLs, and cross-reference lists are not loaded into agent-readable note content by default
- note loading is limited to the minimum required entities

## Phase 4: Initialization And Refresh Flow

### Goal

Make the system practical to populate and maintain.

### Work

- define configured input sources such as local repo roots and project or package manifests
- implement scanning for repos, packages, namespaces, and other resolvable identifiers
- map discovered identifiers to entities, links, and locations
- detect stale local paths and stale git mappings where possible
- support safe re-runs without duplicating or corrupting curated records

### Expected Output

- initial bootstrap command or workflow
- refresh command or workflow
- stale mapping detection rules

### Exit Criteria

- the system can be initialized from a configured set of local sources
- refresh can update mappings without destroying curated note content
- stale or missing data is flagged clearly instead of guessed

## Phase 5: Lessons And Issues Capture

### Goal

Support low-friction capture of durable workflow knowledge.

### Work

- define minimal structured fields for lessons and issues
- support linking captured lessons and issues to relevant entities and tags
- define guardrails that prevent noisy or weak captures
- make captured records available through the same retrieval flow as other knowledge

### Expected Output

- lesson and issue data model
- create and update flow for captured knowledge
- retrieval support for lesson and issue recall

### Exit Criteria

- a lesson can be captured and later found through linked entities or conceptual search
- an issue can be captured with problem description, impact, and next-step direction
- captures stay short and durable rather than becoming a dumping ground

## Phase 6: Semantic Retrieval

### Goal

Add semantic recall only where it improves real queries that structured lookup cannot answer well.

### Work

- integrate vector support inside SQLite if the chosen library fits the repo constraints
- define which note content is eligible for embedding
- add semantic search for fuzzy architecture questions, lesson recall, and issue recall
- preserve exact structured lookup as the first retrieval path

### Expected Output

- vector-enabled retrieval for selected note content
- fallback logic that combines exact lookup and semantic recall safely

### Exit Criteria

- semantic retrieval improves fuzzy queries without degrading deterministic identifier resolution
- embeddings are only generated for compact curated content
- exact-match flows remain the default for namespace, package, and repo lookup

## Cross-Cutting Constraints

- exact structured lookup must always be cheaper and preferred over semantic search for exact identifiers
- agent-facing knowledge must stay compact and readable
- missing or uncertain mappings should be flagged, not invented
- local code remains the source of truth for implementation behavior
- the system should support both human curation and automated refresh

## Suggested Milestones

### Milestone 1

Deliver a working deterministic knowledge store with:

- entity model
- schema
- exact lookup
- relation expansion
- compact note support for core entities

This milestone should already answer questions like:

- where is `MyCompanyName.Ebay.Custom.Client`
- what repo or macro-repo owns it
- what system and domain it belongs to
- where a developer should start navigating

### Milestone 2

Deliver maintainability features with:

- initialization
- refresh
- stale mapping detection
- lesson and issue capture

### Milestone 3

Deliver semantic recall with:

- vector indexing for compact notes
- fuzzy architecture retrieval
- lesson and issue similarity search

## Verification Strategy

Each phase should be verified with concrete sample queries and known entities.

Minimum verification scenarios:

- exact lookup by canonical library name
- exact lookup by alias or namespace-like identifier
- expansion from library to project, system, and domain
- retrieval of local path and git URL without loading noisy note content
- detection of missing or stale mappings
- lesson capture and recall
- issue capture and recall
- fuzzy retrieval that finds relevant knowledge only after structured lookup is insufficient

## Risks

- over-modeling the schema before real queries validate it
- polluting curated note content with machine-oriented metadata
- relying on semantic search too early and losing deterministic behavior
- making refresh flows destructive to hand-curated knowledge
- allowing lesson and issue capture to become noisy and untrusted

## Recommended First Slice

The first implementation slice should include only:

- core entity schema
- exact lookup indexes
- link and location storage
- compact note references
- one initialization path from configured local repos
- one retrieval path for library and project resolution

This is the smallest version that can already return high-value answers while leaving semantic retrieval for later.

## Dependencies

- feature definition in `docs/features/knowledge-system/README.md`
- later consumer feature in `docs/features/knowledge-system-skills/README.md`

The skill work should stay downstream from this implementation plan.
