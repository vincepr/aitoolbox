use anyhow::{Context, Result};
use serde::Deserialize;

const DEFAULT_RECALL_TOP_K: u32 = 5;
const MAX_RECALL_TOP_K: u32 = 100;
const DEFAULT_PIPELINE_MAX_ATTEMPTS: u32 = 3;
const MAX_PIPELINE_MAX_ATTEMPTS: u32 = 100;
const DEFAULT_PIPELINE_PROVIDER_BATCH_SIZE: u32 = 32;
const MAX_PIPELINE_PROVIDER_BATCH_SIZE: u32 = 1024;
const DEFAULT_PIPELINE_PROVIDER_TIMEOUT_MS: u64 = 3_000;
const MAX_PIPELINE_PROVIDER_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_EMBEDDINGS_PROVIDER: &str = "none";
const DEFAULT_EMBEDDINGS_MODEL: &str = "embeddinggemma-300m-GGUF";
const DEFAULT_EMBEDDINGS_BASE_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_EMBEDDINGS_TIMEOUT_MS: u64 = 5_000;
const MAX_EMBEDDINGS_TIMEOUT_MS: u64 = 300_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveConfig {
    pub recall: RecallConfig,
    pub embeddings: EmbeddingsConfig,
    pub pipeline: PipelineConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallConfig {
    pub top_k: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingsConfig {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineConfig {
    pub enabled: bool,
    pub max_attempts: u32,
    pub provider: PipelineProviderConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineProviderConfig {
    pub kind: String,
    pub runtime: String,
    pub model: String,
    pub batch_size: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolveOverrides {
    pub recall_top_k: Option<u32>,
    pub embeddings_provider: Option<String>,
    pub embeddings_model: Option<String>,
    pub embeddings_base_url: Option<String>,
    pub embeddings_timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    #[serde(default)]
    recall: FileRecallConfig,
    #[serde(default)]
    embeddings: FileEmbeddingsConfig,
    #[serde(default)]
    pipeline: FilePipelineConfig,
}

#[derive(Debug, Deserialize, Default)]
struct FileRecallConfig {
    top_k: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
struct FileEmbeddingsConfig {
    provider: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
    timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct FilePipelineConfig {
    enabled: Option<bool>,
    max_attempts: Option<u32>,
    #[serde(default)]
    provider: FilePipelineProviderConfig,
}

#[derive(Debug, Deserialize, Default)]
struct FilePipelineProviderConfig {
    kind: Option<String>,
    runtime: Option<String>,
    model: Option<String>,
    batch_size: Option<u32>,
    timeout_ms: Option<u64>,
}

/// Resolves effective config values with precedence: file -> env -> CLI.
///
/// # Arguments
///
/// * `file_json` - Optional JSON config document.
/// * `env_top_k` - Optional recall top-k from environment.
/// * `cli_top_k` - Optional recall top-k from CLI.
/// * `env_embeddings_provider` - Optional embeddings provider from environment.
/// * `cli_embeddings_provider` - Optional embeddings provider from CLI.
/// * `env_embeddings_model` - Optional embeddings model from environment.
/// * `cli_embeddings_model` - Optional embeddings model from CLI.
/// * `env_embeddings_base_url` - Optional embeddings base URL from environment.
/// * `cli_embeddings_base_url` - Optional embeddings base URL from CLI.
/// * `env_embeddings_timeout_ms` - Optional embeddings timeout in ms from environment.
/// * `cli_embeddings_timeout_ms` - Optional embeddings timeout in ms from CLI.
///
/// # Returns
///
/// Effective validated configuration.
///
/// # Errors
///
/// Returns an error when JSON parsing fails or values are invalid.
pub fn resolve(
    file_json: Option<&str>,
    env_overrides: ResolveOverrides,
    cli_overrides: ResolveOverrides,
) -> Result<EffectiveConfig> {
    let file_cfg = match file_json {
        Some(raw) => serde_json::from_str::<FileConfig>(raw)
            .context("failed to parse knowledge config JSON")?,
        None => FileConfig::default(),
    };

    let top_k = cli_overrides
        .recall_top_k
        .or(env_overrides.recall_top_k)
        .or(file_cfg.recall.top_k)
        .unwrap_or(DEFAULT_RECALL_TOP_K);

    if top_k == 0 || top_k > MAX_RECALL_TOP_K {
        anyhow::bail!(
            "invalid config: recall.top_k must be between 1 and {MAX_RECALL_TOP_K}, got {top_k}"
        );
    }

    let embeddings_provider = cli_overrides
        .embeddings_provider
        .or(env_overrides.embeddings_provider)
        .or(file_cfg.embeddings.provider)
        .unwrap_or_else(|| DEFAULT_EMBEDDINGS_PROVIDER.to_string());
    let embeddings_model = cli_overrides
        .embeddings_model
        .or(env_overrides.embeddings_model)
        .or(file_cfg.embeddings.model)
        .unwrap_or_else(|| DEFAULT_EMBEDDINGS_MODEL.to_string());
    let embeddings_base_url = cli_overrides
        .embeddings_base_url
        .or(env_overrides.embeddings_base_url)
        .or(file_cfg.embeddings.base_url)
        .unwrap_or_else(|| DEFAULT_EMBEDDINGS_BASE_URL.to_string());
    let embeddings_timeout_ms = cli_overrides
        .embeddings_timeout_ms
        .or(env_overrides.embeddings_timeout_ms)
        .or(file_cfg.embeddings.timeout_ms)
        .unwrap_or(DEFAULT_EMBEDDINGS_TIMEOUT_MS);

    if embeddings_provider.trim().is_empty() {
        anyhow::bail!("invalid config: embeddings.provider must be a non-empty string");
    }
    if embeddings_model.trim().is_empty() {
        anyhow::bail!("invalid config: embeddings.model must be a non-empty string");
    }
    if embeddings_base_url.trim().is_empty() {
        anyhow::bail!("invalid config: embeddings.base_url must be a non-empty string");
    }
    if embeddings_timeout_ms == 0 || embeddings_timeout_ms > MAX_EMBEDDINGS_TIMEOUT_MS {
        anyhow::bail!(
            "invalid config: embeddings.timeout_ms must be between 1 and {MAX_EMBEDDINGS_TIMEOUT_MS}, got {embeddings_timeout_ms}"
        );
    }

    let max_attempts = file_cfg
        .pipeline
        .max_attempts
        .unwrap_or(DEFAULT_PIPELINE_MAX_ATTEMPTS);
    if max_attempts == 0 || max_attempts > MAX_PIPELINE_MAX_ATTEMPTS {
        anyhow::bail!(
            "invalid config: pipeline.max_attempts must be between 1 and {MAX_PIPELINE_MAX_ATTEMPTS}, got {max_attempts}"
        );
    }

    let provider_batch_size = file_cfg
        .pipeline
        .provider
        .batch_size
        .unwrap_or(DEFAULT_PIPELINE_PROVIDER_BATCH_SIZE);
    if provider_batch_size == 0 || provider_batch_size > MAX_PIPELINE_PROVIDER_BATCH_SIZE {
        anyhow::bail!(
            "invalid config: pipeline.provider.batch_size must be between 1 and {MAX_PIPELINE_PROVIDER_BATCH_SIZE}, got {provider_batch_size}"
        );
    }

    let provider_timeout_ms = file_cfg
        .pipeline
        .provider
        .timeout_ms
        .unwrap_or(DEFAULT_PIPELINE_PROVIDER_TIMEOUT_MS);
    if provider_timeout_ms == 0 || provider_timeout_ms > MAX_PIPELINE_PROVIDER_TIMEOUT_MS {
        anyhow::bail!(
            "invalid config: pipeline.provider.timeout_ms must be between 1 and {MAX_PIPELINE_PROVIDER_TIMEOUT_MS}, got {provider_timeout_ms}"
        );
    }

    let provider_kind = file_cfg
        .pipeline
        .provider
        .kind
        .unwrap_or_else(|| "disabled".to_string());
    let provider_runtime = file_cfg
        .pipeline
        .provider
        .runtime
        .unwrap_or_else(|| "none".to_string());
    let provider_model = file_cfg
        .pipeline
        .provider
        .model
        .unwrap_or_else(|| "none".to_string());

    if provider_kind.trim().is_empty() {
        anyhow::bail!("invalid config: pipeline.provider.kind must be a non-empty string");
    }
    if provider_runtime.trim().is_empty() {
        anyhow::bail!("invalid config: pipeline.provider.runtime must be a non-empty string");
    }
    if provider_model.trim().is_empty() {
        anyhow::bail!("invalid config: pipeline.provider.model must be a non-empty string");
    }

    Ok(EffectiveConfig {
        recall: RecallConfig { top_k },
        embeddings: EmbeddingsConfig {
            provider: embeddings_provider,
            model: embeddings_model,
            base_url: embeddings_base_url,
            timeout_ms: embeddings_timeout_ms,
        },
        pipeline: PipelineConfig {
            enabled: file_cfg.pipeline.enabled.unwrap_or(false),
            max_attempts,
            provider: PipelineProviderConfig {
                kind: provider_kind,
                runtime: provider_runtime,
                model: provider_model,
                batch_size: provider_batch_size,
                timeout_ms: provider_timeout_ms,
            },
        },
    })
}

/// Test helper for precedence and validation checks.
pub fn resolve_for_test(
    file_json: &str,
    env_top_k: Option<u32>,
    cli_top_k: Option<u32>,
) -> Result<EffectiveConfig> {
    resolve(
        Some(file_json),
        ResolveOverrides {
            recall_top_k: env_top_k,
            ..ResolveOverrides::default()
        },
        ResolveOverrides {
            recall_top_k: cli_top_k,
            ..ResolveOverrides::default()
        },
    )
}
