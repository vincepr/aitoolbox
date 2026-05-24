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
            EntityInput::new("MyCompanyName.Ebay.Custom.Client", EntityKind::Library)
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

    assert_eq!(
        result.entity.canonical_name,
        "MyCompanyName.Ebay.Custom.Client"
    );
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
