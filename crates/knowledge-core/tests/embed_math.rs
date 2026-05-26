use knowledge_core::embed::cosine_similarity;

#[test]
fn cosine_similarity_handles_basic_case() {
    let left = [1.0_f32, 0.0, 0.0];
    let right = [1.0_f32, 0.0, 0.0];
    let score = cosine_similarity(&left, &right).expect("cosine should exist");
    assert!((score - 1.0).abs() < 1e-6);
}

#[test]
fn cosine_similarity_rejects_mismatched_dims() {
    let left = [1.0_f32, 0.0];
    let right = [1.0_f32, 0.0, 0.0];
    assert!(cosine_similarity(&left, &right).is_none());
}
