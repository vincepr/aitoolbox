---
name: knowledge-refresh
description: "Run periodic knowledge system governance checks (migration, config authority, docs-code/contract drift) and return prioritized remediation actions."
---

# knowledge-refresh

Use this skill for periodic operational health checks, not routine query/update tasks.

## Triggers
- before releases
- after major refactors
- when retrieval quality regresses
- when docs and behavior may have drifted

## Steps
1. Verify CLI and DB readiness:
   - `knowledge-cli version`
   - `knowledge-cli migrate --verify --dry-run`
2. Validate migration posture:
   - `bash scripts/migrate-knowledge-db.sh`
   - capture required/applied/failed state
3. Validate contract surface:
   - run relevant contract tests (for example `cargo test -p knowledge-cli --test contracts`)
4. Validate docs-code synchronization:
   - `bash scripts/audit-docs.sh`
5. Validate skill/docs parity:
   - ensure canonical skills and plugin copies are synchronized
6. Summarize risk and prioritized next actions.

## Output (strict)
- `status`: `ok|partial|blocked`
- `checks`: `migration,contracts,docs_code,skill_sync,config_authority`
- `failures`: short list
- `risk`: `low|medium|high`
- `next_actions`: up to 5 ordered items

## Policy
- No speculative fixes during verification by default.
- Separate detection from mutation.
- If auto-fix is requested, perform only low-risk local changes and report exactly what changed.
