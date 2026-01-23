use crate::data::EngineError;
use base64::Engine;
use csml_interpreter::data::{CsmlBot, CsmlResult};
use csml_interpreter::{BINCODE_CONFIG, load_components, search_for_modules, validate_bot};

/**
 * Initialize the bot
 */
pub fn init_bot(bot: &mut CsmlBot) -> Result<(), EngineError> {
    // load native components into the bot
    bot.native_components = match load_components() {
        Ok(components) => Some(components),
        Err(err) => return Err(EngineError::Interpreter(err.format_error())),
    };

    if let Err(err) = search_for_modules(bot) {
        return Err(EngineError::Interpreter(format!("{err:?}")));
    }

    set_bot_ast(bot)
}

/**
 * Initialize bot ast
 */
pub fn set_bot_ast(bot: &mut CsmlBot) -> Result<(), EngineError> {
    let CsmlResult {
        flows,
        extern_flows,
        errors,
        ..
    } = validate_bot(bot);
    if !errors.is_empty() {
        return Err(EngineError::Interpreter(format!("{errors:?}")));
    }
    if flows.is_empty() {
        return Err(EngineError::Interpreter("empty bot".to_string()));
    }
    bot.bot_ast = Some(
        base64::engine::general_purpose::STANDARD
            .encode(bincode::encode_to_vec((&flows, &extern_flows), BINCODE_CONFIG).unwrap()),
    );

    Ok(())
}
