use crate::data::filter::ClientMessageFilter;
use crate::data::models::{Message, PaginationData};
use crate::data::{ConversationInfo, EngineError, models};
use crate::encrypt::encrypt_data;
use chrono::{DateTime, NaiveDateTime};
use csml_engine_db::messages::Query;
use csml_engine_entity::conversation;
use csml_engine_entity::messages;
use csml_engine_entity::messages::Direction;
use csml_interpreter::data::Client;
use sea_orm::prelude::*;

use crate::future::db_connectors::sea_orm::util::filter_by_client;
use num_traits::ToPrimitive;
use sea_orm::{
    ActiveValue, ConnectionTrait, EntityTrait, IntoActiveValue, PaginatorTrait, QueryOrder,
};
use std::convert::TryFrom;

pub async fn add_messages_bulk<C: ConnectionTrait>(
    data: &mut ConversationInfo,
    msgs: &[serde_json::Value],
    interaction_order: u32,
    direction: Direction,
    expires_at: Option<NaiveDateTime>,
    db: &C,
) -> Result<(), EngineError> {
    if msgs.is_empty() {
        return Ok(());
    }

    let new_messages = msgs
        .iter()
        .enumerate()
        .map(|(message_order, message)| {
            let conversation_id = data.conversation_id;

            let msg = messages::ActiveModel {
                id: ActiveValue::Set(Uuid::new_v4()),
                conversation_id: conversation_id.into_active_value(),

                flow_id: data.context.flow.clone().into_active_value(),
                step_id: data.context.step.get_step().to_owned().into_active_value(),
                direction: ActiveValue::Set(direction),
                payload: encrypt_data(message)?.into_active_value(),
                content_type: message["content_type"]
                    .as_str()
                    .unwrap_or("text")
                    .to_owned()
                    .into_active_value(),

                message_order: message_order
                    .to_i32()
                    .ok_or_else(|| {
                        EngineError::Internal(format!(
                            "can't convert message_order value ({message_order}) to i32"
                        ))
                    })?
                    .into_active_value(),
                interaction_order: interaction_order
                    .to_i32()
                    .ok_or_else(|| {
                        EngineError::Internal(format!(
                            "can't convert interaction_order value ({interaction_order}) to i32"
                        ))
                    })?
                    .into_active_value(),
                expires_at: expires_at.into_active_value(),

                ..Default::default()
            };
            Ok(msg)
        })
        .collect::<Result<Vec<_>, EngineError>>()?;

    messages::Entity::insert_many(new_messages).exec(db).await?;

    Ok(())
}

pub async fn get_client_messages<C: ConnectionTrait>(
    db: &C,
    filter: ClientMessageFilter<'_>,
) -> Result<models::Paginated<Message>, EngineError> {
    let ClientMessageFilter {
        client,
        limit,
        pagination_key,
        from_date,
        to_date,
        conversation_id,
    } = filter;

    let pagination_key = pagination_key.unwrap_or(0);

    let (messages, total_pages) = get_messages_with_filter(
        client,
        db,
        limit,
        from_date,
        to_date,
        conversation_id,
        pagination_key,
    )
    .await?;

    let pagination = (pagination_key < total_pages).then_some(PaginationData {
        page: pagination_key,
        total_pages,
        per_page: limit,
    });
    Ok(models::Paginated {
        data: messages,
        pagination,
    })
}

pub(crate) async fn get_messages_with_filter<C: ConnectionTrait>(
    client: &Client,
    conn: &C,
    limit_per_page: u32,
    from_date: Option<i64>,
    to_date: Option<i64>,
    conversation_id: Option<Uuid>,
    pagination_key: u32,
) -> Result<(Vec<Message>, u32), EngineError> {
    let client = client.clone();

    let mut statement = messages::Entity::find()
        .inner_join(conversation::Entity)
        .filter(conversation::Column::BotId.eq(client.bot_id))
        .filter(conversation::Column::ChannelId.eq(client.channel_id))
        .filter(conversation::Column::UserId.eq(client.user_id));

    if let Some(from_date) = from_date {
        let from_date = DateTime::from_timestamp(from_date, 0)
            .ok_or(EngineError::DateTimeError(
                "Date time is out of range".to_owned(),
            ))?
            .naive_utc();
        let to_date = match to_date {
            Some(to_date) => DateTime::from_timestamp(to_date, 0)
                .ok_or(EngineError::DateTimeError(
                    "Date time is out of range".to_owned(),
                ))?
                .naive_utc(),
            None => chrono::Utc::now().naive_utc(),
        };

        statement = statement
            .filter(messages::Column::CreatedAt.gte(from_date))
            .filter(messages::Column::CreatedAt.lte(to_date));
    }

    if let Some(conversation_id) = conversation_id {
        statement = statement.filter(conversation::Column::Id.eq(conversation_id));
    }
    statement = statement
        .order_by_desc(messages::Column::CreatedAt)
        .order_by_desc(messages::Column::MessageOrder);

    let paginated = PaginatorTrait::paginate(statement, conn, u64::from(limit_per_page));

    let res = paginated.fetch_page(u64::from(pagination_key)).await?;
    let page_count = paginated.num_pages().await?;
    let res = res
        .into_iter()
        .map(Message::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((
        res,
        page_count.to_u32().ok_or_else(|| {
            EngineError::Internal(format!(
                "can't convert page_count value ({page_count}) to u32"
            ))
        })?,
    ))
}

pub async fn get_conversation_messages<C: ConnectionTrait>(
    conn: &C,
    conversation_id: Uuid,
) -> Result<Vec<Message>, EngineError> {
    let messages = Query::get_conversation_messages(conn, conversation_id).await?;

    let messages = messages
        .into_iter()
        .map(Message::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

pub async fn delete_user_messages<C: ConnectionTrait>(
    conn: &C,
    client: &Client,
) -> Result<(), EngineError> {
    let conversations = filter_by_client(conversation::Entity::find(), client)
        .all(conn)
        .await?;
    for conversation in conversations {
        messages::Entity::delete_many()
            .filter(messages::Column::ConversationId.eq(conversation.id))
            .exec(conn)
            .await?;
    }

    Ok(())
}
