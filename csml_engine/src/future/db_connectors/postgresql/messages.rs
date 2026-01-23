use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use std::convert::{TryFrom, TryInto};

use crate::{Client, data, encrypt::encrypt_data};

use super::pagination::Paginate;
use crate::data::filter::ClientMessageFilter;
use crate::data::models::PaginationData;
use crate::data::{AsyncPostgresqlClient, ConversationInfo, EngineError};
use crate::db_connectors::diesel::Direction;
use crate::db_connectors::postgresql::{
    models,
    schema::{csml_conversations, csml_messages},
};
use chrono::{DateTime, NaiveDateTime};
use num_traits::ToPrimitive;
use uuid::Uuid;

pub async fn add_messages_bulk(
    data: &mut ConversationInfo,
    messages: &[serde_json::Value],
    interaction_order: u32,
    direction: Direction,
    expires_at: Option<NaiveDateTime>,
    db: &mut AsyncPostgresqlClient<'_>,
) -> Result<(), EngineError> {
    if messages.is_empty() {
        return Ok(());
    }

    let new_messages = messages
        .iter()
        .enumerate()
        .map(|(message_order, message)| {
            let conversation_id = data.conversation_id;
            let msg = models::NewMessages {
                id: Uuid::new_v4(),
                conversation_id,

                flow_id: &data.context.flow,
                step_id: data.context.step.get_step(),
                direction,
                payload: encrypt_data(message)?,
                content_type: message["content_type"].as_str().unwrap_or("text"),

                message_order: message_order.to_i32().ok_or_else(|| {
                    EngineError::Internal(format!(
                        "can't convert message_order value ({message_order}) to i32",
                    ))
                })?,
                interaction_order: interaction_order.to_i32().ok_or_else(|| {
                    EngineError::Internal(format!(
                        "can't convert interaction_order value ({interaction_order}) to i32",
                    ))
                })?,
                expires_at,
            };
            Ok(msg)
        })
        .collect::<Result<Vec<_>, EngineError>>()?;

    diesel::insert_into(csml_messages::table)
        .values(&new_messages)
        .get_result::<models::Message>(db.client.as_mut())
        .await?;

    Ok(())
}

pub async fn delete_user_messages(
    client: &Client,
    db: &mut AsyncPostgresqlClient<'_>,
) -> Result<(), EngineError> {
    let conversations: Vec<models::Conversation> = csml_conversations::table
        .filter(csml_conversations::bot_id.eq(&client.bot_id))
        .filter(csml_conversations::channel_id.eq(&client.channel_id))
        .filter(csml_conversations::user_id.eq(&client.user_id))
        .load(db.client.as_mut())
        .await?;

    for conversation in conversations {
        diesel::delete(
            csml_messages::table.filter(csml_messages::conversation_id.eq(&conversation.id)),
        )
        .execute(db.client.as_mut())
        .await
        .ok();
    }

    Ok(())
}

pub async fn get_client_messages<'a, 'conn: 'a, 'b>(
    db: &'a mut AsyncPostgresqlClient<'conn>,
    filter: ClientMessageFilter<'b>,
) -> Result<data::models::Paginated<data::models::Message>, EngineError> {
    let ClientMessageFilter {
        client,
        limit,
        pagination_key,
        from_date,
        to_date,
        conversation_id,
    } = filter;

    let pagination_key = pagination_key.unwrap_or(0);

    let (conversation_with_messages, total_pages) = match conversation_id {
        None => {
            get_messages_without_conversation_filter(
                client,
                db,
                limit,
                from_date,
                to_date,
                pagination_key,
            )
            .await?
        }
        Some(conv_id) => {
            get_messages_with_conversation_filter(
                client,
                db,
                limit,
                from_date,
                to_date,
                pagination_key,
                conv_id,
            )
            .await?
        }
    };

    let (_, messages): (Vec<_>, Vec<_>) = conversation_with_messages.into_iter().unzip();

    let messages = messages
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<Vec<_>, _>>()?;

    let pagination = (pagination_key < total_pages).then_some(PaginationData {
        page: pagination_key,
        total_pages,
        per_page: limit,
    });
    Ok(data::models::Paginated {
        data: messages,
        pagination,
    })
}

