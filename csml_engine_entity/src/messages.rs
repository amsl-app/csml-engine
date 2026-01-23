use chrono::NaiveDateTime;
use sea_orm::entity::prelude::*;
use strum::AsRefStr;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum, Clone, Copy, AsRefStr)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Direction {
    #[sea_orm(string_value = "SEND")]
    #[strum(serialize = "SEND")]
    Send,
    #[sea_orm(string_value = "RECEIVE")]
    #[strum(serialize = "RECEIVE")]
    Receive,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "csml_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub conversation_id: Uuid,

    pub flow_id: String,
    pub step_id: String,
    pub direction: Direction,
    pub payload: String,
    pub content_type: String,

    pub message_order: i32,
    pub interaction_order: i32,

    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,

    pub expires_at: Option<NaiveDateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::conversation::Entity",
        from = "Column::ConversationId",
        to = "super::conversation::Column::Id"
    )]
    Conversation,
}

impl Related<super::conversation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Conversation.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
