use knowledge_core::config::resolve_for_test;

#[test]
fn precedence_is_file_then_env_then_cli() {
    let file_cfg = r#"{"recall":{"top_k":5}}"#;
    let env_top_k = Some(10_u32);
    let cli_top_k = Some(20_u32);
    let effective = resolve_for_test(file_cfg, env_top_k, cli_top_k).unwrap();
    assert_eq!(effective.recall.top_k, 20);
}

#[test]
fn env_overrides_file_when_cli_missing() {
    let file_cfg = r#"{"recall":{"top_k":5}}"#;
    let effective = resolve_for_test(file_cfg, Some(9), None).unwrap();
    assert_eq!(effective.recall.top_k, 9);
}

#[test]
fn invalid_top_k_is_rejected() {
    let file_cfg = r#"{"recall":{"top_k":0}}"#;
    let err = resolve_for_test(file_cfg, None, None).unwrap_err();
    assert!(err.to_string().contains("recall.top_k"));
}

#[test]
fn pipeline_defaults_are_disabled_and_typed() {
    let file_cfg = r#"{"recall":{"top_k":5}}"#;
    let effective = resolve_for_test(file_cfg, None, None).unwrap();
    assert!(!effective.pipeline.enabled);
    assert_eq!(effective.pipeline.max_attempts, 3);
    assert_eq!(effective.pipeline.provider.kind, "disabled");
}

#[test]
fn embeddings_defaults_are_none_and_openai_compatible() {
    let file_cfg = r#"{}"#;
    let effective = resolve_for_test(file_cfg, None, None).expect("defaults resolve");
    assert_eq!(effective.embeddings.provider, "none");
    assert_eq!(effective.embeddings.model, None);
    assert_eq!(effective.embeddings.base_url, None);
    assert_eq!(effective.embeddings.dimensions, None);
}

#[test]
fn enabled_embeddings_require_model_and_base_url() {
    let file_cfg = r#"{"embeddings":{"provider":"openai-compatible"}}"#;
    let err = resolve_for_test(file_cfg, None, None).unwrap_err();
    assert!(err.to_string().contains("embeddings.model"));
}

#[test]
fn invalid_embeddings_dimensions_are_rejected() {
    let file_cfg = r#"{"embeddings":{"dimensions":0}}"#;
    let err = resolve_for_test(file_cfg, None, None).unwrap_err();
    assert!(err.to_string().contains("embeddings.dimensions"));
}
