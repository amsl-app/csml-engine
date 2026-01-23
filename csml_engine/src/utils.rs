use crate::data::{ConversationInfo, EngineError};

use base64::Engine;

use csml_interpreter::{
    BINCODE_CONFIG,
    data::{
        Context, Event, Interval, Memory, Message,
        ast::{Flow, InsertStep, InstructionScope},
        context::ContextStepInfo,
    },
    get_step,
    interpreter::json_to_literal,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;

use crate::data::models::{CsmlRequest, Direction, MessageData};
use csml_interpreter::data::{CsmlBot, CsmlFlow};
use csml_interpreter::error_format::ErrorInfo;
use csml_model::FlowTrigger;
use md5::{Digest, Md5};

/**
 * Update current context memories in place.
 * This method is used to avoid saving memories in DB every time a `remember` is used
 * Instead, the memory is saved in bulk at the end of each step or interaction, but we still
 * must allow the user to use the `remembered` data immediately.
 */
pub fn update_current_context(
    data: &mut ConversationInfo,
    memories: &HashMap<String, Memory>,
) -> Result<(), ErrorInfo> {
    for mem in memories.values() {
        let lit = json_to_literal(&mem.value, Interval::default(), &data.context.flow)?;

        data.context.current.insert(mem.key.clone(), lit);
    }
    Ok(())
}

/**
 * Prepare a formatted "content" for the event object, based on the user's input.
 * This will trim extra data and only keep the main value.
 */
pub fn get_event_content(content_type: &str, metadata: &Value) -> Result<String, EngineError> {
    let res = match content_type {
        file if ["file", "audio", "video", "image", "url"].contains(&file) => {
            if let Some(val) = metadata["url"].as_str() {
                Ok(val.to_string())
            } else {
                Err(EngineError::Interpreter(
                    "no url content in event".to_owned(),
                ))
            }
        }
        "payload" => {
            if let Some(val) = metadata.get("payload") {
                Ok(val.to_string())
            } else {
                Err(EngineError::Interpreter(
                    "no payload content in event".to_owned(),
                ))
            }
        }
        "text" => {
            if let Some(val) = metadata["text"].as_str() {
                Ok(val.to_string())
            } else {
                Err(EngineError::Interpreter(
                    "no text content in event".to_owned(),
                ))
            }
        }
        "regex" => {
            if let Some(val) = metadata["payload"].as_str() {
                Ok(val.to_string())
            } else {
                Err(EngineError::Interpreter(
                    "invalid payload for event type regex".to_owned(),
                ))
            }
        }
        "flow_trigger" => match serde_json::from_value::<FlowTrigger>(metadata.clone()) {
            Ok(_) => Ok(metadata.to_string()),
            Err(_) => Err(EngineError::Interpreter(
                "invalid content for event type flow_trigger: expect flow_id and optional step_id"
                    .to_owned(),
            )),
        },
        content_type => {
            tracing::error!(%content_type, "invalid content_type");
            return Err(EngineError::Interpreter(format!(
                "{content_type} is not a valid content_type"
            )));
        }
    };

    if let Err(EngineError::Interpreter(ref err)) = res {
        tracing::error!(err);
    }

    res
}

/**
 * Format the incoming (JSON-formatted) event into an Event struct.
 */
pub fn format_event(request: &CsmlRequest) -> Result<Event, EngineError> {
    let step_limit = request.step_limit;
    let json_event = json!(request);

    let Some(content_type) = json_event["payload"]["content_type"].as_str() else {
        tracing::error!("no content_type in event payload");
        return Err(EngineError::Interpreter(
            "no content_type in event payload".to_owned(),
        ));
    };
    let content = json_event["payload"]["content"].clone();

    let content_value = get_event_content(content_type, &content)?;

    Ok(Event {
        content_type: content_type.to_string(),
        content_value,
        content,
        ttl_duration: json_event["ttl_duration"].as_i64(),
        low_data_mode: json_event["low_data_mode"].as_bool(),
        step_limit,
        secure: json_event["payload"]["secure"].as_bool().unwrap_or(false),
    })
}

/**
 * Update `ConversationInfo` data with current information about the request.
 */
pub(crate) fn add_info_to_message(
    msg: Message,
    message_order: u32,
    interaction_order: u32,
) -> MessageData {
    MessageData {
        message_order,
        interaction_order,
        direction: Direction::Send,
        payload: msg.into(),
    }
}

/**
 * Prepare correctly formatted messages as requested in both:
 * - send action: when `callback_url` is set, messages are sent as they come to a defined endpoint
 * - return action: at the end of the interaction, all messages are returned as they were processed
 */
pub fn messages_formatter(
    vec_msg: Vec<Message>,
    interaction_order: u32,
) -> Result<Vec<MessageData>, EngineError> {
    vec_msg
        .into_iter()
        .enumerate()
        .map(|(message_order, msg)| {
            let message_order = u32::try_from(message_order).map_err(|_| {
                EngineError::Internal(format!("message_order is too large ({message_order})"))
            });
            message_order
                .map(|message_order| add_info_to_message(msg, message_order, interaction_order))
        })
        .collect()
}

/// Retrieve a flow in a given bot by an identifier:
///
/// - Matching method is case-insensitive
///
/// - As name is similar to a flow's alias, both flow.name and flow.id can be matched.
pub fn get_flow_by_id<'a>(
    flow_id: &str,
    flows: &'a [CsmlFlow],
) -> Result<&'a CsmlFlow, EngineError> {
    let id = flow_id.to_ascii_lowercase();
    // TODO: move to_lowercase at creation of vars
    tracing::trace!(flow = id, "searching for flow");
    flows
        .iter()
        .find(|&val| val.id.to_ascii_lowercase() == id || val.name.to_ascii_lowercase() == id)
        .ok_or_else(|| {
            tracing::trace!(flow = id, "flow not found");
            EngineError::Interpreter(format!("Flow '{flow_id}' does not exist"))
        })
}

