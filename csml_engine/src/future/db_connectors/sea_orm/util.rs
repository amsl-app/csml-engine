use csml_engine_entity::conversation;
use csml_model::Client;
use sea_orm::{ColumnTrait, QueryFilter};

pub fn filter_by_client<T>(query: T, client: &Client) -> T
where
    T: QueryFilter,
{
    query
        .filter(conversation::Column::BotId.eq(&client.bot_id))
        .filter(conversation::Column::ChannelId.eq(&client.channel_id))
        .filter(conversation::Column::UserId.eq(&client.user_id))
}
