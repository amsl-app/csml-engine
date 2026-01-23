use crate::MigratableTable;
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(crate) enum CsmlStates {
    Table,
    Id,
    BotId,
    ChannelId,
    UserId,
    Type,
    Key,
    Value,
    ExpiresAt,
    UpdatedAt,
    CreatedAt,
}

impl MigratableTable for CsmlStates {
    fn create() -> TableCreateStatement {
        Table::create()
            .table(Self::Table)
            .if_not_exists()
            .col(ColumnDef::new(Self::Id).uuid().not_null().primary_key())
            .col(ColumnDef::new(Self::BotId).string().not_null())
            .col(ColumnDef::new(Self::ChannelId).string().not_null())
            .col(ColumnDef::new(Self::UserId).string().not_null())
            .col(ColumnDef::new(Self::Type).string().not_null())
            .col(ColumnDef::new(Self::Key).string().not_null())
            .col(ColumnDef::new(Self::Value).string().not_null())
            .col(ColumnDef::new(Self::ExpiresAt).timestamp())
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
            .clone()
    }
}
