use crate::data::{
    Data, MSG, MessageData,
    ast::{Block, Expr, Interval},
};
use crate::error_format::ErrorInfo;
use crate::interpreter::{ast_interpreter::if_statement::valid_condition, interpret_scope};
use std::sync::mpsc;

pub(crate) fn while_loop(
    cond: &Expr,
    block: &Block,
    _range_interval: &Interval,
    mut msg_data: MessageData,
    data: &mut Data,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<MessageData, ErrorInfo> {
    while valid_condition(cond, data, &mut msg_data, sender) {
        msg_data = msg_data + interpret_scope(block, data, sender)?;

        if msg_data.branch().is_break() {
            break;
        }
    }

    Ok(msg_data)
}