/**
 * Retrieve a bot's default flow.
 * The default flow must exist!
 */
pub fn get_default_flow(bot: &CsmlBot) -> Result<&CsmlFlow, EngineError> {
    match bot
        .flows
        .iter()
        .find(|&flow| flow.id == bot.default_flow || flow.name == bot.default_flow)
    {
        Some(flow) => Ok(flow),
        None => Err(EngineError::Interpreter(
            "The bot's default_flow does not exist".to_owned(),
        )),
    }
}

pub fn get_current_step_hash(context: &Context, bot: &CsmlBot) -> Result<String, EngineError> {
    let mut hash = Md5::new();

    let step = match &context.step {
        ContextStepInfo::Normal(step) => {
            let flow = &get_flow_by_id(&context.flow, &bot.flows)?.content;

            let ast = match &bot.bot_ast {
                Some(ast) => {
                    let base64decoded = base64::engine::general_purpose::STANDARD.decode(ast)?;
                    let csml_bot: HashMap<String, Flow> =
                        bincode::decode_from_slice(&base64decoded[..], BINCODE_CONFIG)
                            .unwrap()
                            .0;
                    match csml_bot.get(&context.flow) {
                        Some(flow) => flow.clone(),
                        None => csml_bot.get(&get_default_flow(bot)?.name).unwrap().clone(),
                    }
                }
                None => return Err(EngineError::Manager("not valid ast".to_string())),
            };

            get_step(step, flow, &ast)
        }
        ContextStepInfo::UnknownFlow(step) => {
            let flow = &get_flow_by_id(&context.flow, &bot.flows)?.content;

            match &bot.bot_ast {
                Some(ast) => {
                    let base64decoded = base64::engine::general_purpose::STANDARD.decode(ast)?;
                    let csml_bot: HashMap<String, Flow> =
                        bincode::decode_from_slice(&base64decoded[..], BINCODE_CONFIG)
                            .unwrap()
                            .0;

                    let default_flow = csml_bot.get(&get_default_flow(bot)?.name).unwrap();

                    match csml_bot.get(&context.flow) {
                        Some(target_flow) => {
                            // check if there is a inserted step with the same name as the target step
                            let insertion_expr = target_flow.flow_instructions.get_key_value(
                                &InstructionScope::InsertStep(InsertStep {
                                    name: step.clone(),
                                    original_name: None,
                                    from_flow: String::new(),
                                    interval: Interval::default(),
                                }),
                            );

                            // if there is a inserted step get the flow of the target step and
                            if let Some((InstructionScope::InsertStep(insert), _)) = insertion_expr
                            {
                                match csml_bot.get(&insert.from_flow) {
                                    Some(inserted_step_flow) => {
                                        let inserted_raw_flow =
                                            &get_flow_by_id(&insert.from_flow, &bot.flows)?.content;

                                        get_step(step, inserted_raw_flow, inserted_step_flow)
                                    }
                                    None => get_step(step, flow, default_flow),
                                }
                            } else {
                                get_step(step, flow, target_flow)
                            }
                        }
                        None => get_step(step, flow, default_flow),
                    }
                }
                None => return Err(EngineError::Manager("not valid ast".to_string())),
            }
        }
        ContextStepInfo::InsertedStep {
            step,
            flow: inserted_flow,
        } => {
            let flow = &get_flow_by_id(inserted_flow, &bot.flows)?.content;

            let ast = match &bot.bot_ast {
                Some(ast) => {
                    let base64decoded = base64::engine::general_purpose::STANDARD.decode(ast)?;
                    let csml_bot: HashMap<String, Flow> =
                        bincode::decode_from_slice(&base64decoded[..], BINCODE_CONFIG)
                            .unwrap()
                            .0;

                    match csml_bot.get(inserted_flow) {
                        Some(flow) => flow.clone(),
                        None => csml_bot.get(&get_default_flow(bot)?.name).unwrap().clone(),
                    }
                }
                None => return Err(EngineError::Manager("not valid ast".to_string())),
            };

            get_step(step, flow, &ast)
        }
    };

    hash.update(step.as_bytes());

    Ok(format!("{:x}", hash.finalize()))
}

