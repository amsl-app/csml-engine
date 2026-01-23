use crate::data::core::PreviousInfo;
use crate::data::position::Position;
use crate::data::warnings::DisplayWarnings;
use crate::data::{
    Literal, MSG, Memory, MemoryType, MessageData,
    ast::{
        AssignType, DoType, Expr, GotoType, InsertStep, InstructionScope, Interval, ObjectType,
        PathLiteral, PathState, PreviousType,
    },
    context::ContextStepInfo,
    core::Data,
    literal::ContentType,
    message::{Message, MessageType},
    primitive::{PrimitiveNull, PrimitiveString, closure::capture_variables},
};
use crate::error_format::{
    ERROR_GET_VAR_INFO, ERROR_START_INSTRUCTIONS, ErrorInfo, gen_error_info,
};
use crate::interpreter::variable_handler::{
    exec_path_actions, expr_to_literal,
    forget_memories::{forget_scope_memories, remove_message_data_memories},
    get_var_from_mem,
    interval::{interval_from_expr, interval_from_reserved_fn},
    memory::{save_literal_in_mem, search_in_memory_type},
    resolve_fn_args, search_goto_var_memory,
};
use crate::parser::ExitCondition;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

#[allow(clippy::type_complexity)]
fn get_var_info<'a>(
    expr: &'a Expr,
    path: Option<&[(Interval, PathState)]>,
    data: &'a mut Data,
    msg_data: &mut MessageData,
    sender: Option<&Sender<MSG>>,
) -> Result<
    (
        &'a mut Literal,
        String,
        MemoryType,
        Option<Vec<(Interval, PathLiteral)>>,
    ),
    ErrorInfo,
> {
    match expr {
        Expr::PathExpr { literal, path } => {
            get_var_info(literal, Some(path), data, msg_data, sender)
        }
        Expr::IdentExpr(var) => {
            if search_in_memory_type(var, data).is_err() {
                // If a variable doesn't exist,
                // create the variable in the flow scope 'use' with NULL as value
                // this is done to prevent stopping errors
                let lit = PrimitiveNull::get_literal(var.interval);

                data.step_vars.insert(var.ident.clone(), lit);
            }
            get_var_from_mem(
                var.clone(),
                DisplayWarnings::On,
                path,
                data,
                msg_data,
                sender,
            )
        }
        _ => Err(gen_error_info(
            Position::new(interval_from_expr(expr), &data.context.flow),
            ERROR_GET_VAR_INFO.to_owned(),
        )),
    }
}

fn check_if_inserted_step(name: &str, interval: &Interval, data: &Data) -> Option<String> {
    match data
        .flow
        .flow_instructions
        .get_key_value(&InstructionScope::InsertStep(InsertStep {
            name: name.to_owned(),
            original_name: None,
            from_flow: String::new(),
            interval: *interval,
        })) {
        Some((InstructionScope::InsertStep(insert_step), _expr)) => {
            Some(insert_step.from_flow.clone())
        }
        _ => None,
    }
}

fn check_secure(
    mut msg_data: MessageData,
    data: &mut Data,
    sender: Option<&Sender<MSG>>,
    lit: Literal,
) -> Result<MessageData, ErrorInfo> {
    if lit.secure_variable {
        let err = gen_error_info(
            Position::new(lit.interval, &data.context.flow),
            "Secure variable can not be displayed".to_owned(),
        );

        MSG::send_error_msg(sender, &mut msg_data, Err(err));
        Ok(msg_data)
    } else {
        let msg = Message::new(lit, &data.context.flow)?;
        MSG::send(sender, MSG::Message(msg.clone()));
        Ok(Message::add_to_message(msg_data, MessageType::Msg(msg)))
    }
}

