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

Install:

- `cargo install --path crates/knowledge-cli`

Use:

- `mkdir -p .local`
- `knowledge-cli init --db .local/knowledge.db --source-file config/knowledge/sources.example.json`
- `knowledge-cli init --db .local/knowledge.db --source-json '{"entities":[{"canonical_name":"MyCompanyName.Ebay.Custom.Client","kind":"library","namespace":"MyCompanyName.Ebay.Custom.Client"}]}'`
- `knowledge-cli get --db .local/knowledge.db --notes-root knowledge/notes --input-json '{"entity":"MyCompanyName.Ebay.Custom.Client"}'`
- `knowledge-cli capture-lesson --db .local/knowledge.db --notes-root knowledge/notes --input-json '{"slug":"avoid-global-singleton","body":"Global state leaked between tests"}'`
- `knowledge-cli capture-issue --db .local/knowledge.db --notes-root knowledge/notes --input-json '{"slug":"stale-mapping-refresh","body":"Need automatic refresh for stale repository paths"}'`
