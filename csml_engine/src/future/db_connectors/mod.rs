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
use crate::data::{AsyncDatabase, EngineError};
use crate::error_messages::ERROR_DB_SETUP;
#[cfg(any(feature = "sea-orm", feature = "postgresql-async"))]
use crate::error_messages::ERROR_DB_URI;

#[cfg(feature = "postgresql-async")]
use self::postgresql as postgresql_connector;
#[cfg(feature = "sea-orm")]
use self::sea_orm as sea_orm_connector;

pub mod bot;
pub mod conversations;
pub mod memories;
pub mod messages;
pub mod state;

pub mod clean_db;
pub mod user;

pub mod db_test;

#[cfg(feature = "postgresql-async")]
pub(crate) mod postgresql;

#[cfg(feature = "sea-orm")]
pub(crate) mod sea_orm;

#[cfg(feature = "postgresql-async")]
pub fn get_postgresql_uri() -> Result<Option<String>, EngineError> {
    let Ok(engine_type) = std::env::var("ENGINE_DB_TYPE") else {
        return Ok(None);
    };
    if engine_type != "postgresql" {
        return Ok(None);
    }
    let uri = std::env::var("POSTGRESQL_URL")
        .map_err(|_| EngineError::Manager(ERROR_DB_URI.to_owned()))?;
    Ok(Some(uri))
}

#[cfg(feature = "sea-orm")]
pub fn get_seaorm_uri() -> Result<Option<String>, EngineError> {
    let Ok(engine_type) = std::env::var("ENGINE_DB_TYPE") else {
        return Ok(None);
    };
    let env_var = match engine_type.as_str() {
        "postgresql" => "POSTGRESQL_URL",
        "sqlite" => "SQLITE_URL",
        _ => return Ok(None),
    };

    let uri = std::env::var(env_var).map_err(|_| EngineError::Manager(ERROR_DB_URI.to_owned()))?;
    Ok(Some(uri))
}

pub async fn init_db() -> Result<AsyncDatabase<'static>, EngineError> {
    #[cfg(feature = "sea-orm")]
    if let Some(db) = get_seaorm_uri()? {
        return sea_orm_connector::init(&db).await;
    }

    #[cfg(feature = "postgresql-async")]
    if let Some(uri) = get_postgresql_uri()? {
        return postgresql_connector::init(&uri).await;
    }

    Err(EngineError::Manager(ERROR_DB_SETUP.to_owned()))
}
