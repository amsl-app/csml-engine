use crate::data::EngineError;
use crate::data::models::{Conversation, Paginated, PaginationData};
use crate::future::db_connectors::sea_orm::util::filter_by_client;
use chrono::NaiveDateTime;
use csml_engine_db::conversation::Mutation;
use csml_engine_entity::conversation;
use csml_engine_entity::conversation::{Model, Status};
use csml_interpreter::data::Client;
use num_traits::ToPrimitive;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, IntoActiveValue, QueryOrder, QuerySelect};
use std::error::Error;
use uuid::Uuid;

pub async fn create_conversation<C: ConnectionTrait>(
    flow_id: &str,
    step_id: &str,
    client: &Client,
    expires_at: Option<NaiveDateTime>,
    db: &C,
) -> Result<Uuid, EngineError> {
    let new_conversation = conversation::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        bot_id: ActiveValue::Set(client.bot_id.clone()),
        channel_id: ActiveValue::Set(client.channel_id.clone()),
        user_id: ActiveValue::Set(client.user_id.clone()),
        flow_id: ActiveValue::Set(flow_id.to_string()),
        step_id: ActiveValue::Set(step_id.to_string()),
        status: ActiveValue::Set(Status::Open),
        expires_at: expires_at.into_active_value(),
        ..Default::default()
    };

    let conversation = new_conversation.insert(db).await?;

    Ok(conversation.id)
}

pub async fn close_all_conversations<C: ConnectionTrait>(
    client: &Client,
    db: &C,
) -> Result<(), EngineError> {
    let changes = conversation::ActiveModel {
        status: ActiveValue::Set(Status::Closed),
        ..Default::default()
    };
    filter_by_client(conversation::Entity::update_many(), client)
        .set(changes)
        .exec(db)
        .await?;

    Ok(())
}

pub async fn get_latest_open<C: ConnectionTrait>(
    client: &Client,
    db: &C,
) -> Result<Option<Conversation>, EngineError> {
    let result = filter_by_client(conversation::Entity::find(), client)
        .filter(conversation::Column::Status.eq(Status::Open))
        .order_by_desc(conversation::Column::UpdatedAt)
        .one(db)
        .await?;

    result.map(|conv| Ok(conv.into())).transpose()
}

pub async fn update_conversation<C: ConnectionTrait>(
    conversation_id: Uuid,
    flow_id: Option<&str>,
    step_id: Option<&str>,
    db: &C,
) -> Result<(), EngineError> {
    if flow_id.is_none() && step_id.is_none() {
        return Ok(());
    }

    let active_model = conversation::ActiveModel {
        id: ActiveValue::Set(conversation_id),
        flow_id: flow_id
            .map(|flow_id| ActiveValue::Set(flow_id.to_string()))
            .unwrap_or_default(),
        step_id: step_id
            .map(|step_id| ActiveValue::Set(step_id.to_string()))
            .unwrap_or_default(),
        ..Default::default()
    };
    conversation::Entity::update(active_model).exec(db).await?;

    Ok(())
}

pub async fn close_conversation<C: ConnectionTrait>(id: Uuid, db: &C) -> Result<(), EngineError> {
    let res = Mutation::set_status(db, id, Status::Closed).await;
    if let Err(error) = res {
        tracing::error!(error = &error as &dyn Error, %id, "error trying  to close conversation");
        return Err(error.into());
    }

    Ok(())
}

pub async fn get_conversation<C: ConnectionTrait>(
    db: &C,
    id: Uuid,
) -> Result<Conversation, EngineError> {
    let conversation = conversation::Entity::find_by_id(id).one(db).await?;

    let conversation = conversation.ok_or(EngineError::SqlErrorCode(format!(
        "Conversation {id} not found"
    )))?;

    Ok(conversation.into())
}

async fn get_client_conversations_inner<C: ConnectionTrait>(
    conn: &C,
    client: &Client,
    limit: Option<u32>,
    pagination_key: Option<u32>,
) -> Result<(Vec<Model>, Option<PaginationData>), EngineError> {
    let query = filter_by_client(conversation::Entity::find(), client);

    let (Some(limit), Some(pagination_key)) = (limit, pagination_key) else {
        let res = query.limit(limit.map(u64::from)).all(conn).await?;
        return Ok((res, None));
    };

    let pagination = query.paginate(conn, u64::from(limit));
    let page = pagination.fetch_page(u64::from(pagination_key)).await?;
    let page_count = pagination.num_pages().await?;

    let pagination = PaginationData {
        page: pagination_key,
        total_pages: page_count.to_u32().ok_or_else(|| {
            EngineError::Internal(format!(
                "can't convert page_count value ({page_count}) to u32"
            ))
        })?,
        per_page: limit,
    };

    Ok((page, Some(pagination)))
}

pub async fn get_client_conversations<C: ConnectionTrait>(
    conn: &C,
    client: &Client,
    limit: Option<u32>,
    pagination_key: Option<u32>,
) -> Result<Paginated<Conversation>, EngineError> {
    let (models, pagination) = get_client_conversations_inner(conn, client, limit, pagination_key)
        .await
        .inspect_err(|error| {
            tracing::error!(
                error = error as &dyn Error,
                "error trying to get client conversations"
            );
        })?;

    let data = models.into_iter().map(Into::into).collect();
    Ok(Paginated { data, pagination })
}

pub async fn delete_user_conversations<C: ConnectionTrait>(
    conn: &C,
    client: &Client,
) -> Result<(), EngineError> {
    filter_by_client(conversation::Entity::delete_many(), client)
        .exec(conn)
        .await?;

    Ok(())
}
