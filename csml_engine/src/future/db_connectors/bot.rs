#[cfg(feature = "postgresql-async")]
use crate::future::db_connectors::postgresql_connector;
#[cfg(feature = "sea-orm")]
use crate::future::db_connectors::sea_orm_connector;

use uuid::Uuid;

use crate::data::{AsyncDatabase, EngineError, SeaOrmDbTraits};
use crate::error_messages::ERROR_DB_SETUP;
use crate::models::BotVersion;
use csml_interpreter::data::CsmlBot;

pub async fn create_bot_version<T: SeaOrmDbTraits>(
    bot_id: String,
    csml_bot: CsmlBot,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<String, EngineError> {
    tracing::debug!(bot_id, ?csml_bot, "db call create bot version");

    let serializable_bot = crate::data::to_serializable_bot(&csml_bot);
    let bot = serde_json::json!(serializable_bot).to_string();

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            let version_id = postgresql_connector::bot::create_bot_version(bot_id, bot, db).await?;

            Ok(version_id)
        }
        AsyncDatabase::SeaOrm(db) => {
            let version_id =
                sea_orm_connector::bot::create_bot_version(db.db_ref(), bot_id, bot).await?;

            Ok(version_id.to_string())
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn get_last_bot_version<T: SeaOrmDbTraits>(
    bot_id: &str,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<Option<BotVersion>, EngineError> {
    tracing::debug!(bot_id, "db call get last bot version");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::bot::get_last_bot_version(bot_id, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::bot::get_last_bot_version(bot_id, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn get_by_version_id<T: SeaOrmDbTraits>(
    version_id: Uuid,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<Option<BotVersion>, EngineError> {
    tracing::debug!(%version_id, "db call get by version id");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::bot::get_bot_by_version_id(version_id, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::bot::get_bot_by_version_id(version_id, db.db_ref()).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn get_bot_versions<T: SeaOrmDbTraits>(
    bot_id: &str,
    limit: Option<u32>,
    pagination_key: Option<u32>,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<serde_json::Value, EngineError> {
    tracing::debug!(bot_id, ?limit, ?pagination_key, "db call get bot versions");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::bot::get_bot_versions(bot_id, limit, pagination_key, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::bot::get_bot_versions(
                db.db_ref(),
                bot_id,
                limit.map(u64::from),
                pagination_key.map(u64::from),
            )
            .await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn delete_bot_version<T: SeaOrmDbTraits>(
    _bot_id: &str,
    version_id: &str,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(version_id, "db call delete bot version");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::bot::delete_bot_version(version_id, db).await
        }
        _ => Err(EngineError::Manager(ERROR_DB_SETUP.to_owned())),
    }
}

pub async fn delete_bot_versions<T: SeaOrmDbTraits>(
    bot_id: &str,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(bot_id, "db call delete bot versions");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::bot::delete_bot_versions(bot_id, db).await
        }
        #[cfg(feature = "sea-orm")]
        AsyncDatabase::SeaOrm(db) => {
            sea_orm_connector::bot::delete_bot_versions(db.db_ref(), bot_id).await
        }
        #[cfg(not(feature = "sea-orm"))]
        AsyncDatabase::_Impossible(_, _) => unreachable!(),
    }
}

pub async fn delete_all_bot_data<T: SeaOrmDbTraits>(
    bot_id: &str,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(bot_id, "db call delete all bot data");

    match db {
        #[cfg(feature = "postgresql-async")]
        AsyncDatabase::Postgresql(db) => {
            postgresql_connector::bot::delete_bot_versions(bot_id, db).await?;
            postgresql_connector::conversations::delete_all_bot_data(bot_id, db).await?;
            postgresql_connector::memories::delete_all_bot_data(bot_id, db).await?;
            postgresql_connector::state::delete_all_bot_data(bot_id, db).await?;
            Ok(())
        }
        _ => Err(EngineError::Manager(ERROR_DB_SETUP.to_owned())),
    }
}
