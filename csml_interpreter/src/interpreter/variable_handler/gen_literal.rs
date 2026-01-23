use crate::data::literal::ContentType;
use crate::data::position::Position;
use crate::data::{
    Data, Literal, MSG, MemoryType, MessageData,
    ast::{Interval, PathState},
};
use crate::data::{ast::PathLiteral, primitive::PrimitiveString, warnings::DisplayWarnings};
use crate::error_format::{
    ERROR_COMPONENT_NAMESPACE, ERROR_COMPONENT_UNKNOWN, ERROR_EVENT_CONTENT_TYPE, ErrorInfo,
    gen_error_info,
};
use crate::interpreter::variable_handler::gen_generic_component::gen_generic_component;
use crate::interpreter::{
    json_to_rust::json_to_literal,
    variable_handler::{exec_path_actions, resolve_path},
};
use std::sync::mpsc;

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn gen_literal_from_event(
    interval: Interval,
    dis_warnings: DisplayWarnings,
    path: Option<&[(Interval, PathState)]>,
    data: &mut Data,
    msg_data: &mut MessageData,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<Literal, ErrorInfo> {
    let Some(path) = path else {
        let mut lit = PrimitiveString::get_literal(&data.event.content_value, interval);

        lit.secure_variable = data.event.secure;

        return Ok(lit);
    };
    let path = resolve_path(path, dis_warnings, data, msg_data, sender)?;
    let mut lit = json_to_literal(&data.event.content, interval, &data.context.flow)?;

    lit.set_content_type("event");

    let content_type = match ContentType::get(&lit) {
        ContentType::Event(_) => ContentType::Event(data.event.content_type.clone()),
        _ => {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_EVENT_CONTENT_TYPE.to_owned(),
            ));
        }
    };

    let (lit, _tmp_mem_update) = exec_path_actions(
        &mut lit,
        dis_warnings,
        &MemoryType::Event("event".to_owned()),
        None,
        Some(path),
        &content_type,
        data,
        msg_data,
        sender,
    )?;

    Ok(lit)
}

pub fn gen_literal_from_component(
    interval: Interval,
    path: Option<&[(Interval, PathState)]>,
    data: &mut Data,
    msg_data: &mut MessageData,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<Literal, ErrorInfo> {
    let Some(path) = path else {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_COMPONENT_NAMESPACE.to_owned(),
        ));
    };
    let mut path = resolve_path(path, DisplayWarnings::On, data, msg_data, sender)?;

    if let Some((
        _interval,
        PathLiteral::Func {
            name,
            interval,
            args,
        },
    )) = path.first()
        && let Some(component) = data.custom_component.get(name)
    {
        let mut lit =
            gen_generic_component(name, true, &data.context.flow, interval, args, component)?;

        path.drain(..1);

        let (lit, _tmp_mem_update) = exec_path_actions(
            &mut lit,
            DisplayWarnings::On,
            &MemoryType::Use,
            None,
            Some(path),
            &ContentType::Primitive,
            data,
            msg_data,
            sender,
        )?;

        return Ok(lit);
    }

    Err(gen_error_info(
        Position::new(interval, &data.context.flow),
        ERROR_COMPONENT_UNKNOWN.to_owned(),
    ))
}
