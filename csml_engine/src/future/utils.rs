use crate::{
    data::{AsyncDatabase, ConversationInfo, EngineError},
    future::db_connectors::state::delete_state_key,
    future::send::send_to_callback_url,
};

use csml_interpreter::data::{Client, CsmlBot, CsmlFlow, Event, Message};
use rand::seq::IndexedRandom;
use serde_json::json;

use crate::data::SeaOrmDbTraits;
use crate::utils::{get_flow_by_id, messages_formatter};
use csml_model::FlowTrigger;
use regex::Regex;

/**
 * Send a message to the configured `callback_url`.
 * If not `callback_url` is configured, skip this action.
 */
pub async fn send_msg_to_callback_url(
    data: &mut ConversationInfo,
    msg: Vec<Message>,
    interaction_order: u32,
) -> Result<(), EngineError> {
    let messages = messages_formatter(msg, interaction_order)?;

    send_to_callback_url(data, json!(messages)).await;
    Ok(())
}

/**
 * Find a flow in a bot based on the user's input.
 * - `flow_trigger` events will match a flow's id or name and reset the hold position
 * - other events will try to match a flow trigger
 */
pub async fn search_flow<'a, T: SeaOrmDbTraits>(
    event: &Event,
    bot: &'a CsmlBot,
    client: &Client,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(&'a CsmlFlow, String), EngineError> {
    match event {
        event if event.content_type == "flow_trigger" => {
            delete_state_key(client, "hold", "position", db).await?;

            let flow_trigger: FlowTrigger = serde_json::from_str(&event.content_value)?;

            match get_flow_by_id(&flow_trigger.flow_id, &bot.flows) {
                Ok(flow) => match flow_trigger.step_id {
                    Some(step_id) => Ok((flow, step_id)),
                    None => Ok((flow, "start".to_owned())),
                },
                Err(_) => Ok((
                    get_flow_by_id(&bot.default_flow, &bot.flows)?,
                    "start".to_owned(),
                )),
            }
        }
        event if event.content_type == "regex" => {
            let mut random_flows = vec![];

            for flow in &bot.flows {
                let contains_command = flow.commands.iter().any(|cmd| {
                    Regex::new(&event.content_value).is_ok_and(|action| action.is_match(cmd))
                });

                if contains_command {
                    random_flows.push(flow);
                }
            }

            // Thread rng is not send, so we have to drop it right away
            let random_flow = {
                let rng = &mut rand::rng();
                random_flows.choose(rng)
            };

            match random_flow {
                Some(flow) => {
                    delete_state_key(client, "hold", "position", db).await?;
                    Ok((flow, "start".to_owned()))
                }
                None => Err(EngineError::Interpreter(format!(
                    "no match found for regex: {}",
                    event.content_value
                ))),
            }
        }
        event => {
            let mut random_flows = vec![];

            for flow in &bot.flows {
                let contains_command = flow
                    .commands
                    .iter()
                    .any(|cmd| cmd.as_str().to_lowercase() == event.content_value.to_lowercase());

                if contains_command {
                    random_flows.push(flow);
                }
            }

            // Thread rng is not send, so we have to drop it right away
            let random_flow = {
                let rng = &mut rand::rng();
                random_flows.choose(rng)
            };

            match random_flow {
                Some(flow) => {
                    delete_state_key(client, "hold", "position", db).await?;
                    Ok((flow, "start".to_owned()))
                }
                None => Err(EngineError::Interpreter(format!(
                    "Flow '{}' does not exist",
                    event.content_value
                ))),
            }
        }
    }
}

pub async fn clean_hold_and_restart<T: SeaOrmDbTraits>(
    data: &mut ConversationInfo,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    delete_state_key(&data.client, "hold", "position", db).await?;
    data.context.hold = None;
    Ok(())
}
