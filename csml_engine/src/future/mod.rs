pub mod db_connectors;
pub mod init;
pub mod utils;
// mod encrypt;
// mod error_messages;
pub mod interpreter_actions;
pub mod send;
// mod models;

pub use csml_interpreter::{
    data::{
        Client, CsmlResult, Event,
        ast::{Expr, Flow, InstructionScope},
        csml_logs::*,
        error_info::ErrorInfo,
        position::Position,
        warnings::Warnings,
    },
    load_components, search_for_modules,
};
use std::sync::Arc;

use crate::data::{AsyncDatabase, ConversationInfo, EngineError, SeaOrmDbTraits};
use db_connectors::{
    conversations, messages, state,
    state::{delete_state_key, set_state_items},
};
use init::{init_conversation_info, switch_bot};
use interpreter_actions::interpret_step;
use utils::clean_hold_and_restart;

use crate::data::filter::ClientMessageFilter;
use crate::data::models::interpreter_actions::SwitchBot;
use crate::data::models::{
    BotOpt, Conversation, CsmlRequest, Direction, Message, MessageData, Paginated,
};
use crate::future::interpreter_actions::{InterpreterMessage, stream_step};
use crate::init::init_bot;
use crate::utils::{format_event, get_current_step_hash, get_default_flow};
use chrono::prelude::*;
use csml_interpreter::data::{Hold, IndexInfo, csml_bot::CsmlBot};
use futures::future::{BoxFuture, FutureExt};
use futures::{Stream, StreamExt};
use uuid::Uuid;

enum ConversationStatus {
    Delayed(Conversation),
    Started,
}

pub struct StreamData<'a, 'db: 'a, DB: SeaOrmDbTraits + Send> {
    conversation_info: ConversationInfo,
    db: &'a mut AsyncDatabase<'db, DB>,
    bot: Arc<CsmlBot>,
    event: Arc<Event>,
}

impl<'a, 'db: 'a, DB> StreamData<'a, 'db, DB>
where
    DB: SeaOrmDbTraits + Send,
{
    pub fn new(
        db: &'a mut AsyncDatabase<'db, DB>,
        bot: CsmlBot,
        event: Event,
        conversation_info: ConversationInfo,
    ) -> Self {
        Self {
            conversation_info,
            db,
            bot: Arc::new(bot),
            event: Arc::new(event),
        }
    }

    pub async fn stream(
        &mut self,
    ) -> Result<impl Stream<Item = Result<MessageData, EngineError>>, EngineError> {
        Ok(
            stream_step(&mut self.conversation_info, &self.event, &self.bot, self.db)
                .await?
                .map(|data| match data? {
                    InterpreterMessage::Message(message) => Ok(message),
                    InterpreterMessage::SwitchBot(_) => {
                        panic!("Switch bot not supported in stream");
                    }
                })
                .boxed(),
        )
    }

    pub async fn finalize(self) -> Result<Conversation, EngineError> {
        get_conversation(self.db, self.conversation_info.conversation_id).await
    }
}

pub async fn start_conversation_stream<'dbref, 'db: 'dbref, DB: SeaOrmDbTraits + Send>(
    request: CsmlRequest,
    mut bot_opt: BotOpt,
    db: &'dbref mut AsyncDatabase<'db, DB>,
) -> Result<(Conversation, Option<StreamData<'dbref, 'db, DB>>), EngineError> {
    let (conversation_info, conversation_status, bot, event) =
        setup_conversation(request, &mut bot_opt, db).await?;

    let conversation = match conversation_status {
        ConversationStatus::Started => {
            get_conversation(db, conversation_info.conversation_id).await?
        }
        ConversationStatus::Delayed(conversation) => return Ok((conversation, None)),
    };
    let stream = StreamData::new(db, bot, event, conversation_info);

    // let message_data = check_switch_bot(
    //     messages,
    //     next_bot,
    //     &mut conversation_info,
    //     &mut bot,
    //     &mut bot_opt,
    //     &mut event,
    //     db,
    // )
    //     .await?;
    //
    // let conversation = get_conversation(db, conversation_info.conversation_id).await?;

    Ok((conversation, Some(stream)))
}

