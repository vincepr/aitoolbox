# Core Concepts

These concepts should remain stable even if implementation choices change.

## Project Definition

A typed description of a local project, including key paths, important commands, and supported execution strategies.

## Execution Strategy

The way work is performed against a project. The system should be designed to support at least:

- `in-place`
- `container`
- `isolated-worktree`

Even before all strategies are implemented, the model should acknowledge them explicitly.

## Tool Adapter

A boundary for integrating a specific AI CLI or related local tool while keeping internal models consistent.

## Skill

A reusable task definition or workflow recipe that can consume local context, knowledge sources, and tool capabilities.

## Knowledge Source

A declared source of relevant information such as files, docs, schemas, indexes, APIs, or MCP-backed systems. The framework should describe how to find and use these sources without embedding domain knowledge in the core.

## Context Pack

A generated bundle of relevant project facts, knowledge pointers, workflow hints, and tool-specific context prepared for an AI coding session.
