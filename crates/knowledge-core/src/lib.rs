/// Note capture helpers for lesson and issue entities.
pub mod capture;
/// Source import APIs for loading entities from JSON.
pub mod import;
/// Core domain enums for entities and relationships.
pub mod model;
/// Filesystem-backed note store utilities.
pub mod notes;
/// SQLite schema bootstrap for the knowledge store.
pub mod schema;
/// Query and mutation APIs for the knowledge store.
pub mod store;
