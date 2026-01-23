#[cfg(feature = "postgresql-async")]
use crate::future::db_connectors::postgresql_connector;
#[cfg(feature = "sea-orm")]
use crate::future::db_connectors::sea_orm_connector;

use uuid::Uuid;

use crate::data::models::Conversation;

use crate::db_connectors::utils::get_expires_at;

use crate::data::{AsyncDatabase, EngineError, SeaOrmDbTraits};

use crate::future::db_connectors::state;
use crate::{Client, data};
use std::error::Error;

pub async fn create_conversation<T: SeaOrmDbTraits>(
    flow_id: &str,
    step_id: &str,
    client: &Client,
    ttl: Option<chrono::Duration>,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<Uuid, EngineError> {
    tracing::debug!(?client, %flow_id, %step_id, "creating conversation");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::conversations::create_conversation(
                flow_id,
                step_id,
                client,
                get_expires_at(ttl),
                db,
            )
            .await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::conversations::create_conversation(
                flow_id,
                step_id,
                client,
                get_expires_at(ttl),
                db.db_ref(),
            )
            .await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn close_conversation<T: SeaOrmDbTraits>(
    id: Uuid,
    client: &Client,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(%id, ?client, "closing conversation");

    // delete previous bot info at the end of the conversation
    state::delete_state_key(client, "bot", "previous", db).await?;

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::conversations::close_conversation(id, "CLOSED", db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::conversations::close_conversation(id, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn close_all_conversations<T: SeaOrmDbTraits>(
    client: &Client,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(?client, "closing all conversations");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::conversations::close_all_conversations(client, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::conversations::close_all_conversations(client, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn get_latest_open<T: SeaOrmDbTraits>(
    client: &Client,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<Option<Conversation>, EngineError> {
    tracing::debug!(?client, "getting latest open conversation");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::conversations::get_latest_open(client, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::conversations::get_latest_open(client, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn update_conversation<T: SeaOrmDbTraits>(
    conversation_id: Uuid,
    flow_id: Option<&str>,
    step_id: Option<&str>,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(
        %conversation_id, ?flow_id, ?step_id, "updating conversation"
    );

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::conversations::update_conversation(
                conversation_id,
                flow_id,
                step_id,
                db,
            )
            .await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::conversations::update_conversation(
                conversation_id,
                flow_id,
                step_id,
                db.db_ref(),
            )
            .await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn get_conversation<T: SeaOrmDbTraits>(
    db: &mut AsyncDatabase<'_, T>,
    conversation_id: Uuid,
) -> Result<Conversation, EngineError> {
    tracing::debug!(%conversation_id, "loading conversation");
    let res = match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::conversations::get_conversation(db, conversation_id).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::conversations::get_conversation(db.db_ref(), conversation_id).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    };

    if let Err(error) = &res {
        tracing::error!(error = error as &dyn Error, %conversation_id, "error loading conversation");
    }

    res
}

pub async fn get_client_conversations<T: SeaOrmDbTraits>(
    client: &Client,
    db: &mut AsyncDatabase<'_, T>,
    limit: Option<u32>,
    pagination_key: Option<u32>,
) -> Result<data::models::Paginated<Conversation>, EngineError> {
    tracing::debug!(
        ?client,
        ?limit,
        ?pagination_key,
        "getting client conversations"
    );

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::conversations::get_client_conversations(
                client,
                db,
                limit,
                pagination_key,
            )
            .await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::conversations::get_client_conversations(
                db.db_ref(),
                client,
                limit,
                pagination_key,
            )
            .await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}
