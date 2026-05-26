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
    "provider": "openai-compatible",
    "model": "embeddinggemma",
    "base_url": "http://127.0.0.1:11434/v1",
    "timeout_ms": 5000
  }
}
```

`embeddings.provider` supports `none` and `openai-compatible`.

Embeddings are disabled by default. When `embeddings.provider` is not `none`, `model` and
`base_url` must be configured explicitly.

Containerized Ollama startup and integration test:

```bash
docker compose -f docker-compose.embeddings.ollama.yml up -d
KNOWLEDGE_CLI_EMBEDDINGS_INTEGRATION=1 cargo test \
  openai_compatible_embeddings_index_and_recall_with_real_container --test query_cli
```

Alternative TEI stack with a public Qwen model:

```bash
docker compose -f docker-compose.embeddings.tei.yml up -d
KNOWLEDGE_CLI_EMBEDDINGS_BASE_URL=http://127.0.0.1:18080/v1 \
KNOWLEDGE_CLI_EMBEDDINGS_MODEL=onnx-community/Qwen3-Embedding-0.6B-ONNX \
KNOWLEDGE_CLI_EMBEDDINGS_INTEGRATION=1 cargo test \
  openai_compatible_embeddings_index_and_recall_with_real_container --test query_cli
```

Other OpenAI-compatible embedding servers can use the same provider by changing `base_url` and
`model`.

Related environment variables:
- `KNOWLEDGE_CLI_CONFIG_FILE`
- `KNOWLEDGE_CLI_RECALL_TOP_K`
- `KNOWLEDGE_CLI_EMBEDDINGS_PROVIDER`
- `KNOWLEDGE_CLI_EMBEDDINGS_MODEL`
- `KNOWLEDGE_CLI_EMBEDDINGS_BASE_URL`
- `KNOWLEDGE_CLI_EMBEDDINGS_TIMEOUT_MS`
- `KNOWLEDGE_CLI_EMBEDDINGS_DIMENSIONS`

Related CLI flags:
- `--config-file <path>`
- `--recall-top-k <u32>`
- `--embeddings-provider <none|openai-compatible>`
- `--embeddings-model <name>`
- `--embeddings-base-url <url>`
- `--embeddings-timeout-ms <u64>`
- `--embeddings-dimensions <u32>`