pub async fn start_conversation_db<T: SeaOrmDbTraits + Send>(
    request: CsmlRequest,
    mut bot_opt: BotOpt,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(Conversation, Vec<MessageData>), EngineError> {
    let (mut conversation_info, conversation_status, mut bot, mut event) =
        setup_conversation(request, &mut bot_opt, db).await?;

    if let ConversationStatus::Delayed(conversation) = conversation_status {
        return Ok((conversation, vec![]));
    }

    let (messages, next_bot) =
        interpret_step(&mut conversation_info, event.clone(), &bot, db).await?;

    let message_data = check_switch_bot(
        messages,
        next_bot,
        &mut conversation_info,
        &mut bot,
        &mut bot_opt,
        &mut event,
        db,
    )
    .await?;

    let conversation = get_conversation(db, conversation_info.conversation_id).await?;
    Ok((conversation, message_data))
}

async fn setup_conversation<T: SeaOrmDbTraits + Send>(
    request: CsmlRequest,
    bot_opt: &mut BotOpt,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(ConversationInfo, ConversationStatus, CsmlBot, Event), EngineError> {
    let mut formatted_event = format_event(&request)?;

    let mut bot = bot_opt.search_bot_async(db).await?;
    init_bot(&mut bot)?;

    let mut data = init_conversation_info(
        get_default_flow(&bot)?.name.clone(),
        &formatted_event,
        &request,
        &bot,
        db,
    )
    .await?;

    check_for_hold(&mut data, &bot, &mut formatted_event, db).await?;

    /////////// block user event if delay variable is on and delay_time is bigger than current time
    let delay = delay_user_event(db, &mut bot, &mut data).await?;
    if let Some(conversation) = delay {
        return Ok((
            data,
            ConversationStatus::Delayed(conversation),
            bot,
            formatted_event,
        ));
    }
    //////////////////////////////////////

    // save event in db as message RECEIVE
    if !data.low_data {
        let message = if formatted_event.secure {
            serde_json::json!({"content_type": "secure"})
        } else {
            request.payload
        };

        messages::add_messages_bulk(&mut data, vec![message], 0, Direction::Receive, db).await?;
    }
    Ok((data, ConversationStatus::Started, bot, formatted_event))
}

async fn delay_user_event<T: SeaOrmDbTraits + Send>(
    db: &mut AsyncDatabase<'_, T>,
    bot: &mut CsmlBot,
    data: &mut ConversationInfo,
) -> Result<Option<Conversation>, EngineError> {
    if let Some(delay) = bot.no_interruption_delay {
        if let Some(delay) = state::get_state_key(&data.client, "delay", "content", db).await?
            && let (Some(delay), Some(timestamp)) =
                (delay["delay_value"].as_i64(), delay["timestamp"].as_i64())
            && timestamp + delay >= Utc::now().timestamp()
        {
            let conversation = get_conversation(db, data.conversation_id).await?;
            return Ok(Some(conversation));
        }

        let delay: serde_json::Value = serde_json::json!({
            "delay_value": delay,
            "timestamp": Utc::now().timestamp()
        });

        set_state_items(
            &data.client,
            "delay",
            vec![("content", &delay)],
            data.ttl,
            db,
        )
        .await?;
    }
    Ok(None)
}

fn check_switch_bot<'a, T: SeaOrmDbTraits + Send>(
    mut messages: Vec<MessageData>,
    next_bot: Option<SwitchBot>,
    data: &'a mut ConversationInfo,
    bot: &'a mut CsmlBot,
    bot_opt: &'a mut BotOpt,
    event: &'a mut Event,
    db: &'a mut AsyncDatabase<'_, T>,
) -> BoxFuture<'a, Result<Vec<MessageData>, EngineError>> {
    async move {
        match next_bot {
            Some(next_bot) => {
                println!("switching bot");
                println!("Current payloads: {:?}", data.payloads);

                if let Err(err) = switch_bot(data, bot, next_bot, bot_opt, event, db).await {
                    // End no interruption delay
                    if bot.no_interruption_delay.is_some() {
                        delete_state_key(&data.client, "delay", "content", db).await?;
                    }
                    return Err(err);
                }

                let (message_data, next_bot) = interpret_step(data, event.clone(), bot, db).await?;

                let mut new_messages =
                    check_switch_bot(message_data, next_bot, data, bot, bot_opt, event, db).await?;

                messages.append(&mut new_messages);
            }
            None => {
                // End no interruption delay
                if bot.no_interruption_delay.is_some() {
                    delete_state_key(&data.client, "delay", "content", db).await?;
                }
            }
        }
        Ok(messages)
    }
    .boxed()
}

pub async fn get_client_messages_filtered<'a, 'conn: 'a, 'b, T: SeaOrmDbTraits>(
    db: &'a mut AsyncDatabase<'conn, T>,
    filter: ClientMessageFilter<'b>,
) -> Result<Paginated<Message>, EngineError> {
    messages::get_client_messages(db, filter).await
}

pub async fn get_conversation<T: SeaOrmDbTraits>(
    db: &mut AsyncDatabase<'_, T>,
    id: Uuid,
) -> Result<Conversation, EngineError> {
    conversations::get_conversation(db, id).await
}

/**
 * Verify if the user is currently on hold in a given conversation.
 *
 * If a hold is found, make sure that the flow has not been updated since last conversation.
 * If that's the case, we can not be sure that the hold is in the same position,
 * so we need to clear the hold's position and restart the conversation.
 *
 * If the hold is valid, we also need to load the local step memory
 * (`context.hold.step_vars`) into the conversation context.
 */
async fn check_for_hold<T: SeaOrmDbTraits>(
    data: &mut ConversationInfo,
    bot: &CsmlBot,
    event: &mut Event,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    if let Ok(Some(hold)) = state::get_state_key(&data.client, "hold", "position", db).await {
        let Some(hash_value) = hold.get("hash") else {
            return Ok(());
        };

        let flow_hash = get_current_step_hash(&data.context, bot)?;
        // cleanup the current hold and restart flow
        if flow_hash != *hash_value {
            return clean_hold_and_restart(data, db).await;
        }

        let Ok(index) = serde_json::from_value::<IndexInfo>(hold["index"].clone()) else {
            delete_state_key(&data.client, "hold", "position", db).await?;
            return Ok(());
        };

        let secure_hold = hold["secure"].as_bool().unwrap_or(false);

        if secure_hold {
            event.secure = true;
        }

        // all good, let's load the position and local variables
        data.context.hold = Some(Hold {
            index,
            step_vars: hold["step_vars"].clone(),
            step_name: data.context.step.get_step().to_owned(),
            flow_name: data.context.flow.clone(),
            previous: serde_json::from_value(hold["previous"].clone()).unwrap_or(None),
            secure: secure_hold,
        });

        delete_state_key(&data.client, "hold", "position", db).await?;
    }
    Ok(())
}
