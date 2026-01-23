use csml_engine_entity::conversation::{Entity, Model};
use sea_orm::prelude::*;

pub struct Query {}

impl Query {
    pub async fn get<C: ConnectionTrait>(conn: &C, id: Uuid) -> Result<Model, DbErr> {
        Entity::find_by_id(id).one(conn).await.map(|model| {
            model.ok_or(DbErr::RecordNotFound(format!(
                "Conversation {id} not found"
            )))
        })?
    }
}
