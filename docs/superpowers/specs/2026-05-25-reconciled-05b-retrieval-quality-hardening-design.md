# Reconciled 05b: Retrieval Quality Hardening Design

**Context:** Added after implementing Task 5, informed by SignetAI and Mnemosyne reference patterns.

## Why this addendum exists
Current Task 5 introduces deterministic hybrid ranking and eval metrics. Two additional guardrails should follow to make it more robust:

1. Introduce bounded dataset-versioning for eval fixtures so metric regressions are comparable over time.
2. Add optional recency weighting and diversity-reranking switches while preserving deterministic defaults.

## Reference-informed notes
- SignetAI emphasizes inspectable recall and durable instrumentation loops.
- Mnemosyne emphasizes deterministic, local, low-latency retrieval and score transparency.

## Proposed follow-up tasks
1. Add `eval dataset` metadata (`dataset_id`, `version`, `generated_at`) and enforce schema in CLI.
2. Add deterministic recency score term sourced from entity update timestamp.
3. Add diversity cap to avoid top-k near-duplicates of one namespace.
4. Add regression baseline file + threshold checks in CI.
