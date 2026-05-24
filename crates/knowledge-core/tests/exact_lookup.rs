use knowledge_core::model::{EntityKind, RelationshipKind};
use knowledge_core::schema::bootstrap;
use knowledge_core::store::{EntityInput, KnowledgeStore};
use rusqlite::Connection;

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
