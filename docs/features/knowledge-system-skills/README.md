# Knowledge System Skills

## Summary

Add a small set of skills that use the future knowledge system as a targeted retrieval layer for company-specific architecture, repository mapping, lessons, and workflow issues.

These skills should not own storage design or infrastructure concerns. They should provide disciplined ways for agents to query, initialize, refresh, and capture knowledge while keeping prompt context small and relevant.

## Problem

Even with a strong underlying knowledge system, agents still need consistent behavior for when and how to use it.

Without dedicated skills, agents are likely to:

- read too much knowledge too early
- mix exact entity resolution with vague semantic guessing
- skip reusable lessons and repeat avoidable mistakes
- fail to record important workflow issues for later improvement
- inspect source code before checking whether internal knowledge already answers the question
- dump raw metadata into prompt context instead of returning concise guidance

The storage layer alone is not enough. The AI workflow needs opinionated retrieval and capture behavior on top of it.

## Goal

Define skill-level workflows that let agents use the knowledge system efficiently and predictably.

The skills should:

- query the knowledge system before broad source-code exploration when relevant
- resolve exact identifiers before trying fuzzy semantic search
- load only minimal note content needed for the current task
- explain confidence and failure modes clearly
- capture short lessons and workflow issues in a reusable form
- support initialization and refresh of the knowledge store without turning that work into ad hoc manual steps

## Non-Goals

- defining the final database schema or filesystem layout here
- implementing the CLI or persistence layer in this feature spec
- replacing general-purpose skills such as brainstorming, debugging, or code review
- making every agent interaction depend on the knowledge system
- turning the skills into full autonomous repo crawlers by default

## Proposed Skills

The initial skill set should be:

- `knowledge-query`
- `knowledge-init-update`
- `knowledge-capture`

An optional later skill may be added for review or maintenance, but the first version should stay small.

## Knowledge Query Skill

### Purpose

Help an agent retrieve the minimum knowledge needed to answer a question or to navigate toward the right source code quickly.

### When To Use

Use this skill when the task appears to involve:

- company-specific libraries, namespaces, packages, repos, or systems
- business domains such as `marketplaces`
- provider-specific architecture such as `Amazon` or `Ebay`
- previously captured lessons or workflow issues
- internal tags or framework concepts such as `Laika`

### Expected Behavior

The skill should:

1. classify the task
2. query the knowledge system using exact structured lookup first
3. expand through direct relations only as needed
4. load only minimal agent-facing note content
5. report concise findings, confidence, and next steps
6. recommend source inspection only if the knowledge answer is insufficient

### Query Types

The skill should distinguish between at least these query classes:

- entity lookup
- domain or system understanding
- navigation help
- lesson recall
- issue recall

### Expected Output Shape

The result should stay compact and should typically include:

- matched entity or entities
- short summary
- local repo path or other relevant location when available
- connected git repo when available
- one or two navigation hints
- linked domain, system, project, or tag context when relevant
- confidence level
- failure explanation if the knowledge is incomplete

## Knowledge Initialization And Update Skill

### Purpose

Create and refresh the knowledge system contents in a controlled way.

### When To Use

Use this skill when:

- the knowledge system is being initialized for the first time
- new repos, packages, systems, or domains need to be registered
- mappings may be stale
- the user explicitly asks to update knowledge

### Expected Behavior

The skill should:

1. inspect configured sources such as local repos and project manifests
2. register or refresh entities, aliases, links, and locations
3. update compact notes conservatively
4. mark uncertainty or stale mappings instead of inventing missing data
5. refresh semantic indexes only for the content that benefits from it

### Constraints

- prefer precise mappings over broad auto-generated summaries
- avoid generating long version-sensitive explanations
- do not silently overwrite curated knowledge without clear justification

## Knowledge Capture Skill

### Purpose

Persist useful operational knowledge discovered during work so future sessions can benefit from it.

### Capture Types

The skill should support:

- `lesson`
- `issue`

### Lesson Capture

Use a lesson when the information is:

- short
- high-signal
- corrective or preventive
- likely to help avoid repeating a mistake

### Issue Capture

Use an issue when the information is:

- a workflow or architecture problem
- not fully solved during the current task
- likely to need follow-up design or implementation work

### Expected Behavior

The skill should:

1. classify whether the finding is a lesson or an issue
2. attach relevant entities or tags when known
3. keep the captured content short and queryable
4. avoid turning transient observations into noisy permanent records

## Interaction Rules

These skills should use the knowledge system with strict discipline.

### Query First, Then Read

They should not load broad documents or scan raw knowledge content before first asking the structured retrieval layer what matches.

### Exact First, Semantic Second

If the user gives an exact namespace, package, class-like identifier, or repo name, the skill should prefer exact resolution before fuzzy retrieval.

### Minimal Context Loading

The skills should load only the notes required for the current answer, such as:

- the matched library note
- one parent system note
- one relevant tag note
- one or two linked lessons if they materially change the next action

### Source-Code Inspection Is A Fallback

If the knowledge system gives sufficient routing information, the skill may answer directly. It should only escalate to code inspection when the user needs implementation-specific behavior or the stored knowledge is incomplete.

## Session Behavior

The skill layer should remember what knowledge has already been loaded in the current session.

Examples:

- loaded domain notes such as `marketplaces`
- loaded system notes such as `ebay`
- loaded tag notes such as `laika-framework`
- recently resolved entities

This avoids repeatedly paying token cost for the same shared context.

## Failure Handling

The skills should fail clearly and compactly.

Examples:

- no entity matched the given namespace or package
- the entity was resolved, but no local repo mapping exists
- the entity was resolved, but the relation to project or system is incomplete
- knowledge exists, but implementation details still require source inspection
- the information is too uncertain to capture automatically as a durable lesson or issue

The agent should understand what was found, what was missing, and why the skill chose the next action.

## Relationship To The Knowledge System Feature

This feature depends on the future knowledge system described in `docs/features/knowledge-system/README.md`.

That feature defines the storage and retrieval substrate. This feature defines how agents should behave when using it.

The knowledge-system work should come first for the underlying CLI, filesystem, and database concerns. These skills should then be implemented as thin, disciplined consumers of that system rather than as an alternative storage solution.

## Expected Outputs

- feature documentation for `knowledge-query`, `knowledge-init-update`, and `knowledge-capture`
- clear rules for when agents should consult the knowledge system
- compact retrieval behavior that minimizes token usage
- capture rules for lessons and issues
- failure-reporting conventions that improve trust in the workflow

## Open Questions

- should these remain three separate skills or later be wrapped by a higher-level orchestrating skill
- what structured result format should the retrieval skill return to the calling agent
- how much automatic capture should ever be allowed without explicit user confirmation
- should there be a dedicated maintenance skill for stale mappings and knowledge cleanup
