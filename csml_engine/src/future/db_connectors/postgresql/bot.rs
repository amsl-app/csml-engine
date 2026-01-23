use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use std::convert::TryFrom;

use super::pagination::Paginate;
use crate::db_connectors::postgresql::{models, schema::csml_bot_versions};

use crate::data::SerializeCsmlBot;
use crate::data::{AsyncPostgresqlClient, EngineError};
use crate::models::BotVersion;
use std::env;
use uuid::Uuid;

pub async fn create_bot_version(
    bot_id: String,
    bot: String,
    db: &mut AsyncPostgresqlClient<'_>,
) -> Result<String, EngineError> {
    let new_bot = models::NewBot {
        id: Uuid::new_v4(),
        bot_id: &bot_id,
        bot: &bot,
        engine_version: env!("CARGO_PKG_VERSION"),
    };

    let bot: models::Bot = diesel::insert_into(csml_bot_versions::table)
        .values(&new_bot)
        .get_result(db.client.as_mut())
        .await?;

    Ok(bot.id.to_string())
}

pub async fn get_bot_versions(
    bot_id: &str,
    limit: Option<u32>,
    pagination_key: Option<u32>,
    db: &mut AsyncPostgresqlClient<'_>,
) -> Result<serde_json::Value, EngineError> {
    let pagination_key = pagination_key.unwrap_or(0);

    let bot_id = bot_id.to_owned();
    let mut query = csml_bot_versions::table
        .order_by(csml_bot_versions::updated_at.desc())
        .filter(csml_bot_versions::bot_id.eq(bot_id))
        .paginate(pagination_key);

    let limit_per_page = limit.unwrap_or(25).min(25);
    query = query.per_page(limit_per_page);
    let (bot_versions, total_pages) = query
        .load_and_count_pages::<models::Bot>(db.client.as_mut())
        .await?;

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

    if pagination_key < total_pages {
        let pagination_key = (pagination_key + 1).to_string();
        Ok(serde_json::json!({ "bots": bots, "pagination_key": pagination_key }))
    } else {
        Ok(serde_json::json!({ "bots": bots }))
    }
}

pub async fn get_bot_by_version_id(
    id: Uuid,
    db: &mut AsyncPostgresqlClient<'_>,
) -> Result<Option<BotVersion>, EngineError> {
    let bot_version: Option<models::Bot> = csml_bot_versions::table
        .filter(csml_bot_versions::id.eq(&id))
        .get_result(db.client.as_mut())
        .await
        .optional()?;

    bot_version.map(BotVersion::try_from).transpose()
}

pub async fn get_last_bot_version(
    bot_id: &str,
    db: &mut AsyncPostgresqlClient<'_>,
) -> Result<Option<BotVersion>, EngineError> {
    let bot_version: Option<models::Bot> = csml_bot_versions::table
        .filter(csml_bot_versions::bot_id.eq(&bot_id))
        .order_by(csml_bot_versions::created_at.desc())
        .get_result(db.client.as_mut())
        .await
        .optional()?;

    bot_version.map(BotVersion::try_from).transpose()
}

pub async fn delete_bot_version(
    version_id: &str,
    db: &mut AsyncPostgresqlClient<'_>,
) -> Result<(), EngineError> {
    let Ok(id) = Uuid::parse_str(version_id) else {
        // TODO Handle this properly
        tracing::error!(
            version_id,
            "failed to parse version_id - ignoring delete request"
        );
        return Ok(());
    };

    diesel::delete(csml_bot_versions::table.filter(csml_bot_versions::id.eq(id)))
        .get_result::<models::Bot>(db.client.as_mut())
        .await
        .ok();

    Ok(())
}

pub async fn delete_bot_versions(
    bot_id: &str,
    db: &mut AsyncPostgresqlClient<'_>,
) -> Result<(), EngineError> {
    diesel::delete(csml_bot_versions::table.filter(csml_bot_versions::bot_id.eq(bot_id)))
        .get_result::<models::Bot>(db.client.as_mut())
        .await
        .ok();

    Ok(())
}
