use crate::MigratableTable;
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(crate) enum CsmlConversations {
    Table,
    Id,
    BotId,
    ChannelId,
    UserId,
    FlowId,
    StepId,
    Status,
    LastInteractionAt,
    UpdatedAt,
    CreatedAt,
    ExpiresAt,
}

impl MigratableTable for CsmlConversations {
    fn create() -> TableCreateStatement {
        Table::create()
            .table(Self::Table)
            .if_not_exists()
            .col(ColumnDef::new(Self::Id).uuid().not_null().primary_key())
            .col(ColumnDef::new(Self::BotId).string().not_null())
            .col(ColumnDef::new(Self::ChannelId).string().not_null())
            .col(ColumnDef::new(Self::UserId).string().not_null())
            .col(ColumnDef::new(Self::FlowId).string().not_null())
            .col(ColumnDef::new(Self::StepId).string().not_null())
            .col(ColumnDef::new(Self::Status).string().not_null())
            .col(
                ColumnDef::new(Self::LastInteractionAt)
                    .timestamp()
                    .not_null()
                    .default(Expr::current_timestamp()),
            )
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
