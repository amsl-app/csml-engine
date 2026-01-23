use crate::data::EngineError;
use crate::encrypt::{decrypt_data, encrypt_data};
use chrono::NaiveDateTime;
use csml_engine_entity::memory;
use csml_engine_entity::memory::Model;
use csml_interpreter::data::{Client, Memory};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveValue, QueryFilter};
use serde_json::{Value, json};
use std::collections::HashMap;

fn create_on_conflict_update_value() -> OnConflict {
    OnConflict::columns([
        memory::Column::BotId,
        memory::Column::ChannelId,
        memory::Column::UserId,
        memory::Column::Key,
    ])
    .update_column(memory::Column::Value)
    .clone()
}

pub async fn add_memories<C: ConnectionTrait, S: std::hash::BuildHasher>(
    client: &Client,
    memories: &HashMap<String, Memory, S>,
    expires_at: Option<NaiveDateTime>,
    db: &C,
) -> Result<(), EngineError> {
    if memories.is_empty() {
        return Ok(());
    }

    let new_memories = memories
        .iter()
        .map(|(key, mem)| {
            let value = encrypt_data(&mem.value)?;
            let new_mem = memory::ActiveModel {
                id: uuid::Uuid::new_v4().into_active_value(),
                bot_id: client.bot_id.clone().into_active_value(),
                channel_id: client.channel_id.clone().into_active_value(),
                user_id: client.user_id.clone().into_active_value(),
                key: key.clone().into_active_value(),
                value: value.into_active_value(),
                expires_at: expires_at.into_active_value(),
                ..Default::default()
            };
            Ok(new_mem)
        })
        .collect::<Result<Vec<_>, EngineError>>()?;
    memory::Entity::insert_many(new_memories)
        .on_conflict(create_on_conflict_update_value())
        .exec(db)
        .await?;

    Ok(())
}

pub async fn create_client_memory<C: ConnectionTrait>(
    client: &Client,
    key: &str,
    value: &serde_json::Value,
    expires_at: Option<NaiveDateTime>,
    db: &C,
) -> Result<(), EngineError> {
    let value = encrypt_data(value)?;
    let new_memory = memory::ActiveModel {
        id: uuid::Uuid::new_v4().into_active_value(),
        bot_id: client.bot_id.clone().into_active_value(),
        channel_id: client.channel_id.clone().into_active_value(),
        user_id: client.user_id.clone().into_active_value(),
        key: key.to_string().into_active_value(),
        value: value.into_active_value(),
        expires_at: expires_at.into_active_value(),
        ..Default::default()
    };
    memory::Entity::insert(new_memory)
        .on_conflict(create_on_conflict_update_value())
        .exec(db)
        .await?;

    Ok(())
}

pub async fn internal_use_get_memories<C: ConnectionTrait>(
    client: &Client,
    db: &C,
) -> Result<serde_json::Value, EngineError> {
    let memories = memory::Entity::find()
        .filter(memory::Column::BotId.eq(&client.bot_id))
        .filter(memory::Column::ChannelId.eq(&client.channel_id))
        .filter(memory::Column::UserId.eq(&client.user_id))
        .all(db)
        .await?;

    let mut map = serde_json::Map::new();
    for mem in memories {
        if !map.contains_key(&mem.key) {
            let value: serde_json::Value = decrypt_data(mem.value)?;
            map.insert(mem.key, value);
        }
    }

    Ok(serde_json::json!(map))
}

pub async fn get_memory<C: ConnectionTrait>(
    db: &C,
    client: &Client,
    key: &str,
) -> Result<Value, EngineError> {
    let memory = memory::Entity::find()
        .filter(memory::Column::Key.eq(key))
        .filter(memory::Column::BotId.eq(&client.bot_id))
        .filter(memory::Column::ChannelId.eq(&client.channel_id))
        .filter(memory::Column::UserId.eq(&client.user_id))
        .one(db)
        .await?;

    let Some(memory) = memory else {
        return Err(DbErr::RecordNotFound(format!("Memory {key} not found")).into());
    };

    model_to_value(memory)
}

pub async fn get_memories<C: ConnectionTrait>(
    db: &C,
    client: &Client,
) -> Result<Value, EngineError> {
    let memory = memory::Entity::find()
        .filter(memory::Column::BotId.eq(&client.bot_id))
        .filter(memory::Column::ChannelId.eq(&client.channel_id))
        .filter(memory::Column::UserId.eq(&client.user_id))
        .all(db)
        .await?;

    memory
        .into_iter()
        .map(model_to_value)
        .collect::<Result<_, _>>()
}

fn model_to_value(memory: Model) -> Result<Value, EngineError> {
    let value = decrypt_data(memory.value)?;
    Ok(json!({
        "key": memory.key,
        "value": value,
        "created_at": memory.created_at.to_string()
    }))
}

pub async fn delete_client_memory<C: ConnectionTrait>(
    client: &Client,
    key: &str,
    db: &C,
) -> Result<(), EngineError> {
    memory::Entity::delete_many()
        .filter(memory::Column::Key.eq(key))
        .filter(memory::Column::BotId.eq(&client.bot_id))
        .filter(memory::Column::ChannelId.eq(&client.channel_id))
        .filter(memory::Column::UserId.eq(&client.user_id))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn delete_client_memories<C: ConnectionTrait>(
    client: &Client,
    db: &C,
) -> Result<(), EngineError> {
    memory::Entity::delete_many()
        .filter(memory::Column::BotId.eq(&client.bot_id))
        .filter(memory::Column::ChannelId.eq(&client.channel_id))
        .filter(memory::Column::UserId.eq(&client.user_id))
        .exec(db)
        .await?;

    Ok(())
}
