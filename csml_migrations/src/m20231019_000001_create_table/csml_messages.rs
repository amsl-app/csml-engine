use super::CsmlConversations;
use crate::MigratableTable;
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(crate) enum CsmlMessages {
    Table,
    Id,
    ConversationId,
    FlowId,
    StepId,
    Direction,
    Payload,
    ContentType,
    MessageOrder,
    InteractionOrder,
    UpdatedAt,
    CreatedAt,
    ExpiresAt,
}

impl MigratableTable for CsmlMessages {
    fn create() -> TableCreateStatement {
        Table::create()
            .table(Self::Table)
            .if_not_exists()
            .col(ColumnDef::new(Self::Id).uuid().not_null().primary_key())
            .col(ColumnDef::new(Self::ConversationId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(Self::Table, Self::ConversationId)
                    .to(CsmlConversations::Table, CsmlConversations::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(Self::FlowId).string().not_null())
            .col(ColumnDef::new(Self::StepId).string().not_null())
            .col(ColumnDef::new(Self::Direction).string().not_null())
            .col(ColumnDef::new(Self::Payload).string().not_null())
            .col(ColumnDef::new(Self::ContentType).string().not_null())
            .col(ColumnDef::new(Self::MessageOrder).integer().not_null())
            .col(ColumnDef::new(Self::InteractionOrder).integer().not_null())
            .col(
                ColumnDef::new(Self::UpdatedAt)
                    .timestamp()
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .col(
                ColumnDef::new(Self::CreatedAt)
                    .timestamp()
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
            .col(ColumnDef::new(Self::ExpiresAt).timestamp())
            .clone()
    }
}
