use crate::data::{EngineError, SerializeCsmlBot};
use crate::models::BotVersion;
use csml_engine_entity::bot_version;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveValue, Order, QueryOrder};
use std::convert::TryFrom;

pub async fn get_last_bot_version<C: ConnectionTrait>(
    bot_id: &str,
    db: &C,
) -> Result<Option<BotVersion>, EngineError> {
    let bot_version = bot_version::Entity::find()
        .filter(bot_version::Column::BotId.eq(bot_id))
        .order_by(bot_version::Column::CreatedAt, Order::Desc)
        .one(db)
        .await?;

    bot_version.map(BotVersion::try_from).transpose()
}

pub async fn get_bot_by_version_id<C: ConnectionTrait>(
    id: Uuid,
    db: &C,
) -> Result<Option<BotVersion>, EngineError> {
    let bot_version = bot_version::Entity::find_by_id(id).one(db).await?;

    bot_version.map(BotVersion::try_from).transpose()
}

pub async fn get_bot_versions<C: ConnectionTrait>(
    conn: &C,
    bot_id: &str,
    limit: Option<u64>,
    pagination_key: Option<u64>,
) -> Result<serde_json::Value, EngineError> {
    let page = pagination_key.unwrap_or(0);
    let limit_per_page = limit.unwrap_or(25).min(25);

    let bot_id = bot_id.to_owned();
    let query = bot_version::Entity::find()
        .filter(bot_version::Column::BotId.eq(bot_id))
        .order_by(bot_version::Column::CreatedAt, Order::Asc)
        .paginate(conn, limit_per_page);

    let bot_versions = query.fetch_page(page).await?;
    let total_pages = query.num_pages().await?;

    let bots: Vec<_> = bot_versions
        .into_iter()
        .map(|bot_version| {
            let csml_bot: SerializeCsmlBot = serde_json::from_str(&bot_version.bot).unwrap();

            let mut json = serde_json::json!({
                "version_id": bot_version.id,
                "id": csml_bot.id,
                "name": csml_bot.name,
                "default_flow": csml_bot.default_flow,
                "engine_version": bot_version.engine_version,
                "created_at": bot_version.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string()
            });

            if let Some(custom_components) = csml_bot.custom_components {
                json["custom_components"] = serde_json::json!(custom_components);
            }
            json
        })
        .collect();

    if page < total_pages {
        let pagination_key = (page + 1).to_string();
        Ok(serde_json::json!({ "bots": bots, "pagination_key": pagination_key }))
    } else {
        Ok(serde_json::json!({ "bots": bots }))
    }
}

pub async fn create_bot_version<C: ConnectionTrait>(
    conn: &C,
    bot_id: String,
    bot: String,
) -> Result<Uuid, EngineError> {
    let new_bot = bot_version::ActiveModel {
        id: Uuid::new_v4().into_active_value(),
        bot_id: bot_id.into_active_value(),
        bot: bot.into_active_value(),
        engine_version: env!("CARGO_PKG_VERSION").to_owned().into_active_value(),
        updated_at: ActiveValue::NotSet,
        created_at: ActiveValue::NotSet,
    };

    let bot_version = bot_version::Entity::insert(new_bot).exec(conn).await?;

    Ok(bot_version.last_insert_id)
}

pub async fn delete_bot_versions<C: ConnectionTrait>(
    conn: &C,
    bot_id: &str,
) -> Result<(), EngineError> {
    bot_version::Entity::delete_many()
        .filter(bot_version::Column::BotId.eq(bot_id))
        .exec(conn)
        .await?;

    Ok(())
}
