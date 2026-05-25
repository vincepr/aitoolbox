# Plan 05: Update `knowledge-get` Skill Guidance

## Issue reference
- `docs/issues/05-skill-doesnt-teach-naming.md`

## Scope
- Update skill instructions to match actual CLI behavior and fallback contract.

## Files
- Modify: `skills/core/knowledge-get/SKILL.md`
- Modify: `plugins/aitoolbox-knowledge-skills/skills/knowledge-get/SKILL.md`

## Tasks
1. Add canonical naming rule with concrete positive/negative examples.
2. Add exact-first command example using real canonical format.
3. Add bounded fallback recipe using `knowledge-cli list --grep ...` then re-run `get` once.
4. Add explicit rule to prefer `local:` source path over decompilation while available.
5. Add end-to-end worked example (`get` hit -> location -> source inspection).
6. Verify consistency with CLI help and subcommand behavior implemented in plans 01-03.

## Acceptance checks
- Skill text teaches canonical format and fallback flow concretely.
- Skill guidance aligns with shipped CLI behavior and does not reference unavailable commands.
