use crate::data::EngineError;
use crate::encrypt::{decrypt_data, encrypt_data};
use chrono::NaiveDateTime;
use csml_engine_entity::state;
use csml_interpreter::data::Client;
use sea_orm::IntoActiveValue;
use sea_orm::prelude::*;

pub async fn delete_state_key<C: ConnectionTrait>(
    client: &Client,
    type_: &str,
    key: &str,
    db: &C,
) -> Result<(), EngineError> {
    state::Entity::delete_many()
        .filter(state::Column::BotId.eq(&client.bot_id))
        .filter(state::Column::ChannelId.eq(&client.channel_id))
        .filter(state::Column::UserId.eq(&client.user_id))
        .filter(state::Column::Type.eq(type_))
        .filter(state::Column::Key.eq(key))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn get_state_key<C: ConnectionTrait>(
    client: &Client,
    type_: &str,
    key: &str,
    db: &C,
) -> Result<Option<serde_json::Value>, EngineError> {
    let state = state::Entity::find()
        .filter(state::Column::BotId.eq(&client.bot_id))
        .filter(state::Column::ChannelId.eq(&client.channel_id))
        .filter(state::Column::UserId.eq(&client.user_id))
        .filter(state::Column::Type.eq(type_))
        .filter(state::Column::Key.eq(key))
        .one(db)
        .await?;

    state.map(|state| decrypt_data(state.value)).transpose()
}

pub async fn set_state_items<C: ConnectionTrait>(
    client: &Client,
    type_: &str,
    keys_values: Vec<(&str, &serde_json::Value)>,
    expires_at: Option<NaiveDateTime>,
    db: &C,
) -> Result<(), EngineError> {
    if keys_values.is_empty() {
        return Ok(());
    }

    let new_states = keys_values
        .into_iter()
        .map(|(key, value)| {
            let value = encrypt_data(value)?;

            let state = state::ActiveModel {
                id: Uuid::new_v4().into_active_value(),
                bot_id: client.bot_id.clone().into_active_value(),
                channel_id: client.channel_id.clone().into_active_value(),
                user_id: client.user_id.clone().into_active_value(),
                type_: type_.to_string().into_active_value(),
                key: key.to_string().into_active_value(),
                value: value.into_active_value(),
                expires_at: expires_at.into_active_value(),
                ..Default::default()
            };
            Ok(state)
        })
        .collect::<Result<Vec<_>, EngineError>>()?;

    state::Entity::insert_many(new_states).exec(db).await?;

    Ok(())
}

pub async fn delete_user_state<C: ConnectionTrait>(
    conn: &C,
    client: &Client,
) -> Result<(), EngineError> {
    state::Entity::delete_many()
        .filter(state::Column::BotId.eq(&client.bot_id))
        .filter(state::Column::ChannelId.eq(&client.channel_id))
        .filter(state::Column::UserId.eq(&client.user_id))
        .exec(conn)
        .await?;

    Ok(())
}
