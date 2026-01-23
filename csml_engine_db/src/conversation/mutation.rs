use csml_engine_entity::conversation::{ActiveModel, Model, Status};
use sea_orm::ActiveValue;
use sea_orm::prelude::*;
pub struct Mutation {}

impl Mutation {
    pub async fn set_status<C: ConnectionTrait>(
        db: &C,
        id: Uuid,
        status: Status,
    ) -> Result<Model, DbErr> {
        let conversation = ActiveModel {
            id: ActiveValue::Set(id),
            status: ActiveValue::Set(status),
            ..Default::default()
        };

        conversation.update(db).await
    }
}
