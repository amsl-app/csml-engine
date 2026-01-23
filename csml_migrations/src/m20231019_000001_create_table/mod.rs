mod csml_bot_versions;
mod csml_conversations;
mod csml_memories;
mod csml_messages;
mod csml_states;

use crate::MigratableTable;
pub(crate) use csml_bot_versions::CsmlBotVersions;
pub(crate) use csml_conversations::CsmlConversations;
pub(crate) use csml_memories::CsmlMemories;
pub(crate) use csml_messages::CsmlMessages;
pub(crate) use csml_states::CsmlStates;
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &'static str {
        "m20231019_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let create_statements = [
            CsmlBotVersions::create(),
            CsmlConversations::create(),
            CsmlMessages::create(),
            CsmlMemories::create(),
            CsmlStates::create(),
        ];
        for create_statement in create_statements {
            manager.create_table(create_statement).await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let drop_statements = [
            CsmlBotVersions::drop(),
            CsmlConversations::drop(),
            CsmlMessages::drop(),
            CsmlMemories::drop(),
            CsmlStates::drop(),
        ];
        for drop_statement in drop_statements.into_iter().rev() {
            manager.drop_table(drop_statement).await?;
        }
        Ok(())
    }
}
