use crate::future::db_connectors::{
    conversations::{close_conversation, update_conversation},
    memories::{add_memories, delete_client_memories, delete_client_memory},
    messages::add_messages_bulk,
    state::set_state_items,
};
use crate::future::utils::send_msg_to_callback_url;

use crate::data::models::interpreter_actions::{InterpreterReturn, SwitchBot};
use crate::data::models::{Direction, MessageData};
use crate::data::{AsyncDatabase, ConversationInfo, EngineError, SeaOrmDbTraits};
use crate::utils::{
    get_current_step_hash, get_flow_by_id, messages_formatter, update_current_context,
};
use csml_interpreter::data::Memory;
use csml_interpreter::data::context::ContextStepInfo;
use csml_interpreter::{
    data::{
        Client, Event, Hold, MSG, Message, MultiBot, ast::ForgetMemory, csml_bot::CsmlBot,
        csml_flow::CsmlFlow, csml_logs::LogLvl,
    },
    interpret,
};

use async_stream::try_stream;
use futures::{Stream, StreamExt, pin_mut, stream};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, mpsc};
use tokio::task;

fn interpreter_stream(receiver: mpsc::Receiver<MSG>) -> impl Stream<Item = MSG> {
    stream::unfold(receiver, |receiver| async move {
        let res = task::spawn_blocking(move || {
            // Receive will only return an error if the sender has been dropped
            let received = receiver.recv().ok();
            (received, receiver)
        })
        .await;
        match res {
            Ok((received, receiver)) => {
                if let Some(received) = received {
                    return Some((received, receiver));
                }
            }
            Err(error) => {
                tracing::error!(error = &error as &dyn Error, "could not read file");
            }
        }
        None
    })
}

async fn flush<T: SeaOrmDbTraits>(
    data: &mut ConversationInfo,
    memories: &mut HashMap<String, Memory>,
    offset: &mut usize,
    interaction_order: u32,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(client = ?data.client, flow = &data.context.flow, "flushing messages");

    if !data.low_data {
        let msgs: Vec<Value> = data.payloads[*offset..]
            .iter()
            .map(Message::message_to_json)
            .collect();
        add_messages_bulk(data, msgs, interaction_order, Direction::Send, db).await?;
    }
    *offset = data.payloads.len();

    add_memories(data, memories, db).await?;
    memories.clear();

    Ok(())
}

pub enum InterpreterMessage {
    Message(MessageData),
    SwitchBot(SwitchBot),
}

pub fn format_messages(
    vec_msg: &[Message],
    start: usize,
    interaction_order: u32,
) -> Result<Vec<MessageData>, EngineError> {
    vec_msg
        .iter()
        .enumerate()
        .skip(start)
        .map(|(message_order, msg)| {
            let message_order = u32::try_from(message_order).map_err(|_| {
                EngineError::Internal(format!("message_order is too large ({message_order})"))
            });
            message_order.map(|message_order| {
                crate::utils::add_info_to_message(msg.clone(), message_order, interaction_order)
            })
        })
        .collect()
}

pub async fn stream_step<'a, 'dbref, 'db: 'dbref, 'conv, 'bot, T: SeaOrmDbTraits>(
    data: &'conv mut ConversationInfo,
    event: &'bot Arc<Event>,
    bot: &'bot Arc<CsmlBot>,
    db: &'dbref mut AsyncDatabase<'db, T>,
) -> Result<
    impl Stream<Item = Result<InterpreterMessage, EngineError>> + use<'db, 'dbref, 'conv, 'bot, T>,
    EngineError,