pub(crate) fn match_actions(
    function: &ObjectType,
    mut msg_data: MessageData,
    data: &mut Data,
    sender: Option<&Sender<MSG>>,
) -> Result<MessageData, ErrorInfo> {
    match function {
        ObjectType::Say(arg) => {
            let lit = expr_to_literal(arg, DisplayWarnings::On, None, data, &mut msg_data, sender)?;

            // check if it is secure variable
            check_secure(msg_data, data, sender, lit)
        }
        ObjectType::Debug(args, interval) => {
            let args = resolve_fn_args(args, data, &mut msg_data, DisplayWarnings::On, sender)?;

            let lit = args.args_to_debug(*interval);

            // check if it is secure variable
            check_secure(msg_data, data, sender, lit)
        }
        ObjectType::Log {
            expr,
            interval,
            log_lvl,
        } => {
            let args = resolve_fn_args(expr, data, &mut msg_data, DisplayWarnings::On, sender)?;
            let log_msg = args.args_to_log();

            MSG::send(
                sender,
                MSG::Log {
                    flow: data.context.flow.clone(),
                    line: interval.start_line,
                    message: log_msg,
                    log_lvl: *log_lvl,
                },
            );

            Ok(msg_data)
        }
        ObjectType::Use(arg) => {
            expr_to_literal(arg, DisplayWarnings::On, None, data, &mut msg_data, sender)?;
            Ok(msg_data)
        }
        ObjectType::Do(DoType::Update(assign_type, old, new)) => {
            // ######################
            // create a temporary scope
            let (
                tmp_default_flow,
                mut tmp_context,
                tmp_event,
                tmp_env,
                tmp_loop_indexes,
                tmp_loop_index,
                mut tmp_step_count,
                tmp_step_limit,
                tmp_step_vars,
            ) = data.copy_scope();

            let mut new_scope_data = Data::new(
                data.flows,
                data.extern_flows,
                data.flow,
                tmp_default_flow,
                &mut tmp_context,
                &tmp_event,
                &tmp_env,
                tmp_loop_indexes,
                tmp_loop_index,
                &mut tmp_step_count,
                tmp_step_limit,
                tmp_step_vars,
                data.previous_info.clone(),
                data.custom_component,
                data.native_component,
            );
            // #####################

            let mut new_value =
                expr_to_literal(new, DisplayWarnings::On, None, data, &mut msg_data, sender)?;

            // check if it is secure variable
            if new_value.secure_variable {
                let err = gen_error_info(
                    Position::new(new_value.interval, &data.context.flow),
                    "Assignation of secure variable is not allowed".to_owned(),
                );

                MSG::send_error_msg(sender, &mut msg_data, Err(err));
                return Ok(msg_data);
            }

            // only for closure capture the step slot
            let memory: HashMap<String, Literal> = data.get_all_memories();
            capture_variables(&mut new_value, memory, &data.context.flow);

            let (lit, name, mem_type, path) = get_var_info(old, None, data, &mut msg_data, sender)?;

            let primitive = match assign_type {
                AssignType::AdditionAssignment => {
                    Some(lit.primitive.clone() + new_value.primitive.clone())
                }
                AssignType::SubtractionAssignment => {
                    Some(lit.primitive.clone() - new_value.primitive.clone())
                }
                AssignType::DivisionAssignment => {
                    Some(lit.primitive.clone() / new_value.primitive.clone())
                }
                AssignType::MultiplicationAssignment => {
                    Some(lit.primitive.clone() * new_value.primitive.clone())
                }
                AssignType::RemainderAssignment => {
                    Some(lit.primitive.clone() % new_value.primitive.clone())
                }
                AssignType::Assignment => None,
            };

            match primitive {
                Some(Ok(primitive)) => {
                    new_value = Literal {
                        content_type: new_value.content_type,
                        interval: new_value.interval,
                        additional_info: None,
                        secure_variable: false,
                        primitive,
                    };
                }
                Some(Err(err)) => {
                    new_value = PrimitiveString::get_literal(&err, lit.interval);
                    MSG::send_error_msg(
                        sender,
                        &mut msg_data,
                        Err(gen_error_info(
                            Position::new(new_value.interval, &new_scope_data.context.flow),
                            err,
                        )),
                    );
                }
                None => {}
            }

            //TODO: refactor memory update system

            let (new_value, update) = if let MemoryType::Constant = mem_type {
                MSG::send_error_msg(
                    sender,
                    &mut msg_data,
                    Err(gen_error_info(
                        Position::new(new_value.interval, &new_scope_data.context.flow),
                        "const slot are immutable".to_string(),
                    )),
                );

                (None, false)
            } else {
                (Some(new_value), true)
            };

            exec_path_actions(
                lit,
                DisplayWarnings::On,
                &mem_type,
                new_value,
                path,
                &ContentType::get(lit),
                &mut new_scope_data,
                &mut msg_data,
                sender,
            )?;

            save_literal_in_mem(
                lit.clone(),
                name,
                &mem_type,
                update,
                data,
                &mut msg_data,
                sender,
            );

            Ok(msg_data)
        }
        ObjectType::Do(DoType::Exec(expr)) => {
            expr_to_literal(expr, DisplayWarnings::On, None, data, &mut msg_data, sender)?;
            Ok(msg_data)
        }
        ObjectType::Goto(GotoType::Step(step), interval) => {
            let step = search_goto_var_memory(step, &mut msg_data, data, sender)?;

            // previous flow/step
            match data.previous_info {
                Some(ref mut previous_info) => {
                    previous_info.goto(data.context.flow.clone(), data.context.step.clone());
                }
                None => {
                    data.previous_info = Some(PreviousInfo::new(
                        data.context.flow.clone(),
                        data.context.step.clone(),
                    ));
                }
            }

            let insert_from_flow = check_if_inserted_step(&step, interval, data);

            // current flow/step
            data.context.step = match insert_from_flow {
                Some(from_flow) => ContextStepInfo::InsertedStep {
                    step: step.clone(),
                    flow: from_flow,
                },
                None => ContextStepInfo::Normal(step.clone()),
            };

            MSG::send(
                sender,
                MSG::Next {
                    flow: None,
                    step: Some(data.context.step.clone()),
                    bot: None,
                },
            );

            msg_data.exit_condition = Some(ExitCondition::Goto);

            if step == "end" {
                msg_data.exit_condition = Some(ExitCondition::End);
            }

            Ok(msg_data)
        }
        ObjectType::Goto(GotoType::Flow(flow), ..) => {
            let flow = search_goto_var_memory(flow, &mut msg_data, data, sender)?;

            MSG::send(
                sender,
                MSG::Next {
                    flow: Some(flow.clone()),
                    step: None,
                    bot: None,
                },
            );

            // previous flow/step
            match data.previous_info {
                Some(ref mut previous_info) => {
                    previous_info.goto(data.context.flow.clone(), data.context.step.clone());
                }
                None => {
                    data.previous_info = Some(PreviousInfo::new(
                        data.context.flow.clone(),
                        data.context.step.clone(),
                    ));
                }
            }
            // current flow/step
            data.context.step = ContextStepInfo::Normal("start".to_string());
            data.context.flow = flow;

            msg_data.exit_condition = Some(ExitCondition::Goto);

            Ok(msg_data)
        }
        ObjectType::Goto(
            GotoType::StepFlow {
                step,
                flow,
                bot: None,
            },
            interval,
        ) => {
            let step = match step {
                Some(step) => search_goto_var_memory(step, &mut msg_data, data, sender)?,
                None => "start".to_owned(), // default value start step
            };
            let flow = match flow {
                Some(flow) => search_goto_var_memory(flow, &mut msg_data, data, sender)?,
                None => data.context.flow.clone(), // default value current flow
            };

            let mut flow_opt = Some(flow.clone());

            msg_data.exit_condition = Some(ExitCondition::Goto);

            if step == "end" {
                msg_data.exit_condition = Some(ExitCondition::End);
                flow_opt = None;
            }

            // previous flow/step
            match data.previous_info {
                Some(ref mut previous_info) => {
                    previous_info.goto(data.context.flow.clone(), data.context.step.clone());
                }
                None => {
                    data.previous_info = Some(PreviousInfo::new(
                        data.context.flow.clone(),
                        data.context.step.clone(),
                    ));
                }
            }

            // current flow/step
            data.context.flow = flow;

            let insert_from_flow = check_if_inserted_step(&step, interval, data);

            // current flow/step
            data.context.step = match insert_from_flow {
                Some(from_flow) => ContextStepInfo::InsertedStep {
                    step: step.clone(),
                    flow: from_flow,
                },
                None => ContextStepInfo::Normal(step.clone()),
            };

            MSG::send(
                sender,
                MSG::Next {
                    flow: flow_opt,
                    step: Some(data.context.step.clone()),
                    bot: None,
                },
            );

            Ok(msg_data)
        }

        ObjectType::Goto(
            GotoType::StepFlow {
                step,
                flow,
                bot: Some(next_bot),
            },
            ..,
        ) => {
            let step = match step {
                Some(step) => ContextStepInfo::UnknownFlow(search_goto_var_memory(
                    step,
                    &mut msg_data,
                    data,
                    sender,
                )?),
                None => ContextStepInfo::Normal("start".to_owned()), // default value start step
            };
            let flow = match flow {
                Some(flow) => search_goto_var_memory(flow, &mut msg_data, data, sender).ok(),
                None => None,
            };

            let bot = search_goto_var_memory(next_bot, &mut msg_data, data, sender)?;

            msg_data.exit_condition = Some(ExitCondition::End);

            MSG::send(
                sender,
                MSG::Next {
                    step: Some(step),
                    flow,
                    bot: Some(bot), // need to send previous flow / step / bot info
                },
            );

            Ok(msg_data)
        }

        ObjectType::Previous(previous_type, _) => {
            let flow_opt;
            let mut step_opt = None;

            match (previous_type, &mut data.previous_info) {
                (PreviousType::Flow(_interval), Some(previous_info)) => {
                    let tmp_f = previous_info.flow.clone();
                    flow_opt = Some(tmp_f.clone());

                    previous_info.flow.clone_from(&data.context.flow);
                    previous_info.step_at_flow =
                        (data.context.step.clone(), data.context.flow.clone());

                    data.context.flow = tmp_f;
                    data.context.step = ContextStepInfo::Normal("start".to_string());
                }
                (PreviousType::Step(_interval), Some(previous_info)) => {
                    let (tmp_s, tmp_f) = previous_info.step_at_flow.clone();
                    flow_opt = Some(tmp_f.clone());
                    step_opt = Some(tmp_s.clone());

                    if data.context.flow != tmp_f {
                        previous_info.flow.clone_from(&tmp_f);
                    }
                    previous_info.step_at_flow =
                        (data.context.step.clone(), data.context.flow.clone());

                    data.context.flow = tmp_f;
                    data.context.step = tmp_s;
                }
                (_, None) => {
                    flow_opt = None;
                    step_opt = Some(ContextStepInfo::Normal("end".to_owned()));

                    data.context.step = ContextStepInfo::Normal("end".to_string());
                }
            }

            msg_data.exit_condition = Some(ExitCondition::Goto);

            MSG::send(
                sender,
                MSG::Next {
                    flow: flow_opt,
                    step: step_opt,
                    bot: None,
                },
            );

            Ok(msg_data)
        }
        ObjectType::Remember(name, variable) => {
            let mut new_value = expr_to_literal(
                variable,
                DisplayWarnings::On,
                None,
                data,
                &mut msg_data,
                sender,
            )?;

            // check if it is secure variable
            if new_value.secure_variable {
                let err = gen_error_info(
                    Position::new(new_value.interval, &data.context.flow),
                    "Assignation of secure variable is not allowed".to_owned(),
                );

                MSG::send_error_msg(sender, &mut msg_data, Err(err));
                return Ok(msg_data);
            }

            // only for closure capture the step slot
            let memory: HashMap<String, Literal> = data.get_all_memories();
            capture_variables(&mut new_value, memory, &data.context.flow);

            msg_data.add_to_memory(&name.ident, &new_value);

            MSG::send(
                sender,
                MSG::Remember(Memory::new(name.ident.clone(), new_value.clone())),
            );

            data.context.current.insert(name.ident.clone(), new_value);
            Ok(msg_data)
        }
        ObjectType::Forget(memory, _interval) => {
            // delete memories form message data
            remove_message_data_memories(memory, &mut msg_data);
            // delete memory from current scope
            forget_scope_memories(memory, data);

            MSG::send(sender, MSG::Forget(memory.clone()));

            Ok(msg_data)
        }

        reserved => Err(gen_error_info(
            Position::new(interval_from_reserved_fn(reserved), &data.context.flow),
            ERROR_START_INSTRUCTIONS.to_owned(),
        )),
    }
}
