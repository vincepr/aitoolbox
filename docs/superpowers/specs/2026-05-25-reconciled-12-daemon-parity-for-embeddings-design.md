# Reconciled 12: Daemon Parity for Embedding Workflows Design

**Goal:** Ensure daemon mode reuses the same core embedding and hybrid retrieval behavior as CLI mode, preserving output semantics.

## Scope
- Expose embedding lifecycle and hybrid recall endpoints in daemon mode.
- Reuse `knowledge-core` provider abstraction and ranking logic.
- Add CLI `--daemon-url` parity for embedding and hybrid commands.

## Requirements
1. HTTP handlers do not own schema authority; core startup verifies compatibility.
2. Local CLI and daemon mode produce semantically equivalent outputs for same config/input.
3. Daemon mode remains optional and never required for embeddings.
4. Provider configuration and failure semantics match local CLI behavior.

## Testing
- Endpoint integration tests for embedding lifecycle and hybrid recall.
- Local-vs-daemon parity tests for ranking/provenance fields.
- Concurrency tests for index updates and read consistency.

## Done Criteria
- Embedding/hybrid behavior is parity-tested across CLI local and daemon paths.
- Daemon provides operational convenience only, not architectural lock-in.
