use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Resolved embedding runtime options used by CLI and provider adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddingRuntime {
    /// Provider selector: `none` or `openai-compatible`.
    pub provider: String,
    /// Model identifier passed to provider.
    pub model: String,
    /// Optional base URL for provider runtimes that use HTTP APIs.
    pub base_url: Option<String>,
    /// Provider timeout in milliseconds.
    pub timeout_ms: u64,
    /// Optional output vector dimension count.
    pub dimensions: Option<u32>,
}

/// Text + embedding pair used during semantic ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRow {
    /// SQLite entity id.
    pub entity_id: i64,
    /// Canonical name for user output.
    pub canonical_name: String,
    /// Kind for user output.
    pub kind: String,
    /// Source text fingerprint to detect stale embeddings.
    pub source_fingerprint: String,
    /// Dense vector.
    pub vector: Vec<f32>,
}

/// Provider interface for embedding generation.
pub trait EmbeddingProvider {
    /// Embeds one UTF-8 text input into a dense vector.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

/// Disabled provider implementation for explicit opt-out.
#[derive(Debug, Clone, Copy)]
pub struct DisabledEmbeddingProvider;

impl EmbeddingProvider for DisabledEmbeddingProvider {
    fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        anyhow::bail!("embeddings provider is disabled")
    }
}

/// OpenAI-compatible HTTP embedding provider.
#[derive(Debug, Clone)]
pub struct OpenAiCompatibleEmbeddingProvider {
    base_url: String,
    model: String,
    timeout_ms: u64,
    dimensions: Option<u32>,
}

impl OpenAiCompatibleEmbeddingProvider {
    /// Creates a new OpenAI-compatible embedding provider.
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL ending in `/v1`, for example `http://127.0.0.1:11434/v1`.
    /// * `model` - Embedding model id, for example `embeddinggemma`.
    /// * `timeout_ms` - Request timeout in milliseconds.
    /// * `dimensions` - Optional vector dimension count passed to compatible providers.
    #[must_use]
    pub fn new(base_url: String, model: String, timeout_ms: u64, dimensions: Option<u32>) -> Self {
        Self {
            base_url,
            model,
            timeout_ms,
            dimensions,
        }
    }
}

#[derive(Debug, Serialize)]
struct OpenAiEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
    encoding_format: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}

impl EmbeddingProvider for OpenAiCompatibleEmbeddingProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/embeddings", self.base_url.trim_end_matches('/'));
        let payload = OpenAiEmbeddingRequest {
            model: &self.model,
            input: text,
            encoding_format: "float",
            dimensions: self.dimensions,
        };

        let response = ureq::post(&url)
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .send_json(
                serde_json::to_value(payload)
                    .context("failed to serialize OpenAI-compatible embeddings request")?,
            )
            .with_context(|| {
                format!(
                    "failed to call OpenAI-compatible embeddings endpoint: {url}\n\
If the container is not running, start it with:\n\
  docker compose -f docker-compose.embeddings.ollama.yml up -d\n\
The configured model is: {}",
                    self.model
                )
            })?;

        let parsed: OpenAiEmbeddingResponse = response
            .into_json()
            .context("failed to parse OpenAI-compatible embeddings response JSON")?;

        let embedding = parsed
            .data
            .into_iter()
            .next()
            .map(|item| item.embedding)
            .unwrap_or_default();

        if embedding.is_empty() {
            anyhow::bail!("OpenAI-compatible provider returned an empty embedding vector")
        }

        Ok(embedding)
    }
}

/// Computes cosine similarity between two vectors.
///
/// # Returns
///
/// `None` when vectors are empty, different lengths, or have zero norm.
#[must_use]
pub fn cosine_similarity(left: &[f32], right: &[f32]) -> Option<f32> {
    if left.is_empty() || left.len() != right.len() {
        return None;
    }

    let mut dot = 0.0_f32;
    let mut left_norm = 0.0_f32;
    let mut right_norm = 0.0_f32;

    for (l, r) in left.iter().zip(right.iter()) {
        dot += l * r;
        left_norm += l * l;
        right_norm += r * r;
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        return None;
    }

    Some(dot / (left_norm.sqrt() * right_norm.sqrt()))
}

/// Stable UTF-8 fingerprint used to identify stale embedding cache rows.
#[must_use]
pub fn fingerprint_text(text: &str) -> String {
    let digest = md5::compute(text.as_bytes());
    format!("{digest:x}")
}
