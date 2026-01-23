pub mod models;
pub mod schema;

use crate::data::EngineError;
use diesel::prelude::PgConnection;
use diesel_migrations::{EmbeddedMigrations, HarnessWithOutput, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/postgresql");
pub fn make_migrations(conn: &mut PgConnection) -> Result<(), EngineError> {
    tracing::debug!("running migrations for PostgreSQL");
    let mut harness = HarnessWithOutput::write_to_stdout(conn);
    harness.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

pub fn revert_migrations(conn: &mut PgConnection) -> Result<(), EngineError> {
    tracing::debug!("reverting migrations for PostgreSQL");
    let mut harness = HarnessWithOutput::write_to_stdout(conn);
    harness.revert_all_migrations(MIGRATIONS)?;

    Ok(())
}
