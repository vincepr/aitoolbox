use camino::Utf8PathBuf;
use knowledge_core::model::{EntityKind, RelationshipKind};
use knowledge_core::notes::NoteStore;
use knowledge_core::schema::bootstrap;
use knowledge_core::store::{EntityInput, KnowledgeStore};
use rusqlite::Connection;
use tempfile::tempdir;

#[test]
fn lookup_by_namespace_expands_to_project_system_and_domain() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);

    let domain_id = store
        .upsert_entity(EntityInput::new("marketplaces", EntityKind::Domain))
        .unwrap();
    let system_id = store
        .upsert_entity(EntityInput::new("ebay", EntityKind::System))
        .unwrap();
    let project_id = store
        .upsert_entity(EntityInput::new("ebay-common", EntityKind::Project))
        .unwrap();
    let library_id = store
        .upsert_entity(
            EntityInput::new("ebay-custom-client", EntityKind::Library)
                .with_namespace("MyCompanyName.Ebay.Custom.Client"),
        )
        .unwrap();

    store
        .link(domain_id, system_id, RelationshipKind::Contains)
        .unwrap();
    store
        .link(system_id, project_id, RelationshipKind::Contains)
        .unwrap();
    store
        .link(project_id, library_id, RelationshipKind::Publishes)
        .unwrap();

    let result = store
        .lookup_exact("MyCompanyName.Ebay.Custom.Client")
        .unwrap()
        .expect("library match");

    assert_eq!(result.entity.canonical_name, "ebay-custom-client");
    assert!(result
        .related
        .iter()
        .any(|entity| entity.canonical_name == "ebay-common"));
    assert!(result
        .related
        .iter()
        .any(|entity| entity.canonical_name == "ebay"));
    assert!(result
        .related
        .iter()
        .any(|entity| entity.canonical_name == "marketplaces"));
}

#[test]
fn exact_lookup_orders_ties_by_canonical_name_then_id() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);

    store
        .upsert_entity(
            EntityInput::new("zeta-library", EntityKind::Library).with_namespace("shared"),
        )
        .unwrap();
    store
        .upsert_entity(
            EntityInput::new("alpha-library", EntityKind::Library).with_namespace("shared"),
        )
        .unwrap();

    let result = store.lookup_exact("shared").unwrap().expect("match");

    assert_eq!(result.entity.canonical_name, "alpha-library");
}

#[test]
fn exact_lookup_prefers_canonical_name_over_lower_precedence_matches() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);

    store
        .upsert_entity(
            EntityInput::new("namespace-match", EntityKind::Library).with_namespace("shared"),
        )
        .unwrap();
    store
        .upsert_entity(EntityInput::new("shared", EntityKind::Project))
        .unwrap();

    let result = store.lookup_exact("shared").unwrap().expect("match");

    assert_eq!(result.entity.canonical_name, "shared");
}

#[test]
fn exact_lookup_applies_field_precedence_before_aliases() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);

    let alias_match_id = store
        .upsert_entity(EntityInput::new("alias-match", EntityKind::Library))
        .unwrap();
    conn.execute(
        "INSERT INTO aliases (entity_id, alias) VALUES (?1, ?2)",
        (alias_match_id, "shared"),
    )
    .unwrap();
    store
        .upsert_entity(EntityInput {
            canonical_name: "repo-match".to_string(),
            kind: EntityKind::Library,
            summary: String::new(),
            namespace: None,
            package_name: None,
            repo_name: Some("shared".to_string()),
        })
        .unwrap();
    store
        .upsert_entity(EntityInput {
            canonical_name: "package-match".to_string(),
            kind: EntityKind::Library,
            summary: String::new(),
            namespace: None,
            package_name: Some("shared".to_string()),
            repo_name: None,
        })
        .unwrap();

    let result = store.lookup_exact("shared").unwrap().expect("match");

    assert_eq!(result.entity.canonical_name, "package-match");
}

#[test]
fn graph_expansion_deduplicates_cycles_and_duplicate_paths() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);

    let root_id = store
        .upsert_entity(EntityInput::new("root", EntityKind::Project))
        .unwrap();
    let first_id = store
        .upsert_entity(EntityInput::new("first", EntityKind::Library))
        .unwrap();
    let second_id = store
        .upsert_entity(EntityInput::new("second", EntityKind::Library))
        .unwrap();

    store
        .link(root_id, first_id, RelationshipKind::RelatedTo)
        .unwrap();
    store
        .link(first_id, second_id, RelationshipKind::RelatedTo)
        .unwrap();
    store
        .link(second_id, root_id, RelationshipKind::RelatedTo)
        .unwrap();
    store
        .link(root_id, second_id, RelationshipKind::RelatedTo)
        .unwrap();

    let result = store.lookup_exact("root").unwrap().expect("match");
    let related_names = result
        .related
        .iter()
        .map(|entity| entity.canonical_name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(related_names, vec!["first", "second"]);
}

#[test]
fn query_exact_returns_location_when_present() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);
    let temp = tempdir().unwrap();
    let notes = NoteStore::new(Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap());
    let canonical_name = "MyCompanyName.Ebay.Custom.Client";

    let entity_id = store
        .upsert_entity(EntityInput::new(canonical_name, EntityKind::Library))
        .unwrap();
    conn.execute(
        "INSERT INTO locations (entity_id, local_path, git_url) VALUES (?1, ?2, ?3)",
        (
            entity_id,
            "/workspace/MyCompanyName.Ebay.Custom.Client",
            "https://example.com/repo.git",
        ),
    )
    .unwrap();

    let answer = store.query_exact(canonical_name, &notes).unwrap().unwrap();
    let location = answer.location.expect("location should be present");
    assert_eq!(
        location.local_path.as_deref(),
        Some("/workspace/MyCompanyName.Ebay.Custom.Client")
    );
    assert_eq!(
        location.git_url.as_deref(),
        Some("https://example.com/repo.git")
    );
}

#[test]
fn exact_lookup_matches_separator_variants_directly() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);

    let entity_id = store
        .upsert_entity(EntityInput {
            canonical_name: "laika-marketplaces-jobs-pricestock".to_string(),
            kind: EntityKind::Library,
            summary: String::new(),
            namespace: Some("Relaxdays.Laika.Marketplaces.Jobs.PriceStock".to_string()),
            package_name: Some("Relaxdays.Laika.Marketplaces.Jobs.PriceStock".to_string()),
            repo_name: Some("PriceStock".to_string()),
        })
        .unwrap();
    conn.execute(
        "INSERT INTO aliases (entity_id, alias) VALUES (?1, ?2)",
        (
            entity_id,
            "laika/Marketplaces/Jobs/PriceStock".to_ascii_lowercase(),
        ),
    )
    .unwrap();

    for query in [
        "laika-marketplaces-jobs-pricestock",
        "Laika.Marketplaces.Jobs.PriceStock",
        "laika/Marketplaces/Jobs/PriceStock",
        "laika_marketplaces.jobs-pricestock",
    ] {
        let result = store.lookup_exact(query).unwrap().expect("match");
        assert_eq!(
            result.entity.canonical_name,
            "laika-marketplaces-jobs-pricestock"
        );
    }
}

#[test]
fn exact_lookup_requires_all_query_tokens_for_candidate_match() {
    let conn = Connection::open_in_memory().unwrap();
    bootstrap(&conn).unwrap();
    let store = KnowledgeStore::new(&conn);

    store
        .upsert_entity(EntityInput::new("alpha-beta", EntityKind::Library))
        .unwrap();

    let result = store.lookup_exact("alpha-gamma").unwrap();
    assert!(result.is_none());
}
