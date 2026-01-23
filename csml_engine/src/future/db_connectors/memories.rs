#[cfg(feature = "postgresql-async")]
use crate::future::db_connectors::postgresql_connector;

#[cfg(feature = "sea-orm")]
use crate::future::db_connectors::sea_orm_connector;

use crate::Client;
use crate::data::{AsyncDatabase, ConversationInfo, EngineError, SeaOrmDbTraits};
use crate::db_connectors::utils::get_expires_at;
use csml_interpreter::data::Memory;

use std::collections::HashMap;

pub async fn add_memories<T: SeaOrmDbTraits, S: std::hash::BuildHasher>(
    data: &mut ConversationInfo,
    memories: &HashMap<String, Memory, S>,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(memories = ?memories.keys(), "saving memories");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::memories::add_memories(
                &data.client,
                memories,
                get_expires_at(data.ttl),
                db,
            )
            .await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::memories::add_memories(
                &data.client,
                memories,
                get_expires_at(data.ttl),
                db.db_ref(),
            )
            .await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn create_client_memory<T: SeaOrmDbTraits>(
    client: &Client,
    key: String,
    value: serde_json::Value,
    ttl: Option<chrono::Duration>,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(?client, %key, ?value, "saving memory");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::memories::create_client_memory(
                client,
                &key,
                &value,
                get_expires_at(ttl),
                db,
            )
            .await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::memories::create_client_memory(
                client,
                &key,
                &value,
                get_expires_at(ttl),
                db.db_ref(),
            )
            .await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn internal_use_get_memories<T: SeaOrmDbTraits>(
    client: &Client,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<serde_json::Value, EngineError> {
    tracing::debug!(?client, "getting memories (internal)");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::memories::internal_use_get_memories(client, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::memories::internal_use_get_memories(client, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

/**
 * Get client Memories
 */
pub async fn get_memories<T: SeaOrmDbTraits>(
    client: &Client,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<serde_json::Value, EngineError> {
    tracing::debug!(?client, "getting memories");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::memories::get_memories(client, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::memories::get_memories(db.db_ref(), client).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

/**
 * Get client Memory
 */
pub async fn get_memory<T: SeaOrmDbTraits>(
    client: &Client,
    key: &str,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<serde_json::Value, EngineError> {
    tracing::debug!(key, ?client, "db call get memory");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::memories::get_memory(client, key, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::memories::get_memory(db.db_ref(), client, key).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn delete_client_memory<T: SeaOrmDbTraits>(
    client: &Client,
    key: &str,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(key, ?client, "db call delete memory");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::memories::delete_client_memory(client, key, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::memories::delete_client_memory(client, key, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn delete_client_memories<T: SeaOrmDbTraits>(
    client: &Client,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::info!(?client, "db call delete memories");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::memories::delete_client_memories(client, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::memories::delete_client_memories(client, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}
