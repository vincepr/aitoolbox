/// Mutation history and idempotency helpers.
pub mod audit;
/// Note capture helpers for lesson and issue entities.
pub mod capture;
/// Typed runtime configuration resolution and validation.
pub mod config;
/// Source import APIs for loading entities from JSON.
pub mod import;
/// Ordered schema migration definitions.
pub mod migrations;
/// Core domain enums for entities and relationships.
pub mod model;
/// Filesystem-backed note store utilities.
pub mod notes;
/// SQLite schema bootstrap and version verification for the knowledge store.
pub mod schema;
/// Query and mutation APIs for the knowledge store.
pub mod store;
