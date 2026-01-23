use crate::data::Client;
use crate::data::Literal;
use crate::data::message_data::MessageData;
use crate::data::position::Position;
use crate::error_format::{ERROR_PAYLOAD_EXCEED_MAX_SIZE, ErrorInfo, gen_error_info};

use serde_json::{Value, json};

////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub enum MessageType {
    Msg(Message),
    Empty,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub content_type: String,
    pub content: Value,
}
const MAX_PAYLOAD_SIZE: usize = 16000;

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl Message {
    pub fn new(literal: Literal, flow_name: &str) -> Result<Self, ErrorInfo> {
        if literal.primitive.to_string().len() >= MAX_PAYLOAD_SIZE {
            return Err(gen_error_info(
                Position::new(literal.interval, flow_name),
                ERROR_PAYLOAD_EXCEED_MAX_SIZE.to_owned(),
            ));
        }

        Ok(literal.primitive.to_msg(literal.content_type))
    }

    #[must_use]
    pub fn add_to_message(msg_data: MessageData, action: MessageType) -> MessageData {
        match action {
            MessageType::Msg(msg) => msg_data.add_message(msg),
            MessageType::Empty => msg_data,
        }
    }

    #[must_use]
    pub fn switch_bot_message(bot_id: &str, client: &Client) -> Self {
        Self {
            content_type: "switch_bot".to_owned(),
            content: json!({ "bot_id": bot_id, "client": client }),
        }
    }

    #[must_use]
    pub fn message_to_json(&self) -> Value {
        json! ({
            "content_type": self.content_type,
            "content": self.content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_to_json() {
        let message = Message {
            content_type: "testing".to_owned(),
            content: json!({ "key": "value" }),
        };
        let expected = json!({
            "content_type": "testing",
            "content": { "key": "value" },
        });
        let res = message.message_to_json();
        assert_eq!(res, expected);
    }
}
