# Reconciled 09: Embedding Provider Abstraction (CLI-First) Design

**Goal:** Add replaceable embedding providers through `knowledge-core` interfaces so semantic recall can run in CLI mode without requiring daemon architecture.

## Scope
- Add provider trait(s) in `knowledge-core` for embedding generation.
- Add typed provider config (disabled by default).
- Add provider runtime adapters (initially `none`, then pluggable command/container adapters).
- Keep deterministic exact lookup as primary path.

## Requirements
1. Provider contract is runtime-agnostic (`embed_texts` batch API, typed errors, timeout/cancel).
2. Provider selection is config-driven and replaceable without changing recall logic.
3. Provider disabled mode preserves existing deterministic behavior exactly.
4. Provider interface lives in core, not CLI or daemon-specific crates.

## Testing
- Unit tests for provider selection/validation.
- Contract tests for provider trait behavior (`none`, mock success, mock failure, timeout).
- Regression tests proving deterministic retrieval is unchanged when provider is disabled.

## Done Criteria
- Embedding provider is configurable and replaceable.
- Core retrieval code does not depend on any single model/runtime.
- No daemon dependency introduced.
