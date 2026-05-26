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
  },
  "embeddings": {
    "provider": "none",
    "model": "embeddinggemma-300m-GGUF",
    "base_url": "http://127.0.0.1:11434",
    "timeout_ms": 5000
  }
}
```

`embeddings.provider` supports `none` and `ollama`.

Containerized Ollama startup (recommended):

```bash
docker run -d --name knowledge-ollama -p 11434:11434 ollama/ollama
docker exec -it knowledge-ollama ollama pull embeddinggemma-300m-GGUF
```

Related environment variables:
- `KNOWLEDGE_CLI_CONFIG_FILE`
- `KNOWLEDGE_CLI_RECALL_TOP_K`
- `KNOWLEDGE_CLI_EMBEDDINGS_PROVIDER`
- `KNOWLEDGE_CLI_EMBEDDINGS_MODEL`
- `KNOWLEDGE_CLI_EMBEDDINGS_BASE_URL`
- `KNOWLEDGE_CLI_EMBEDDINGS_TIMEOUT_MS`

Related CLI flags:
- `--config-file <path>`
- `--recall-top-k <u32>`
- `--embeddings-provider <none|ollama>`
- `--embeddings-model <name>`
- `--embeddings-base-url <url>`
- `--embeddings-timeout-ms <u64>`
