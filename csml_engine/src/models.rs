use csml_interpreter::data::CsmlBot;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct BotVersion {
    pub bot: CsmlBot,
    pub version_id: String,
    pub engine_version: String,
}

impl BotVersion {
    pub fn flatten(&self) -> serde_json::Value {
        let mut value = self.bot.to_json();

        value["version_id"] = serde_json::json!(self.version_id);
        value["engine_version"] = serde_json::json!(self.engine_version);

        value
    }
}
