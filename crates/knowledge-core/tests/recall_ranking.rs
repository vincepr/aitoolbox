use knowledge_core::recall::recall;
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
