# Implementation Plan

## Goal

Create additional skills and supporting documentation that work in tandem with `obra/superpowers` to enable containerized development workflows, while keeping the original planning, subagent, review, and TDD methodology largely intact.

## Design Position

This should be treated as a companion integration, not a fork of the whole Superpowers system.

The implementation should:

- preserve reuse of existing methodology where it is not worktree-specific
- introduce explicit container workspace lifecycle steps
- make host-versus-container responsibilities visible
- fail loudly when the environment does not support the selected workflow

## Architectural Approach

Add a small set of companion skills that correspond to the worktree lifecycle edges in Superpowers:

1. container workspace setup
2. container execution guidance
3. container workspace completion or teardown

The skills should complement, not replace, the following existing Superpowers strengths:

- `brainstorming`
- `writing-plans`
- `subagent-driven-development`
- `executing-plans`
- `test-driven-development`
- `requesting-code-review`
- `verification-before-completion`

## Proposed Skill Set

### 1. `using-containerized-workspaces`

Purpose:
- establish a container-backed isolated development environment before implementation begins

Responsibilities:
- detect whether the session is already operating inside an approved containerized workspace
- validate container runtime availability
- validate required project files for container workflow
- create or attach to the container workspace
- run project setup inside the container
- run baseline verification in the container
- report explicit failure if setup or baseline verification fails

Key design rules:
- never silently fall back from container mode to in-place execution
- require clear consent or prior preference before creating containerized state
- distinguish host-side Git state from container-side command execution

### 2. `executing-containerized-plans`

Purpose:
- provide a container-aware execution wrapper for plans when the user wants plan execution to be explicitly container-bound

Responsibilities:
- load and review the plan
- confirm required container workspace is available
- dispatch execution instructions that clearly say which commands run in the container and which run on the host
- stop on environment mismatch, missing mounts, runtime failure, or verification failure

Notes:
- this may remain intentionally thin if `subagent-driven-development` or `executing-plans` can be reused with a strong handoff convention
- if kept thin, its main value is making container execution explicit and reducing ambiguity

### 3. `finishing-containerized-development`

Purpose:
- complete containerized implementation work without assuming worktree cleanup semantics

Responsibilities:
- run final verification in the correct environment
- summarize resulting host Git state
- present completion options that make sense for container-backed work
- handle container cleanup only when ownership is clear
- preserve container state when the user wants to continue iterating

Key difference from worktree completion:
- container cleanup and branch cleanup are not the same concern and must not be conflated

## Required Documentation Work

### Task 1: Define feature folder conventions

- add `docs/features/` as the home for future implementation-oriented features
- document expected contents: overview, implementation plan, resources
- keep `docs/ideas/` for looser concepts

### Task 2: Write companion feature overview

- capture problem statement
- clarify non-goals
- define expected outputs
- state coexistence strategy with Superpowers

### Task 3: Write resource inventory

- record the exact Superpowers skills and docs that inform this feature
- capture local architectural constraints and open questions

### Task 4: Draft companion skill contracts

For each proposed skill, define:

- when to use it
- required inputs
- required environment checks
- success outputs
- failure conditions
- relation to upstream Superpowers skills

## Implementation Work Breakdown

### Phase 1: Documentation-First Design

Deliverables:

- feature overview
- implementation plan
- resources file
- initial skill interface definitions

Exit criteria:

- clear boundary between reusable Superpowers methodology and new container-specific lifecycle skills
- no unresolved ambiguity about why these are companion skills instead of core replacements

### Phase 2: Skill Specification

Deliverables:

- draft `SKILL.md` outlines for each companion skill
- invocation order examples
- environment validation checklist

Exit criteria:

- each skill has a strict purpose and does not overlap excessively with the others
- host/container boundary is explicit in each specification

### Phase 3: Local Tooling Design

Deliverables:

- decision on whether helper tooling is needed
- if needed, define a Rust-first helper CLI boundary for:
  - container detection
  - environment validation
  - baseline verification orchestration
  - lifecycle metadata capture

Exit criteria:

- skill docs remain declarative where possible
- helper tooling is justified only for repeated, structured checks

### Phase 4: Integration Examples

Deliverables:

- example workflow: brainstorming -> writing-plans -> using-containerized-workspaces -> subagent-driven-development -> finishing-containerized-development
- example workflow: writing-plans -> executing-containerized-plans
- example project notes showing host/container responsibilities

Exit criteria:

- examples make the tandem workflow understandable without reading every skill in full

## Open Design Questions

- Should containerization be modeled as a full alternative execution strategy, or as a companion isolation mode attached to an existing strategy?
- Should container-specific execution always imply a separate host-side Git branch, or can it operate directly against the current checkout?
- Should subagents run inside the same container, separate containers, or remain a controller concern outside the container?
- How should mounted volumes, generated artifacts, and caches be described so failures remain explicit?
- Should cleanup decisions preserve containers by default for debugging, similar to preserving worktrees for PR iteration?

## Risks

- duplicating too much of upstream Superpowers instead of composing with it
- vague responsibility splits between host and container behavior
- accidental silent fallback from container execution to host execution
- tying the design too tightly to Docker before broader runtime abstraction is understood
- mixing branch lifecycle logic with container lifecycle logic

## Recommended First Implementation Slice

When implementation starts, begin with:

1. `using-containerized-workspaces`
2. feature-local docs showing exact handoff rules
3. one simple example project contract

Delay:

1. complex teardown automation
2. broad runtime abstraction
3. rich UI or session history integration

## Success Criteria

- a future implementer can tell exactly which parts of Superpowers are reused unchanged
- container lifecycle concerns are isolated into companion skills
- no step depends on hidden fallback behavior
- the resulting workflow remains understandable to a human reviewing the docs alone
