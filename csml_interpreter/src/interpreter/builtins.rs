mod api;
mod crypto;
mod exists;
mod format;
mod functions;
pub(crate) mod http_builtin;
mod jwt;
mod smtp;
mod time;

mod tools;

use crate::data::{
    ArgsType, Data, Literal, MSG, MessageData,
    ast::Interval,
    position::Position,
    tokens::{
        APP, BASE64, CRYPTO, DEBUG, EXISTS, FIND, FLOOR, FN, HEX, HTTP, JWT, LENGTH, ONE_OF,
        OR_BUILT_IN, RANDOM, SHUFFLE, SMTP, TIME, UUID,
    },
};
use crate::error_format::{ERROR_NATIVE_COMPONENT, ErrorInfo, gen_error_info};
use crate::interpreter::variable_handler::gen_generic_component::gen_generic_component;
use std::sync::mpsc;

use crate::data::primitive::{PrimitiveObject, PrimitiveType};
use api::api;
use crypto::crypto;
use exists::exists;
use format::{base64, debug, hex, object};
use functions::{find, floor, length, one_of, or, random, shuffle, uuid_command};
use http_builtin::http;
use jwt::jwt;
use smtp::smtp;
use std::collections::HashMap;
use time::time;
// use uri::*;

pub(crate) fn match_native_builtin(
    name: &str,
    args: &ArgsType,
    interval: Interval,
    data: &mut Data,
) -> Result<Literal, ErrorInfo> {
    if let Some(component) = data.native_component.get(name) {
        gen_generic_component(name, false, &data.context.flow, &interval, args, component)
    } else {
        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("{ERROR_NATIVE_COMPONENT} [{name}]"),
        ))
    }
}

pub(crate) fn match_builtin(
    name: &str,
    args: ArgsType,
    interval: Interval,
    data: &mut Data,
    msg_data: &mut MessageData,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<Literal, ErrorInfo> {
    match name {
        HTTP => http(args, &data.context.flow, interval),
        SMTP => smtp(args, &data.context.flow, interval),
        BASE64 => base64(args, &data.context.flow, interval),
        HEX => hex(args, &data.context.flow, interval),
        FN | APP => api(args, interval, data, msg_data, sender),
        ONE_OF => one_of(args, &data.context.flow, interval),
        OR_BUILT_IN => or(args, &data.context.flow, interval),
        SHUFFLE => shuffle(args, &data.context.flow, interval),
        LENGTH => length(args, &data.context.flow, interval),
        FIND => find(args, &data.context.flow, interval),
        RANDOM => Ok(random(interval)),
        DEBUG => Ok(debug(args, interval)),
        FLOOR => floor(args, &data.context.flow, interval),
        UUID => uuid_command(args, &data.context.flow, interval),
        JWT => jwt(args, &data.context.flow, interval),
        CRYPTO => crypto(args, &data.context.flow, interval),
        TIME => Ok(time(interval)),
        EXISTS => exists(args, data, interval),

        //old builtin
        _object => object(args, &data.context.flow, interval),
    }
}

fn typed_object(
    mut args: ArgsType,
    arg_name: &str,
    content_type: &str,
    flow_name: &str,
    interval: Interval,
    error: &str,
) -> Result<Literal, ErrorInfo> {
    let Some(literal) = args.remove_typed(arg_name, 0, PrimitiveType::PrimitiveString) else {
        return Err(gen_error_info(
            Position::new(interval, flow_name),
            error.to_owned(),
        ));
    };

    let object: HashMap<String, Literal> = HashMap::from([(arg_name.to_owned(), literal)]);

    let result = PrimitiveObject::get_literal_with_type(content_type, object, interval);

    Ok(result)
}
