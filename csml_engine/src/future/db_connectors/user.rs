#[cfg(feature = "postgresql-async")]
use crate::future::db_connectors::postgresql_connector;

use crate::Client;
use crate::data::{AsyncDatabase, EngineError, SeaOrmDbTraits};
use crate::future::db_connectors::sea_orm_connector;

pub async fn delete_client<T: SeaOrmDbTraits>(
    client: &Client,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(?client, "db call delete client");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::conversations::delete_user_conversations(client, db).await?;
            postgresql_connector::memories::delete_client_memories(client, db).await?;
            postgresql_connector::messages::delete_user_messages(client, db).await?;
            postgresql_connector::state::delete_user_state(client, db).await?;
            Ok(())
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::conversations::delete_user_conversations(db.db_ref(), client)
                .await?;
            sea_orm_connector::memories::delete_client_memories(client, db.db_ref()).await?;
            sea_orm_connector::messages::delete_user_messages(db.db_ref(), client).await?;
            sea_orm_connector::state::delete_user_state(db.db_ref(), client).await?;
            Ok(())
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}
