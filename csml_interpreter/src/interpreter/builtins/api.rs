use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::{PrimitiveObject, PrimitiveString, PrimitiveType};
use crate::data::{ApiInfo, ArgsType, Client, Data, Literal, MSG, MessageData, ast::Interval};
use crate::error_format::{ERROR_FN_ENDPOINT, ERROR_FN_ID, ERROR_HTTP_NOT_DATA, gen_error_info};
use crate::interpreter::{
    builtins::{http_builtin::http_request, tools::client_to_json},
    json_to_rust::interpolate,
};

use std::{collections::HashMap, sync::mpsc};

#[allow(clippy::needless_pass_by_value)]
fn format_body(
    args: ArgsType,
    flow_name: &str,
    interval: Interval,
    client: &Client,
) -> Result<Literal, ErrorInfo> {
    let mut map: HashMap<String, Literal> = HashMap::new();

    match args.get("fn_id", 0) {
        Some(literal) if literal.primitive.get_type() == PrimitiveType::PrimitiveString => {
            let fn_id = Literal::get_value::<String, _>(
                &literal.primitive,
                flow_name,
                literal.interval,
                ERROR_FN_ID,
            )?;

            map.insert(
                "function_id".to_owned(),
                PrimitiveString::get_literal(fn_id, interval),
            );
        }
        _ => {
            return Err(gen_error_info(
                Position::new(interval, flow_name),
                ERROR_FN_ID.to_owned(),
            ));
        }
    }
    let mut sub_map = HashMap::new();
    args.populate(&mut sub_map, &["fn_id"], flow_name, interval)?;

    let client = client_to_json(client, interval);

    map.insert(
        "data".to_owned(),
        PrimitiveObject::get_literal(sub_map, interval),
    );
    map.insert(
        "client".to_owned(),
        PrimitiveObject::get_literal(client, interval),
    );

    Ok(PrimitiveObject::get_literal(map, interval))
}

fn format_headers(interval: Interval) -> HashMap<String, Literal> {
    let mut header = HashMap::new();
    header.insert(
        "content-type".to_owned(),
        PrimitiveString::get_literal("application/json", interval),
    );
    header.insert(
        "accept".to_owned(),
        PrimitiveString::get_literal("application/json,text/*", interval),
    );

    header
}

pub(crate) fn api(
    args: ArgsType,
    interval: Interval,
    data: &mut Data,
    msg_data: &mut MessageData,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<Literal, ErrorInfo> {
    let (client, url) = match &data.context.api_info {
        Some(ApiInfo {
            client,
            apps_endpoint,
        }) => (client.clone(), apps_endpoint.clone()),
        None => {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_FN_ENDPOINT.to_owned(),
            ));
        }
    };

    let mut http: HashMap<String, Literal> = HashMap::new();
    let header = format_headers(interval);
    let body = format_body(args, &data.context.flow, interval, &client)?;

    http.insert(
        "url".to_owned(),
        PrimitiveString::get_literal(&url, interval),
    );

    let lit_header = PrimitiveObject::get_literal(header, interval);
    http.insert("header".to_owned(), lit_header);
    http.insert("body".to_owned(), body);

    // flush the message queue before making the http request
    MSG::flush(sender);
    let (value, response_info) =
        match http_request(&http, "post", &data.context.flow, interval, true) {
            Ok((value, response_info)) => (value, response_info),
            Err(err) => return Ok(MSG::send_error_msg(sender, msg_data, Err(err))),
        };
    let literal = if let Some(value) = value.get("data") {
        let mut literal = interpolate(value, interval, data, msg_data, sender)?;
        // add additional information about the http request response: status and headers
        literal.add_info_block(response_info);

        literal
    } else {
        let err = gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_HTTP_NOT_DATA.to_owned(),
        );
        MSG::send_error_msg(sender, msg_data, Err(err))
    };
    Ok(literal)
}
