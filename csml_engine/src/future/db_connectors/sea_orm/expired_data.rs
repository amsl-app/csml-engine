use crate::data::EngineError;
use sea_orm::prelude::*;

pub async fn delete_expired_data<C: ConnectionTrait>(conn: &C) -> Result<(), EngineError> {
    let date_now = chrono::Utc::now().naive_utc();

    csml_engine_entity::conversation::Entity::delete_many()
        .filter(csml_engine_entity::conversation::Column::ExpiresAt.lt(date_now))
        .exec(conn)
        .await?;

    csml_engine_entity::memory::Entity::delete_many()
        .filter(csml_engine_entity::memory::Column::ExpiresAt.lt(date_now))
        .exec(conn)
        .await?;

    csml_engine_entity::state::Entity::delete_many()
        .filter(csml_engine_entity::state::Column::ExpiresAt.lt(date_now))
        .exec(conn)
        .await?;

    Ok(())
}
