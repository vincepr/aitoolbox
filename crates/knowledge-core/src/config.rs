use anyhow::{Context, Result};
use serde::Deserialize;

const DEFAULT_RECALL_TOP_K: u32 = 5;
const MAX_RECALL_TOP_K: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveConfig {
    pub recall: RecallConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecallConfig {
    pub top_k: u32,
}

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    #[serde(default)]
    recall: FileRecallConfig,
}

#[derive(Debug, Deserialize, Default)]
struct FileRecallConfig {
    top_k: Option<u32>,
}

/// Resolves effective config values with precedence: file -> env -> CLI.
///
/// # Arguments
///
/// * `file_json` - Optional JSON config document.
/// * `env_top_k` - Optional recall top-k from environment.
/// * `cli_top_k` - Optional recall top-k from CLI.
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
    env_top_k: Option<u32>,
    cli_top_k: Option<u32>,
) -> Result<EffectiveConfig> {
    let file_cfg = match file_json {
        Some(raw) => serde_json::from_str::<FileConfig>(raw)
            .context("failed to parse knowledge config JSON")?,
        None => FileConfig::default(),
    };

    let top_k = cli_top_k
        .or(env_top_k)
        .or(file_cfg.recall.top_k)
        .unwrap_or(DEFAULT_RECALL_TOP_K);

    if top_k == 0 || top_k > MAX_RECALL_TOP_K {
        anyhow::bail!(
            "invalid config: recall.top_k must be between 1 and {MAX_RECALL_TOP_K}, got {top_k}"
        );
    }

    Ok(EffectiveConfig {
        recall: RecallConfig { top_k },
    })
}

/// Test helper for precedence and validation checks.
pub fn resolve_for_test(
    file_json: &str,
    env_top_k: Option<u32>,
    cli_top_k: Option<u32>,
) -> Result<EffectiveConfig> {
    resolve(Some(file_json), env_top_k, cli_top_k)
}
