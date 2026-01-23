use crate::MigratableTable;
use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub(crate) enum CsmlBotVersions {
    Table,
    Id,
    BotId,
    Bot,
    EngineVersion,
    UpdatedAt,
    CreatedAt,
}

impl MigratableTable for CsmlBotVersions {
    fn create() -> TableCreateStatement {
        Table::create()
            .table(Self::Table)
            .if_not_exists()
            .col(ColumnDef::new(Self::Id).uuid().not_null().primary_key())
            .col(ColumnDef::new(Self::BotId).string().not_null())
            .col(ColumnDef::new(Self::Bot).string().not_null())
            .col(ColumnDef::new(Self::EngineVersion).string().not_null())
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
