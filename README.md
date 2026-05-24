# aitoolbox

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

Start with the docs in [docs/architecture](/C:/Users/vince/RiderProjects/aitoolbox/docs/architecture/README.md) and [docs/ideas](/C:/Users/vince/RiderProjects/aitoolbox/docs/ideas/README.md).

## Knowledge CLI

Use the local knowledge tool to initialize a small SQLite store and query exact identifiers:

- `cargo run -p knowledge-cli -- init --db .local/knowledge.db --source config/knowledge/sources.example.json`
- `cargo run -p knowledge-cli -- query MyCompanyName.Ebay.Custom.Client --db .local/knowledge.db --notes-root knowledge/notes`
