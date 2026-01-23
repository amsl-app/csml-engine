pub mod ast_interpreter;
pub mod builtins;
pub mod components;
pub mod function_scope;
pub mod json_to_rust;
pub mod variable_handler;

pub use json_to_rust::{json_to_literal, memory_to_literal};

use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::{
    Data, Hold, IndexInfo, Literal, MSG, MessageData,
    ast::{Block, Expr, ObjectType},
    warnings::DisplayWarnings,
};
use crate::error_format::{ERROR_START_INSTRUCTIONS, gen_error_info};
use crate::interpreter::{
    ast_interpreter::{for_loop, match_actions, solve_if_statement, while_loop},
    variable_handler::{expr_to_literal, interval::interval_from_expr},
};
use crate::parser::ExitCondition;

use nom::lib::std::collections::HashMap;
use std::sync::mpsc;

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTION
////////////////////////////////////////////////////////////////////////////////

fn step_vars_to_json(map: &HashMap<String, Literal>) -> serde_json::Value {
    serde_json::Value::Object(
        map.iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    value.primitive.format_mem(&value.content_type, true),
                )
            })
            .collect(),
    )
}

pub(crate) fn interpret_scope(
    actions: &Block,
    data: &mut Data,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<MessageData, ErrorInfo> {
    let mut message_data = MessageData::default();

    for (action, instruction_info) in &actions.commands {
        let instruction_total = instruction_info.index + instruction_info.total;

        if let Some(hold) = &mut data.context.hold {
            if hold.index.command_index > instruction_total {
                continue;
            } else if hold.index.command_index == instruction_info.index {
                data.context.hold = None;
                continue; // this command is the hold, we need to skip it to continue the conversation
            }
        }

        if message_data.exit_condition.is_some() {
            return Ok(message_data);
        }

        let handle_hold = |mut message_data: MessageData, secure: bool| {
            let index = instruction_info.index;

            let hold = Hold::new(
                IndexInfo {
                    command_index: index,
                    loop_index: data.loop_indexes.clone(),
                },
                step_vars_to_json(&data.step_vars),
                data.context.step.get_step().to_owned(),
                data.context.flow.clone(),
                data.previous_info.clone(),
                secure,
            );

            message_data.hold = Some(hold.clone());

            MSG::send(sender, MSG::Hold(hold));
            message_data.exit_condition = Some(ExitCondition::Hold);
            message_data
        };

        match action {
            Expr::ObjectExpr(ObjectType::Return(var)) => {
                let lit = expr_to_literal(
                    var,
                    DisplayWarnings::On,
                    None,
                    data,
                    &mut message_data,
                    None,
                )?;
                message_data.exit_condition = Some(ExitCondition::Return(lit));

                return Ok(message_data);
            }
            Expr::ObjectExpr(ObjectType::Break(..)) => {
                message_data.exit_condition = Some(ExitCondition::Break);

                return Ok(message_data);
            }
            Expr::ObjectExpr(ObjectType::Continue(..)) => {
                message_data.exit_condition = Some(ExitCondition::Continue);

                return Ok(message_data);
            }
            Expr::ObjectExpr(ObjectType::Hold(..)) => {
                return Ok(handle_hold(message_data, false));
            }
            Expr::ObjectExpr(ObjectType::HoldSecure(..)) => {
                return Ok(handle_hold(message_data, true));
            }
            Expr::ObjectExpr(fun) => message_data = match_actions(fun, message_data, data, sender)?,
            Expr::IfExpr(if_statement) => {
                message_data =
                    solve_if_statement(if_statement, message_data, data, instruction_info, sender)?;
            }
            Expr::ForEachExpr(ident, index, expr, block, range) => {
                message_data = for_loop(
                    ident,
                    index.as_ref(),
                    expr,
                    block,
                    range,
                    message_data,
                    data,
                    sender,
                )?;
            }
            Expr::WhileExpr(expr, block, range) => {
                message_data = while_loop(expr, block, range, message_data, data, sender)?;
            }
            e => {
                return Err(gen_error_info(
                    Position::new(interval_from_expr(e), &data.context.flow),
                    ERROR_START_INSTRUCTIONS.to_owned(),
                ));
            }
        }
    }

    Ok(message_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Interval;
    use crate::data::primitive::PrimitiveObject;

    #[test]
    fn test_step_vars_to_json() {
        let data = HashMap::from([
            (
                "var1".to_owned(),
                Literal {
                    content_type: "test-1".to_string(),
                    primitive: Box::new(PrimitiveObject::new(HashMap::new())),
                    additional_info: None,
                    secure_variable: false,
                    interval: Interval::default(),
                },
            ),
            (
                "var2".to_owned(),
                Literal {
                    content_type: "test-2".to_string(),
                    primitive: Box::new(PrimitiveObject::new(HashMap::new())),
                    additional_info: None,
                    secure_variable: false,
                    interval: Interval::default(),
                },
            ),
        ]);
        let json = step_vars_to_json(&data);
        let expected = serde_json::json!({
            "var1": {
                "_content": {},
                "_content_type": "test-1",
            },
            "var2": {
                "_content": {},
                "_content_type": "test-2",
            },
        });
        assert_eq!(json, expected);
    }
}
