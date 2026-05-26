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
    "model": "google/embeddinggemma-300m",
    "base_url": "http://127.0.0.1:8080/v1",
    "timeout_ms": 5000,
    "dimensions": 768
  }
}
```

`embeddings.provider` supports `none` and `openai-compatible`.

Default embedding model: [`google/embeddinggemma-300m`](https://huggingface.co/google/embeddinggemma-300m).

This Hugging Face repository is gated; accept the model license and set `HF_TOKEN` before first
container startup.

Containerized Text Embeddings Inference startup and integration test:

```bash
HF_TOKEN=... docker compose -f docker-compose.embeddings.yml up -d tei
KNOWLEDGE_CLI_EMBEDDINGS_INTEGRATION=1 cargo test \
  openai_compatible_embeddings_index_and_recall_with_real_container --test query_cli
```

Other OpenAI-compatible embedding servers can use the same provider by changing `base_url` and
`model`. For example, Ollama's compatible endpoint is typically `http://127.0.0.1:11434/v1`, with
the model name managed by Ollama. The `dimensions` field is sent as the OpenAI-compatible
`dimensions` request field, so TEI and Ollama receive the same vector-size setting.

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
