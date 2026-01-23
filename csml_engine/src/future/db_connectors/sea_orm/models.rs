use crate::data::models::{Conversation, ConversationStatus, Message, Payload};
use crate::data::{EngineError, SerializeCsmlBot};
use crate::encrypt::decrypt_data;
use crate::models::BotVersion;
use chrono::NaiveDateTime;
use csml_engine_entity::messages::Direction;
use csml_engine_entity::{bot_version, conversation, messages};
use csml_interpreter::data::Client;
use num_traits::ToPrimitive;
use std::convert::TryFrom;

impl From<conversation::Status> for ConversationStatus {
    fn from(status: conversation::Status) -> Self {
        match status {
            conversation::Status::Open => Self::Open,
            conversation::Status::Closed => Self::Closed,
        }
    }
}

impl From<conversation::Model> for Conversation {
    fn from(conversation: conversation::Model) -> Self {
        Self {
            id: conversation.id,
            client: Client {
                bot_id: conversation.bot_id,
                channel_id: conversation.channel_id,
                user_id: conversation.user_id,
            },
            flow_id: conversation.flow_id,
            step_id: conversation.step_id,
            status: ConversationStatus::from(conversation.status),
            last_interaction_at: conversation.last_interaction_at.and_utc(),
            updated_at: conversation.updated_at.and_utc(),
            created_at: conversation.created_at.and_utc(),
            expires_at: conversation.expires_at.as_ref().map(NaiveDateTime::and_utc),
        }
    }
}

impl TryFrom<messages::Model> for Message {
    type Error = EngineError;

    fn try_from(message: messages::Model) -> Result<Self, Self::Error> {
        let payload: Payload = serde_json::from_value(decrypt_data(message.payload)?)?;
        if payload.content_type != message.content_type {
            return Err(EngineError::Internal(format!(
                "Message content_type {} does not match payload content_type {}",
                message.content_type, payload.content_type
            )));
        }

        Ok(Self {
            id: message.id,
            conversation_id: message.conversation_id,
            flow_id: message.flow_id,
            step_id: message.step_id,
            direction: crate::data::models::Direction::from(message.direction),
            payload,
            message_order: message.message_order.to_u32().ok_or_else(|| {
                EngineError::Internal(format!(
                    "can't convert message_order value ({}) to u32",
                    message.message_order
                ))
            })?,
            interaction_order: message.interaction_order.to_u32().ok_or_else(|| {
                EngineError::Internal(format!(
                    "can't convert interaction_order value ({}) to u32",
                    message.message_order
                ))
            })?,
            updated_at: message.updated_at.and_utc(),
            created_at: message.created_at.and_utc(),
            expires_at: message.expires_at.as_ref().map(NaiveDateTime::and_utc),
        })
    }
}

impl TryFrom<bot_version::Model> for BotVersion {
    type Error = EngineError;

    fn try_from(version: bot_version::Model) -> Result<Self, Self::Error> {
        let csml_bot: SerializeCsmlBot =
            serde_json::from_str(&version.bot).map_err(EngineError::Serde)?;
        Ok(Self {
            bot: csml_bot.to_bot(),
            version_id: version.id.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_owned(),
        })
    }
}

impl From<crate::data::models::Direction> for Direction {
    fn from(direction: crate::data::models::Direction) -> Self {
        match direction {
            crate::data::models::Direction::Send => Self::Send,
            crate::data::models::Direction::Receive => Self::Receive,
        }
    }
}

impl From<Direction> for crate::data::models::Direction {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Send => Self::Send,
            Direction::Receive => Self::Receive,
        }
    }
}