> {
    let mut current_flow: &CsmlFlow = get_flow_by_id(&data.context.flow, &bot.flows)?;
    let mut interaction_order = 0;
    let mut conversation_end = false;
    let (sender, receiver) = mpsc::channel::<MSG>();
    let context = data.context.clone();
    tracing::debug!(client = ?data.client, flow = &data.context.flow, bot_id = bot.id, "start interpretation");
    let new_bot = Arc::clone(bot);

    let event = Arc::clone(event);
    let interpreter_handle = task::spawn_blocking(move || {
        let _ = interpret(&new_bot, context, &event, Some(&sender));
    });

    let stream = try_stream! {

        let mut switch_bot = None;
        let msg_stream = interpreter_stream(receiver);
        pin_mut!(msg_stream);

        let mut message_index = 0;
        let mut memories = HashMap::new();

        while let Some(received) = msg_stream.next().await {
            match received {
                MSG::Remember(mem) => {
                    memories.insert(mem.key.clone(), mem);
                }
                MSG::Forget(mem) => match mem {
                    ForgetMemory::ALL => {
                        memories.clear();
                        delete_client_memories(&data.client, db).await?;
                    }
                    ForgetMemory::SINGLE(memory) => {
                        memories.remove(&memory.ident);
                        delete_client_memory(&data.client, &memory.ident, db).await?;
                    }
                    ForgetMemory::LIST(mem_list) => {
                        for mem in &mem_list {
                            memories.remove(&mem.ident);
                            delete_client_memory(&data.client, &mem.ident, db).await?;
                        }
                    }
                },
                MSG::Message(msg) => {
                    tracing::debug!(client = ?data.client, flow = &data.context.flow, message = ?msg, "sending message");

                    send_msg_to_callback_url(data, vec![msg.clone()], interaction_order).await?;
                    data.payloads.push(msg);
                }
                MSG::Log {
                    flow,
                    line,
                    message,
                    log_lvl,
                } => {
                    match log_lvl {
                        LogLvl::Info => {
                            tracing::info!(client = ?data.client, flow, line, message, "csml log call");
                        }
                        LogLvl::Warn => {
                            tracing::warn!(client = ?data.client, flow, line, message, "csml log call");
                        }
                        LogLvl::Error => {
                            tracing::error!(client = ?data.client, flow, line, message, "csml log call");
                        }
                        LogLvl::Debug => {
                            tracing::debug!(client = ?data.client, flow, line, message, "csml log call");
                        }
                        LogLvl::Trace => {
                            tracing::trace!(client = ?data.client, flow, line, message, "csml log call");
                        }
                    }
                }
                MSG::Hold(Hold {
                              index,
                              step_vars,
                              step_name,
                              flow_name,
                              previous,
                              secure,
                          }) => {
                    let hash = get_current_step_hash(&data.context, bot)?;
                    let state_hold: Value = serde_json::json!({
                        "index": index,
                        "step_vars": step_vars,
                        "hash": hash,
                        "previous": previous,
                        "secure": secure
                    });

                    tracing::debug!(client = ?data.client, flow = &data.context.flow, state = ?state_hold, "hold bot");

                    set_state_items(&data.client, "hold", vec![("position", &state_hold)], data.ttl, db).await?;
                    data.context.hold = Some(Hold {
                        index,
                        step_vars,
                        step_name,
                        flow_name,
                        previous,
                        secure,
                    });
                }
                MSG::Next { flow, step, bot: None } => {
                    if let Ok(InterpreterReturn::End) = manage_internal_goto(
                        data,
                        &mut conversation_end,
                        &mut interaction_order,
                        &mut current_flow,
                        bot,
                        &mut memories,
                        flow,
                        step,
                        db,
                    )
                        .await
                    {
                        break;
                    }
                }

                MSG::Next {
                    flow,
                    step,
                    bot: Some(target_bot),
                } => {
                    if let Ok(InterpreterReturn::SwitchBot(s_bot)) =
                        manage_switch_bot(data, &mut interaction_order, bot, flow, step, target_bot, db).await
                    {
                        switch_bot = Some(s_bot);
                        break;
                    }
                }

                MSG::Error(err_msg) => {
                    conversation_end = true;
                    tracing::error!(client = ?data.client, flow = &data.context.flow, error = ?err_msg, "interpreter error");

                    send_msg_to_callback_url(data, vec![err_msg.clone()], interaction_order).await?;
                    data.payloads.push(err_msg);
                    close_conversation(data.conversation_id, &data.client, db).await?;
                }

                MSG::Flush => {
                    let old_index = message_index;

                    flush(data, &mut memories, &mut message_index, interaction_order, db).await?;

                    for message in format_messages(&data.payloads, old_index, interaction_order)? {
                        yield InterpreterMessage::Message(message);
                    }
                }
            }
        }
        match interpreter_handle.await {
        Ok(()) => {
            tracing::debug!(client = ?data.client, flow = &data.context.flow, "interpreter finished");
        }
        Err(error) => {
            tracing::error!(error = &error as &dyn Error, "interpreter error");
            Err(EngineError::Interpreter(error.to_string()))?;
        }
        }

        let old_index = message_index;

        flush(data, &mut memories, &mut message_index, interaction_order, db).await?;

        for message in format_messages(&data.payloads, old_index, interaction_order)? {
            yield InterpreterMessage::Message(message);
        }

        if let Some(switch_bot) = switch_bot {
            yield InterpreterMessage::SwitchBot(switch_bot);
        }
    };
    Ok(stream)
}

