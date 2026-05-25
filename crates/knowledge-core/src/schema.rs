use anyhow::Result;
use rusqlite::Connection;

use crate::migrations::{apply_all, current_schema_version, latest_migration_version};

/// Applies all known migrations for the knowledge schema.
///
/// # Arguments
///
/// * `conn` - Open SQLite connection to migrate.
///
/// # Returns
///
/// `Ok(())` when the latest schema is installed.
///
/// # Errors
///
/// Returns an error when migration SQL fails.
pub fn bootstrap(conn: &Connection) -> Result<()> {
    apply_all(conn)
}

/// Verifies that the connected database is on the latest supported schema version.
///
/// # Arguments
///
/// * `conn` - Open SQLite connection to verify.
///
/// # Returns
///
/// `Ok(())` when schema version exactly matches the latest migration version.
///
/// # Errors
///
/// Returns an error if the schema is behind or ahead of supported migrations.
pub fn verify_schema(conn: &Connection) -> Result<()> {
    let current = current_schema_version(conn)?;
    let latest = latest_migration_version();
    if current != latest {
        anyhow::bail!("schema version mismatch: current={current} latest={latest}");
    }
    Ok(())
}

/// Returns the current schema version for the connected database.
///
/// # Arguments
///
/// * `conn` - Open SQLite connection.
///
/// # Returns
///
/// Current schema version.
///
/// # Errors
///
/// Returns an error when the migration ledger cannot be read.
pub fn schema_version(conn: &Connection) -> Result<i64> {
    current_schema_version(conn)
}