pub fn get_ttl_duration_value(event: Option<&Event>) -> Option<chrono::Duration> {
    if let Some(event) = event
        && let Some(ttl) = event.ttl_duration
    {
        return chrono::Duration::try_days(ttl);
    }

    if let Ok(ttl) = env::var("TTL_DURATION")
        && let Ok(ttl) = ttl.parse::<i64>()
    {
        return chrono::Duration::try_days(ttl);
    }

    None
}

pub fn get_low_data_mode_value(event: &Event) -> bool {
    if let Some(low_data) = event.low_data_mode {
        return low_data;
    }

    if let Ok(low_data) = env::var("LOW_DATA_MODE")
        && let Ok(low_data) = low_data.parse::<bool>()
    {
        return low_data;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init::init_bot;

    #[test]
    fn test_get_step_hash() {
        let context = Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "start",
            "flow",
            None,
        );
        let flows = vec![CsmlFlow::new(
            "id",
            "flow",
            "start:\n  hold\n  remember a = event\n  goto end\n",
            vec![],
        )];
        let mut bot = CsmlBot {
            id: "id".to_owned(),
            name: "name".to_owned(),
            default_flow: "flow".to_owned(),
            flows,
            modules: None,
            multibot: None,
            native_components: None,
            bot_ast: None,
            no_interruption_delay: None,
            apps_endpoint: None,
            custom_components: None,
            env: None,
        };
        init_bot(&mut bot).unwrap();
        let hash = get_current_step_hash(&context, &bot).unwrap();
        assert_eq!(hash, "a60ae526e3d4dd07bfb29fbb93e76cb9");
    }
}
