use crate::data::EngineError;
use diesel::prelude::*;
use diesel_migrations::embed_migrations;
use diesel_migrations::{EmbeddedMigrations, HarnessWithOutput, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/sqlite");

pub fn make_migrations(conn: &mut SqliteConnection) -> Result<(), EngineError> {
    tracing::debug!("running migrations for SQLite");
    let mut harness = HarnessWithOutput::write_to_stdout(conn);
    harness.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

pub fn revert_migrations(conn: &mut SqliteConnection) -> Result<(), EngineError> {
    tracing::debug!("reverting migrations for SQLite");
    let mut harness = HarnessWithOutput::write_to_stdout(conn);
    harness.revert_all_migrations(MIGRATIONS)?;

    Ok(())
}
