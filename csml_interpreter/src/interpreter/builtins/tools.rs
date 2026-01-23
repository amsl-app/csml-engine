use crate::data::primitive::PrimitiveString;
use crate::data::{Client, Interval, Literal};
use std::collections::HashMap;

#[must_use]
pub fn client_to_json(client: &Client, interval: Interval) -> HashMap<String, Literal> {
    HashMap::from([
        (
            "bot_id".to_owned(),
            PrimitiveString::get_literal(&client.bot_id, interval),
        ),
        (
            "channel_id".to_owned(),
            PrimitiveString::get_literal(&client.channel_id, interval),
        ),
        (
            "user_id".to_owned(),
            PrimitiveString::get_literal(&client.user_id, interval),
        ),
    ])
}
