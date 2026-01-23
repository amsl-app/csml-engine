#[cfg(feature = "postgresql-async")]
use crate::future::db_connectors::postgresql_connector;
#[cfg(feature = "sea-orm")]
use crate::future::db_connectors::sea_orm_connector;

use uuid::Uuid;

use crate::data::filter::ClientMessageFilter;
use crate::data::models::{Direction, Message, Paginated};

use crate::db_connectors::utils::get_expires_at;

use crate::data::{AsyncDatabase, ConversationInfo, EngineError, SeaOrmDbTraits};
use std::error::Error;

pub async fn add_messages_bulk<T: SeaOrmDbTraits>(
    data: &mut ConversationInfo,
    msgs: Vec<serde_json::Value>,
    interaction_order: u32,
    direction: Direction,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(client = ?data.client, messages = ?msgs, "saving messages");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::messages::add_messages_bulk(
                data,
                &msgs,
                interaction_order,
                direction.into(),
                get_expires_at(data.ttl),
                db,
            )
            .await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::messages::add_messages_bulk(
                data,
                &msgs,
                interaction_order,
                direction.into(),
                get_expires_at(data.ttl),
                db.db_ref(),
            )
            .await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn get_client_messages<'a, 'conn: 'a, 'b, T: SeaOrmDbTraits>(
    db: &'a mut AsyncDatabase<'conn, T>,
    filter: ClientMessageFilter<'b>,
) -> Result<Paginated<Message>, EngineError> {
    let ClientMessageFilter { client, .. } = filter;
    tracing::debug!(?client, "loading client messages");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::messages::get_client_messages(db, filter).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::messages::get_client_messages(db.db_ref(), filter).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn get_conversation_messages<'a, 'conn: 'a, T: SeaOrmDbTraits>(
    db: &'a mut AsyncDatabase<'conn, T>,
    conversation_id: Uuid,
) -> Result<Vec<Message>, EngineError> {
    tracing::debug!(%conversation_id, "loading conversation messages");

    let res = match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::messages::get_conversation_messages(db, conversation_id).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::messages::get_conversation_messages(db.db_ref(), conversation_id)
                .await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    };

    if let Err(error) = &res {
        tracing::error!(error = error as &dyn Error, %conversation_id, "error loading conversation messages");
    }
    res
}
