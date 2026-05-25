use knowledge_core::recall::{recall, recall_with_options, RecallOptions};
use knowledge_core::schema::bootstrap;
use rusqlite::Connection;

#[test]
fn recall_order_is_stable_for_same_input() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    conn.execute(
        "INSERT INTO entities (canonical_name, kind, summary) VALUES (?1, 'library', '')",
        ["MyCompanyName.Ebay.Custom.Client"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO entities (canonical_name, kind, summary) VALUES (?1, 'library', '')",
        ["MyCompanyName.Ebay.Custom.Helper"],
    )
    .unwrap();

    let first = recall(&conn, "custom", 5).unwrap();
    let second = recall(&conn, "custom", 5).unwrap();
    assert_eq!(first, second);
}

#[test]
fn exact_match_has_higher_score_than_partial_match() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    conn.execute(
        "INSERT INTO entities (canonical_name, kind, summary) VALUES (?1, 'library', '')",
        ["custom"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO entities (canonical_name, kind, summary) VALUES (?1, 'library', '')",
        ["custom-client"],
    )
    .unwrap();

    let ranked = recall(&conn, "custom", 5).unwrap();
    assert_eq!(ranked.first().unwrap().canonical_name, "custom");
}

#[test]
fn recency_weight_can_break_ties_deterministically() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    conn.execute(
        "INSERT INTO entities (canonical_name, kind, summary, updated_at) VALUES (?1, 'library', '', '2024-01-01 00:00:00')",
        ["alpha.custom"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO entities (canonical_name, kind, summary, updated_at) VALUES (?1, 'library', '', '2025-01-01 00:00:00')",
        ["beta.custom"],
    )
    .unwrap();

    let ranked = recall_with_options(
        &conn,
        "custom",
        &RecallOptions {
            top_k: 2,
            recency_weight: 5,
            namespace_diversity_cap: None,
        },
    )
    .unwrap();

    assert_eq!(ranked[0].canonical_name, "beta.custom");
    assert!(ranked[0].score_parts.recency > ranked[1].score_parts.recency);
}

#[test]
fn namespace_diversity_cap_limits_similar_results() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();

    conn.execute(
        "INSERT INTO entities (canonical_name, kind, summary, namespace) VALUES (?1, 'library', '', 'ns1')",
        ["ns1.custom.a"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO entities (canonical_name, kind, summary, namespace) VALUES (?1, 'library', '', 'ns1')",
        ["ns1.custom.b"],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO entities (canonical_name, kind, summary, namespace) VALUES (?1, 'library', '', 'ns2')",
        ["ns2.custom.a"],
    )
    .unwrap();

    let ranked = recall_with_options(
        &conn,
        "custom",
        &RecallOptions {
            top_k: 3,
            recency_weight: 0,
            namespace_diversity_cap: Some(1),
        },
    )
    .unwrap();

    let ns1_count = ranked
        .iter()
        .filter(|r| r.canonical_name.starts_with("ns1."))
        .count();
    assert_eq!(ns1_count, 1);
}
