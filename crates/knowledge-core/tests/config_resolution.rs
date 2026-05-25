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