pub(crate) async fn get_messages_without_conversation_filter(
    client: &Client,
    db: &mut AsyncPostgresqlClient<'_>,
    limit_per_page: u32,
    from_date: Option<i64>,
    to_date: Option<i64>,
    pagination_key: u32,
) -> Result<(Vec<(models::Conversation, models::Message)>, u32), EngineError> {
    let client = client.clone();
    let res = if let Some(from_date) = from_date {
        let from_date = DateTime::from_timestamp(from_date, 0)
            .ok_or(EngineError::DateTimeError(
                "Date time is out of range".to_owned(),
            ))?
            .naive_utc();
        let to_date = match to_date {
            Some(to_date) => DateTime::from_timestamp(to_date, 0).ok_or(
                EngineError::DateTimeError("Date time is out of range".to_owned()),
            )?,
            None => chrono::Utc::now(),
        }
        .naive_utc();

        let mut query = csml_conversations::table
            .filter(csml_conversations::bot_id.eq(client.bot_id))
            .filter(csml_conversations::channel_id.eq(client.channel_id))
            .filter(csml_conversations::user_id.eq(client.user_id))
            .inner_join(csml_messages::table)
            .filter(csml_messages::created_at.ge(from_date))
            .filter(csml_messages::created_at.le(to_date))
            .select((csml_conversations::all_columns, csml_messages::all_columns))
            .order_by(csml_messages::created_at.desc())
            .then_order_by(csml_messages::message_order.desc())
            .paginate(pagination_key);

        query = query.per_page(limit_per_page);

        query.load_and_count_pages(db.client.as_mut()).await?
    } else {
        let mut query = csml_conversations::table
            .filter(csml_conversations::bot_id.eq(client.bot_id))
            .filter(csml_conversations::channel_id.eq(client.channel_id))
            .filter(csml_conversations::user_id.eq(client.user_id))
            .inner_join(csml_messages::table)
            .select((csml_conversations::all_columns, csml_messages::all_columns))
            .order_by(csml_messages::created_at.desc())
            .then_order_by(csml_messages::message_order.desc())
            .paginate(pagination_key);

        query = query.per_page(limit_per_page);

        query.load_and_count_pages(db.client.as_mut()).await?
    };
    Ok(res)
}

async fn get_messages_with_conversation_filter(
    client: &Client,
    db: &mut AsyncPostgresqlClient<'_>,
    limit_per_page: u32,
    from_date: Option<i64>,
    to_date: Option<i64>,
    pagination_key: u32,
    conversation_id: Uuid,
) -> Result<(Vec<(models::Conversation, models::Message)>, u32), EngineError> {
    let client = client.clone();
    let res = if let Some(from_date) = from_date {
        let from_date = DateTime::from_timestamp(from_date, 0)
            .ok_or(EngineError::DateTimeError(
                "Date time is out of range".to_owned(),
            ))?
            .naive_utc();
        let to_date = match to_date {
            Some(to_date) => DateTime::from_timestamp(to_date, 0).ok_or(
                EngineError::DateTimeError("Date time is out of range".to_owned()),
            )?,
            None => chrono::Utc::now(),
        }
        .naive_utc();

        let mut query = csml_conversations::table
            .filter(csml_conversations::bot_id.eq(client.bot_id))
            .filter(csml_conversations::channel_id.eq(client.channel_id))
            .filter(csml_conversations::user_id.eq(client.user_id))
            .filter(csml_conversations::id.eq(conversation_id))
            .inner_join(csml_messages::table)
            .filter(csml_messages::created_at.ge(from_date))
            .filter(csml_messages::created_at.le(to_date))
            .select((csml_conversations::all_columns, csml_messages::all_columns))
            .order_by(csml_messages::created_at.desc())
            .then_order_by(csml_messages::message_order.desc())
            .paginate(pagination_key);

        query = query.per_page(limit_per_page);

        query.load_and_count_pages(db.client.as_mut()).await?
    } else {
        let mut query = csml_conversations::table
            .filter(csml_conversations::bot_id.eq(client.bot_id))
            .filter(csml_conversations::channel_id.eq(client.channel_id))
            .filter(csml_conversations::user_id.eq(client.user_id))
            .filter(csml_conversations::id.eq(conversation_id))
            .inner_join(csml_messages::table)
            .select((csml_conversations::all_columns, csml_messages::all_columns))
            .order_by(csml_messages::created_at.desc())
            .then_order_by(csml_messages::message_order.desc())
            .paginate(pagination_key);

        query = query.per_page(limit_per_page);

        query.load_and_count_pages(db.client.as_mut()).await?
    };
    Ok(res)
}

pub async fn get_conversation_messages(
    db: &mut AsyncPostgresqlClient<'_>,
    conversation_id: Uuid,
) -> Result<Vec<data::models::Message>, EngineError> {
    let messages: Vec<(models::Message, i32)> = csml_messages::table
        .select((
            models::Message::as_select(),
            diesel::dsl::sql::<diesel::sql_types::Integer>(
                "SUM(CAST(\"direction\" = 'SEND' AS Integer)) as direction_order",
            ),
        ))
        .filter(csml_messages::conversation_id.eq(conversation_id))
        .order_by(csml_messages::created_at.desc())
        .then_order_by(diesel::dsl::sql::<diesel::sql_types::Integer>(
            "direction_order DESC",
        ))
        .then_order_by(csml_messages::message_order.desc())
        .load(db.client.as_mut())
        .await?;

    let messages = messages
        .into_iter()
        .map(|(message, _)| data::models::Message::try_from(message))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}
