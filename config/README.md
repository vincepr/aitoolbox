# Config

Human-authored configuration, templates, and schemas belong here.

This area should prefer explicit schemas and stable naming so broken references are detected early.

## knowledge-cli runtime config

`knowledge-cli` supports optional JSON config with deterministic precedence:
file -> environment -> CLI flags.

Supported config keys:

```json
{
  "recall": {
    "top_k": 5
  }
}
```

Related environment variables:
- `KNOWLEDGE_CLI_CONFIG_FILE`
- `KNOWLEDGE_CLI_RECALL_TOP_K`

Related CLI flags:
- `--config-file <path>`
- `--recall-top-k <u32>`
