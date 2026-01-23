use crate::data::models::BotOpt;
use crate::data::{AsyncDatabase, EngineError, SeaOrmDbTraits};
use crate::future::db_connectors;
use crate::models::BotVersion;
use csml_interpreter::data::{CsmlBot, MultiBot};

fn map_bot_version(
    apps_endpoint: Option<&str>,
    multi_bot: Option<&[MultiBot]>,
    bot_version: Option<BotVersion>,
) -> Option<CsmlBot> {
    bot_version.map(|mut bot_version| {
        bot_version.bot.apps_endpoint = apps_endpoint.map(ToOwned::to_owned);
        bot_version.bot.multibot = multi_bot.map(<[_]>::to_vec);
        bot_version.bot
    })
}

impl BotOpt {
    pub async fn search_bot_async<T: SeaOrmDbTraits>(
        &self,
        db: &mut AsyncDatabase<'_, T>,
    ) -> Result<CsmlBot, EngineError> {
        match self {
            Self::CsmlBot(csml_bot) => Ok(*csml_bot.clone()),
            Self::BotId {
                bot_id,
                apps_endpoint,
                multibot: multi_bot,
            } => {
                let bot_version = db_connectors::bot::get_last_bot_version(bot_id, db).await?;

                map_bot_version(apps_endpoint.as_deref(), multi_bot.as_deref(), bot_version)
                    .ok_or_else(|| EngineError::Manager(format!("bot ({bot_id}) not found in db")))
            }
            Self::Id {
                version_id,
                apps_endpoint,
                multibot: multi_bot,
                ..
            } => {
                let bot_version = db_connectors::bot::get_by_version_id(*version_id, db).await?;

                map_bot_version(apps_endpoint.as_deref(), multi_bot.as_deref(), bot_version)
                    .ok_or_else(|| {
                        EngineError::Manager(format!("bot version ({version_id}) not found in db"))
                    })
            }
        }
    }
}
