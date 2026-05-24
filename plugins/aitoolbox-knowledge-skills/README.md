# AIToolbox Knowledge Skills Plugin

This package bundles two skills:
- `knowledge-update`
- `knowledge-get`

## Install in Codex

Option 1: install plugin directly from GitHub path.

```bash
npx codex-marketplace add vincepr/aitoolbox/plugins/aitoolbox-knowledge-skills --plugin --project
```

Option 2: install through the repo marketplace file.

```bash
codex plugin marketplace add .
codex plugin install aitoolbox-knowledge-skills@aitoolbox
```

## Install in Claude Code

Add the repo marketplace:

```text
/plugin marketplace add vincepr/aitoolbox
```

Install the plugin:

```text
/plugin install aitoolbox-knowledge-skills@aitoolbox-marketplace
```

## Files

- Codex manifest: `.codex-plugin/plugin.json`
- Claude manifest: `.claude-plugin/plugin.json`
- Codex marketplace entry: `../../.agents/plugins/marketplace.json`
- Claude marketplace entry: `../../.claude-plugin/marketplace.json`
