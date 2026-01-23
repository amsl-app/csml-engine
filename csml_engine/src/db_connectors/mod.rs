/**
 * This module defines the interactions between the CSML Manager and the underlying
 * database engines.
 *
 * There are several engines to choose from (see module features). To use one
 * of the database options, the `ENGINE_DB_TYPE` env var must be set
 * to one of the accepted values.
 *
 * To add a new DB type, please use one of the existing templates implementations.
 * Each method of each module must be fully reimplemented to extend the "generic"
 * implementation at the root of `db_connectors` directory.
 */
use crate::data::{Database, EngineError};

#[cfg(feature = "postgresql")]
pub(crate) mod postgresql;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;

use crate::error_messages::ERROR_DB_SETUP;
#[cfg(any(feature = "sqlite", feature = "postgresql"))]
use ::diesel::Connection;
#[cfg(feature = "postgresql")]
use ::diesel::PgConnection;
#[cfg(feature = "sqlite")]
use ::diesel::SqliteConnection;

#[cfg(feature = "postgresql")]
pub mod diesel;
pub mod utils;

#[cfg(feature = "postgresql")]
pub fn is_postgresql() -> bool {
    match std::env::var("ENGINE_DB_TYPE") {
        Ok(val) => val == *"postgresql",
        Err(_) => false,
    }
}

#[cfg(feature = "sqlite")]
pub fn is_sqlite() -> bool {
    match std::env::var("ENGINE_DB_TYPE") {
        Ok(val) => val == *"sqlite",
        Err(_) => false,
    }
}

pub fn make_migrations() -> Result<(), EngineError> {
    #[cfg(feature = "postgresql")]
    if is_postgresql() {
        let uri = std::env::var("POSTGRESQL_URL").unwrap_or_default();

        let mut pg_connection =
            PgConnection::establish(&uri).unwrap_or_else(|_| panic!("Error connecting to {uri}"));

        return postgresql::make_migrations(&mut pg_connection);
    }

    #[cfg(feature = "sqlite")]
    if is_sqlite() {
        let uri = std::env::var("SQLITE_URL").unwrap_or_default();
        let mut sqlite_connection = SqliteConnection::establish(&uri)
            .unwrap_or_else(|_| panic!("Error connecting to {uri}"));
        return sqlite::make_migrations(&mut sqlite_connection);
    }

    Ok(())
}

pub fn revert_migrations() -> Result<(), EngineError> {
    #[cfg(feature = "postgresql")]
    if is_postgresql() {
        let uri = std::env::var("POSTGRESQL_URL").unwrap_or_default();

        let mut pg_connection =
            PgConnection::establish(&uri).unwrap_or_else(|_| panic!("Error connecting to {uri}"));

        return postgresql::revert_migrations(&mut pg_connection);
    }

    #[cfg(feature = "sqlite")]
    if is_sqlite() {
        let uri = std::env::var("SQLITE_URL").unwrap_or_default();
        let mut sqlite_connection = SqliteConnection::establish(&uri)
            .unwrap_or_else(|_| panic!("Error connecting to {uri}"));
        return sqlite::revert_migrations(&mut sqlite_connection);
    }

    Ok(())
}

pub fn make_migrations_with_conn(conn: &mut Database) -> Result<(), EngineError> {
    tracing::debug!("running migrations");
    match conn {
        #[cfg(feature = "postgresql")]
        Database::Postgresql(conn) => postgresql::make_migrations(conn.client.as_mut()),
        #[cfg(feature = "sqlite")]
        Database::SqLite(conn) => sqlite::make_migrations(conn.client.as_mut()),
        Database::None(_) => Err(EngineError::Manager(ERROR_DB_SETUP.to_owned())),
    }
}

pub fn revert_migrations_with_conn(conn: &mut Database) -> Result<(), EngineError> {
    tracing::debug!("reverting migrations");
    match conn {
        #[cfg(feature = "postgresql")]
        Database::Postgresql(conn) => postgresql::revert_migrations(conn.client.as_mut()),
        #[cfg(feature = "sqlite")]
        Database::SqLite(conn) => sqlite::revert_migrations(conn.client.as_mut()),
        Database::None(_) => Err(EngineError::Manager(ERROR_DB_SETUP.to_owned())),
    }
}
