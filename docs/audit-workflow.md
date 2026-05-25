# Docs-to-Code Audit Workflow

Run `./scripts/audit-docs.sh` before opening a PR when docs/specs or core behavior changed.

- `first audit`: no previous checkpoint exists; a new checkpoint was created.
- `skip audit`: no relevant drift since the last checkpoint.
- `re-audit`: relevant drift detected; checkpoint updated and docs should be reviewed.

State is stored in `docs/.audit-state.json`.

CI rule:
- Pull requests that modify `docs/architecture/**` must also update `docs/.audit-state.json`.
- The check is enforced by `.github/workflows/docs-audit-check.yml`.
