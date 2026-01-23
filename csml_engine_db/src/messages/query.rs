use csml_engine_entity::messages;
use csml_engine_entity::messages::{Direction, Entity, Model};
use sea_orm::prelude::*;
use sea_orm::sea_query::Alias;
use sea_orm::{QueryOrder, QuerySelect};

#[derive(Debug, Clone, Copy, DeriveColumn)]
enum Column {
    DirectionOrder,
}

pub struct Query {}

impl Query {
    pub async fn get_conversation_messages<C: ConnectionTrait>(
        conn: &C,
        conversation_id: Uuid,
    ) -> Result<Vec<Model>, DbErr> {
        let messages = Entity::find()
            .filter(messages::Column::ConversationId.eq(conversation_id))
            .column_as(
                messages::Column::Direction
                    .eq(Direction::Send.as_ref())
                    .cast_as(Alias::new("integer")),
                Column::DirectionOrder,
            )
            .order_by_desc(messages::Column::CreatedAt)
            .order_by_desc(Expr::col(Column::DirectionOrder))
            .order_by_desc(messages::Column::MessageOrder)
            .all(conn)
            .await?;

        Ok(messages)
    }
}