/**
 * This is the CSML Engine action.
 * A request came in and should be handled. Once the `ConversationInfo` is correctly setup,
 * this step is called in a loop until a `hold` or `goto end` is reached.
 */
pub async fn interpret_step<T: SeaOrmDbTraits>(
    data: &mut ConversationInfo,
    event: Event,
    bot: &CsmlBot,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(Vec<MessageData>, Option<SwitchBot>), EngineError> {
    let mut current_flow: &CsmlFlow = get_flow_by_id(&data.context.flow, &bot.flows)?;
    let mut interaction_order = 0;
    let mut conversation_end = false;
    let (sender, receiver) = mpsc::channel::<MSG>();
    let context = data.context.clone();
    let mut switch_bot = None;

    tracing::debug!(client = ?data.client, flow = &data.context.flow, bot_id = bot.id, "start interpretation");
    let new_bot = bot.clone();

    let interpreter_handle = task::spawn_blocking(move || {
        let _ = interpret(&new_bot, context, &event, Some(&sender));
    });

    let msg_stream = interpreter_stream(receiver);
    pin_mut!(msg_stream);

    let mut message_index = 0;
    let mut memories = HashMap::new();

    while let Some(received) = msg_stream.next().await {
        match received {
            MSG::Remember(mem) => {
                memories.insert(mem.key.clone(), mem);
            }
            MSG::Forget(mem) => match mem {
                ForgetMemory::ALL => {
                    memories.clear();
                    delete_client_memories(&data.client, db).await?;
                }
                ForgetMemory::SINGLE(memory) => {
                    memories.remove(&memory.ident);
                    delete_client_memory(&data.client, &memory.ident, db).await?;
                }
                ForgetMemory::LIST(mem_list) => {
                    for mem in &mem_list {
                        memories.remove(&mem.ident);
                        delete_client_memory(&data.client, &mem.ident, db).await?;
                    }
                }
            },
            MSG::Message(msg) => {
                tracing::debug!(client = ?data.client, flow = &data.context.flow, message = ?msg, "sending message");

                send_msg_to_callback_url(data, vec![msg.clone()], interaction_order).await?;
                data.payloads.push(msg);
            }
            MSG::Log {
                flow,
                line,
                message,
                log_lvl,
            } => match log_lvl {
                LogLvl::Info => {
                    tracing::info!(client = ?data.client, flow, line, message, "csml log call");
                }
                LogLvl::Warn => {
                    tracing::warn!(client = ?data.client, flow, line, message, "csml log call");
                }
                LogLvl::Error => {
                    tracing::error!(client = ?data.client, flow, line, message, "csml log call");
                }
                LogLvl::Debug => {
                    tracing::debug!(client = ?data.client, flow, line, message, "csml log call");
                }
                LogLvl::Trace => {
                    tracing::trace!(client = ?data.client, flow, line, message, "csml log call");
                }
            },
            MSG::Hold(Hold {
                index,
                step_vars,
                step_name,
                flow_name,
                previous,
                secure,
            }) => {
                let hash = get_current_step_hash(&data.context, bot)?;
                let state_hold: Value = serde_json::json!({
                    "index": index,
                    "step_vars": step_vars,
                    "hash": hash,
                    "previous": previous,
                    "secure": secure
                });

                tracing::debug!(client = ?data.client, flow = &data.context.flow, state = ?state_hold, "hold bot");

                set_state_items(
                    &data.client,
                    "hold",
                    vec![("position", &state_hold)],
                    data.ttl,
                    db,
                )
                .await?;
                data.context.hold = Some(Hold {
                    index,
                    step_vars,
                    step_name,
                    flow_name,
                    previous,
                    secure,
                });
            }
            MSG::Next {
                flow,
                step,
                bot: None,
            } => {
                if let Ok(InterpreterReturn::End) = manage_internal_goto(
                    data,
                    &mut conversation_end,
                    &mut interaction_order,
                    &mut current_flow,
                    bot,
                    &mut memories,
                    flow,
                    step,
                    db,
                )
                .await
                {
                    break;
                }
            }

            MSG::Next {
                flow,
                step,
                bot: Some(target_bot),
            } => {
                if let Ok(InterpreterReturn::SwitchBot(s_bot)) = manage_switch_bot(
                    data,
                    &mut interaction_order,
                    bot,
                    flow,
                    step,
                    target_bot,
                    db,
                )
                .await
                {
                    switch_bot = Some(s_bot);
                    break;
                }
            }

            MSG::Error(err_msg) => {
                conversation_end = true;
                tracing::error!(client = ?data.client, flow = &data.context.flow, error = ?err_msg, "interpreter error");

                send_msg_to_callback_url(data, vec![err_msg.clone()], interaction_order).await?;
                data.payloads.push(err_msg);
                close_conversation(data.conversation_id, &data.client, db).await?;
            }

            MSG::Flush => {
                flush(
                    data,
                    &mut memories,
                    &mut message_index,
                    interaction_order,
                    db,
                )
                .await?;
            }
        }
    }

    match interpreter_handle.await {
        Ok(()) => {
            tracing::debug!(client = ?data.client, flow = &data.context.flow, "interpreter finished");
        }
        Err(error) => {
            tracing::error!(error = &error as &dyn Error, "interpreter error");
            return Err(EngineError::Interpreter(error.to_string()));
        }
    }

    flush(
        data,
        &mut memories,
        &mut message_index,
        interaction_order,
        db,
    )
    .await?;

    Ok((
        messages_formatter(data.payloads.clone(), interaction_order)?,
        switch_bot,
    ))
}

async fn manage_switch_bot<T: SeaOrmDbTraits>(
    data: &mut ConversationInfo,
    interaction_order: &mut u32,
    bot: &'_ CsmlBot,
    flow: Option<String>,
    step: Option<ContextStepInfo>,
    target_bot: String,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<InterpreterReturn, EngineError> {
    // check if we are allow to switch to 'target_bot'

    let next_bot = if let Some(multibot) = &bot.multibot {
        multibot.iter().find(
            |&MultiBot {
                 id,
                 name,
                 version_id: _,
             }| match name {
                Some(name) => target_bot == *id || target_bot == *name,
                None => target_bot == *id,
            },
        )
    } else {
        None
    };

    let Some(next_bot) = next_bot else {
        let error_message = format!("Switching to Bot: ({target_bot}) is not allowed");
        // send message
        send_msg_to_callback_url(
            data,
            vec![Message {
                content_type: "error".to_owned(),
                content: serde_json::json!({ "error": error_message }),
            }],
            *interaction_order,
        )
        .await?;

        tracing::error!(
            flow = data.context.flow,
            target_bot,
            "switching to bot not allowed"
        );
        return Ok(InterpreterReturn::End);
    };

    let (flow, step) = match (flow, step) {
        (Some(flow), Some(step)) => {
            let step_name = step.get_step();
            tracing::debug!(client = ?data.client, target_flow = flow, target_step = step_name, target_bot_id = target_bot, from_flow = &data.context.flow, from_step = data.context.step.get_step(), from_bot_id = bot.id, "going to flow start");
            (Some(flow), step)
        }
        (Some(flow), None) => {
            tracing::debug!(client = ?data.client, target_flow = flow, target_bot_id = target_bot, from_flow = &data.context.flow, from_step = data.context.step.get_step(), from_bot_id = bot.id, "going to flow start");
            (Some(flow), ContextStepInfo::Normal("start".to_owned()))
        }
        (None, Some(step)) => {
            let step_name = step.get_step();
            tracing::debug!(client = ?data.client, target_step = step_name, target_bot_id = target_bot, from_flow = &data.context.flow, from_step = data.context.step.get_step(), from_bot_id = bot.id, "going to default flow");
            (None, step)
        }
        (None, None) => {
            tracing::debug!(client = ?data.client, target_bot_id = target_bot, from_flow = &data.context.flow, from_step = data.context.step.get_step(), from_bot_id = bot.id, "going to start in default flow");

            (None, ContextStepInfo::Normal("start".to_owned()))
        }
    };

    let message = Message::switch_bot_message(&next_bot.id, &data.client);
    // save message
    data.payloads.push(message.clone());
    // send message switch bot
    send_msg_to_callback_url(data, vec![message], *interaction_order).await?;

    tracing::debug!(client = ?data.client, flow = &data.context.flow, "switching bot");

    close_conversation(data.conversation_id, &data.client, db).await?;

    let previous_bot: Value = serde_json::json!({
        "bot": data.client.bot_id,
        "flow": data.context.flow,
        "step": data.context.step,
    });

    set_state_items(
        &Client {
            bot_id: next_bot.id.clone(),
            channel_id: data.client.channel_id.clone(),
            user_id: data.client.user_id.clone(),
        },
        "bot",
        vec![("previous", &previous_bot)],
        data.ttl,
        db,
    )
    .await?;

    Ok(InterpreterReturn::SwitchBot(SwitchBot {
        bot_id: next_bot.id.clone(),
        version_id: next_bot.version_id,
        flow,
        step: step.get_step().to_owned(),
    }))
}

#[allow(clippy::too_many_arguments)]
async fn manage_internal_goto<'a, T: SeaOrmDbTraits>(
    data: &mut ConversationInfo,
    conversation_end: &mut bool,
    interaction_order: &mut u32,
    current_flow: &mut &'a CsmlFlow,
    bot: &'a CsmlBot,
    memories: &mut HashMap<String, Memory>,
    flow: Option<String>,
    step: Option<ContextStepInfo>,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<InterpreterReturn, EngineError> {
    match (flow, step) {
        (Some(flow), Some(step)) => {
            tracing::debug!(client = ?data.client, target_flow = flow, target_flow = step.get_step(), from_flow = data.context.flow, from_step = data.context.step.get_step(), "goto flow & step");

            // TODO handle error
            update_current_context(data, memories).unwrap();
            goto_flow(data, interaction_order, current_flow, bot, flow, step, db).await?;
        }
        (Some(flow), None) => {
            tracing::debug!(client = ?data.client, target_flow = flow, from_flow = data.context.flow, from_step = data.context.step.get_step(), "goto flow start");

            // TODO handle error
            update_current_context(data, memories).unwrap();
            let step = ContextStepInfo::Normal("start".to_owned());

            goto_flow(data, interaction_order, current_flow, bot, flow, step, db).await?;
        }
        (None, Some(step)) => {
            tracing::debug!(client = ?data.client, flow = &data.context.flow, target_step = step.get_step(), from_step = data.context.step.get_step(), "goto step");
            if goto_step(data, conversation_end, interaction_order, step, db).await? {
                return Ok(InterpreterReturn::End);
            }
        }
        (None, None) => {
            tracing::debug!(client = ?data.client, flow = &data.context.flow, step = data.context.step.get_step(), "goto end");

            let step = ContextStepInfo::Normal("end".to_owned());
            if goto_step(data, conversation_end, interaction_order, step, db).await? {
                return Ok(InterpreterReturn::End);
            }
        }
    }

    Ok(InterpreterReturn::Continue)
}

/**
 * CSML `goto flow` action
 */
async fn goto_flow<'a, T: SeaOrmDbTraits>(
    data: &mut ConversationInfo,
    interaction_order: &mut u32,
    current_flow: &mut &'a CsmlFlow,
    bot: &'a CsmlBot,
    nextflow: String,
    nextstep: ContextStepInfo,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<(), EngineError> {
    tracing::debug!(client = ?data.client, target_flow = &nextflow, target_step = nextstep.get_step(), from_flow = data.context.flow, from_step = data.context.step.get_step(), "performing goto flow");

    *current_flow = get_flow_by_id(&nextflow, &bot.flows)?;
    data.context.flow = nextflow;
    data.context.step = nextstep;

    update_conversation(
        data.conversation_id,
        Some(&current_flow.id),
        Some(data.context.step.get_step()),
        db,
    )
    .await?;

    *interaction_order += 1;

    Ok(())
}

/**
 * CSML `goto step` action
 */
async fn goto_step<T: SeaOrmDbTraits>(
    data: &mut ConversationInfo,
    conversation_end: &mut bool,
    interaction_order: &mut u32,
    nextstep: ContextStepInfo,
    db: &mut AsyncDatabase<'_, T>,
) -> Result<bool, EngineError> {
    tracing::debug!(client = ?data.client, flow = &data.context.flow, target_step = nextstep.get_step(), from_step = data.context.step.get_step(), "performing goto step");

    if nextstep.is_end() {
        *conversation_end = true;

        // send end of conversation
        send_msg_to_callback_url(data, vec![], *interaction_order).await?;
        close_conversation(data.conversation_id, &data.client, db).await?;

        // break interpret_step loop
        return Ok(*conversation_end);
    }

    data.context.step = nextstep;
    update_conversation(
        data.conversation_id,
        None,
        Some(data.context.step.get_step()),
        db,
    )
    .await?;

    *interaction_order += 1;
    Ok(false)
}
