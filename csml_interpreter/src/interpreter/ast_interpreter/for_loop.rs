use crate::data::primitive::PrimitiveInt;
use crate::data::{
    Data, MSG, MessageData, Position,
    ast::{Block, Expr, Identifier, Interval},
    hold::{
        hold_index_end_loop, hold_index_start_loop, hold_loop_decrs_index, hold_loop_incrs_index,
    },
    primitive::tools::get_array,
    warnings::DisplayWarnings,
};
use crate::error_format::{ERROR_FOREACH, ERROR_FOREACH_INDEX_OVERFLOW, ErrorInfo, gen_error_info};
use crate::interpreter::interpret_scope;
use crate::interpreter::variable_handler::expr_to_literal::expr_to_literal;
use num_traits::ToPrimitive;
use std::sync::mpsc;

#[allow(clippy::too_many_arguments)]
pub(crate) fn for_loop(
    ident: &Identifier,
    index: Option<&Identifier>,
    expr: &Expr,
    block: &Block,
    _range_interval: &Interval,
    mut msg_data: MessageData,
    data: &mut Data,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<MessageData, ErrorInfo> {
    let literal = expr_to_literal(expr, DisplayWarnings::On, None, data, &mut msg_data, sender)?;
    let array = get_array(&literal, &data.context.flow, ERROR_FOREACH)?;

    let mut value_skipped = 0;
    let array = hold_index_start_loop(data, &array, &mut value_skipped);

    for (for_loop_index, elem) in array.iter().enumerate() {
        let for_loop_index = for_loop_index + value_skipped;
        data.step_vars.insert(ident.ident.clone(), elem.clone());
        if let Some(index) = index {
            data.step_vars.insert(
                index.ident.clone(),
                PrimitiveInt::get_literal(
                    for_loop_index.to_i64().ok_or(gen_error_info(
                        Position::new(elem.interval, &data.context.flow),
                        ERROR_FOREACH_INDEX_OVERFLOW.to_owned(),
                    ))?,
                    elem.interval,
                ),
            );
        }

        hold_loop_incrs_index(data, for_loop_index);
        msg_data = msg_data + interpret_scope(block, data, sender)?;
        hold_loop_decrs_index(data);

        if msg_data.branch().is_break() {
            break;
        }
    }

    hold_index_end_loop(data);
    data.step_vars.remove(&ident.ident);
    if let Some(index) = index {
        data.step_vars.remove(&index.ident);
    }
    Ok(msg_data)
}
