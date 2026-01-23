use crate::data::{AsyncDatabase, EngineError, SeaOrmClient};
use sea_orm::Database;

pub mod bot;
pub mod conversations;
pub mod expired_data;
pub mod memories;
pub mod messages;
pub mod models;
pub mod state;
pub mod util;

pub(crate) async fn init(uri: &str) -> Result<AsyncDatabase<'static>, EngineError> {
    let connection = Database::connect(uri).await;
    match connection {
        Ok(connection) => {
            let db = AsyncDatabase::SeaOrm(SeaOrmClient::new(connection));
            Ok(db)
        }
        Err(error) => Err(EngineError::Manager(error.to_string())),
    }
}
