# aitoolbox

cargo install --path crates/knowledge-cli

Oneof
- sudo docker compose -f docker-compose.embeddings.tei.yml up
- sudo docker compose -f docker-compose.embeddings.ollama.yml up


Local-first toolbox for AI-assisted coding workflows across multiple CLI tools and execution strategies.

The repository is intentionally structured as a framework host rather than a single application. It is meant to support:

- multiple AI coding CLIs such as Codex, Claude, and OpenCode
- reusable skills and workflow definitions
- project and knowledge discovery on a developer machine
- local or service-backed tooling implemented primarily in Rust, then TypeScript where needed
- future apps such as session history browsing or context inspection

The design favors:

- strong typing and explicit schemas
- fail-fast validation and visible errors
- local text config where simple and practical
- structured storage chosen pragmatically per subsystem
- execution-mode support from the start: in-place, container, isolated worktree

Note: empty scaffolding folders are created on demand and are not kept in git with placeholder files.

Start with the docs in [docs/architecture](docs/architecture/README.md) and [docs/ideas](docs/ideas/README.md).

## Knowledge CLI

Install:

- `cargo install --path crates/knowledge-cli`
- Optional shell alias: `eval "$(knowledge-cli alias bash)"`

Use:

- `knowledge-cli quickstart`
- `knowledge-cli init --source-file config/knowledge/sources.example.json`
- `knowledge-cli init --source-json '{"$schema":"https://aitoolbox/schemas/entity.v1.json","entities":[{"canonical_name":"mycompanyname-ebay-custom-client","kind":"library","summary":null,"namespace":"MyCompanyName.Ebay.Custom.Client","package_name":"MyCompanyName.Ebay.Custom.Client","repo_name":"Custom.Client","aliases":["MyCompanyName.Ebay.Custom.Client"],"location":null,"notes":[]}]}'`
- `knowledge-cli get MyCompanyName.Ebay.Custom.Client`
- `knowledge-cli get mycompanyname-ebay-custom-client`
- `knowledge-cli list --grep custom --limit 20`
- `knowledge-cli capture-lesson --slug avoid-global-singleton --body "Global state leaked between tests"`
- `knowledge-cli capture-issue --slug stale-mapping-refresh --body "Need automatic refresh for stale repository paths"`
- `knowledge-cli completions bash > ~/.local/share/bash-completion/completions/knowledge-cli`
- `knowledge-cli alias bash`
- Edge-case override style (keep DB/notes flags at the end):
- `knowledge-cli get MyCompanyName.Ebay.Custom.Client --db /tmp/knowledge.sqlite3 --notes-root /tmp/notes`
- `knowledge-cli capture-lesson --slug avoid-global-singleton --body "Global state leaked between tests" --db /tmp/knowledge.sqlite3 --notes-root /tmp/notes`

Default path resolution for DB, notes root, and source file is:
1. explicit CLI flag
2. environment variable (`KNOWLEDGE_CLI_DB`, `KNOWLEDGE_CLI_NOTES_ROOT`, `KNOWLEDGE_CLI_SOURCE_FILE`)
3. user-level OS directories (`data_local_dir` / `config_dir`)

Database migration policy:
- major version change of `knowledge-cli` requires DB migration
- migration entrypoint: `scripts/migrate-knowledge-db.sh`

## Install Knowledge Skills (Claude + Codex)

Plugin package path in this repo:
- `plugins/aitoolbox-knowledge-skills`

Codex:
- Install from repo plugin path:
  - `npx codex-marketplace add vincepr/aitoolbox/plugins/aitoolbox-knowledge-skills --plugin --project`
- Or add repo marketplace and install from it:
  - `codex plugin marketplace add .`
  - `codex plugin install aitoolbox-knowledge-skills@aitoolbox`

Claude Code:
- Add repo marketplace:
  - `/plugin marketplace add vincepr/aitoolbox`
- Install plugin:
  - `/plugin install aitoolbox-knowledge-skills@aitoolbox-marketplace`
