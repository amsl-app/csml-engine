pub mod bot;
pub mod conversations;
pub mod memories;
pub mod messages;
pub mod state;

pub mod pagination;

pub mod expired_data;

use crate::data::{AsyncDatabase, AsyncPostgresqlClient, EngineError};
use diesel_async::{AsyncConnection, AsyncPgConnection};

pub async fn init(uri: &str) -> Result<AsyncDatabase<'static>, EngineError> {
    tracing::debug!(%uri, "connecting to PostgreSQL database");
    let pg_connection = AsyncPgConnection::establish(uri)
        .await
        .unwrap_or_else(|_| panic!("Error connecting to {uri}"));

    let db = AsyncDatabase::Postgresql(AsyncPostgresqlClient::new(pg_connection));
    Ok(db)
}
