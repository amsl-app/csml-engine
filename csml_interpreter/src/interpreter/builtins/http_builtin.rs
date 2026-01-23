use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::{PrimitiveInt, PrimitiveObject, PrimitiveString, PrimitiveType};
use crate::data::{ArgsType, Literal, ast::Interval};
use crate::error_format::{
    ERROR_FAIL_RESPONSE_JSON, ERROR_HTTP, ERROR_HTTP_GET_VALUE, ERROR_HTTP_UNKNOWN_METHOD,
    gen_error_info,
};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::hash::BuildHasher;

use reqwest::StatusCode;
use reqwest::blocking::{RequestBuilder, Response};

fn get_value<'lifetime, T: 'static, S: BuildHasher>(
    key: &str,
    object: &'lifetime HashMap<String, Literal, S>,
    flow_name: &str,
    interval: Interval,
    error: &'static str,
) -> Result<&'lifetime T, ErrorInfo> {
    if let Some(literal) = object.get(key) {
        Literal::get_value::<T, _>(
            &literal.primitive,
            flow_name,
            interval,
            format!("'{key}' {error}"),
        )
    } else {
        Err(gen_error_info(
            Position::new(interval, flow_name),
            format!("'{key}' {error}"),
        ))
    }
}

fn set_http_error_info(
    response_info: &HashMap<String, Literal>,
    error_message: String,
    flow_name: &str,
    interval: Interval,
) -> ErrorInfo {
    let mut error = gen_error_info(Position::new(interval, flow_name), error_message);
    error.add_info_block(response_info.clone());

    error
}

fn get_request_info(response: &Response, interval: Interval) -> HashMap<String, Literal> {
    let mut response_info = HashMap::new();

    let status = PrimitiveInt::get_literal(i64::from(response.status().as_u16()), interval);
    response_info.insert("status".to_owned(), status);

    let headers = response
        .headers()
        .iter()
        .fold(HashMap::new(), |mut acc, (header, value)| {
            if let Ok(value) = value.to_str() {
                let value = PrimitiveString::get_literal(value, interval);
                acc.insert(header.to_string(), value);
            }
            acc
        });

    response_info.insert(
        "headers".to_owned(),
        PrimitiveObject::get_literal(headers, interval),
    );

    response_info
}

#[must_use]
pub(crate) fn get_ssl_state<S: BuildHasher>(object: &HashMap<String, Literal, S>) -> bool {
    match object.get("disable_ssl_verify") {
        Some(val) if val.primitive.get_type() == PrimitiveType::PrimitiveBoolean => {
            val.primitive.as_bool()
        }
        _ => false,
    }
}

////////////////////////////////////////////////////////////////////////////////
// pub(crate)LIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub(crate) fn get_url<S: BuildHasher>(
    object: &HashMap<String, Literal, S>,
    flow_name: &str,
    interval: Interval,
) -> Result<String, ErrorInfo> {
    let url: &String = get_value("url", object, flow_name, interval, ERROR_HTTP_GET_VALUE)?;
    let url = &mut url.clone();

    if object.get("query").is_some() {
        let query: &HashMap<String, Literal> =
            get_value("query", object, flow_name, interval, ERROR_HTTP_GET_VALUE)?;

        let length = query.len();
        if length > 0 {
            url.push('?');

            for (index, key) in query.keys().enumerate() {
                let value = match query.get(key) {
                    Some(val) => val.primitive.to_string(),
                    None => {
                        return Err(gen_error_info(
                            Position::new(interval, flow_name),
                            format!("'{key}' {ERROR_HTTP_GET_VALUE}"),
                        ));
                    }
                };

                url.push_str(key);
                url.push('=');
                url.push_str(&value);

                if index + 1 < length {
                    url.push('&');
                }
            }
        }
    }

    Ok(url.clone())
}

fn get_http_request(
    method: &str,
    url: &str,
    flow_name: &str,
    interval: Interval,
    is_ssl_disable: bool,
) -> Result<RequestBuilder, ErrorInfo> {
    let mut client_builder = reqwest::blocking::Client::builder();
    if let Ok(disable_ssl_verify) = env::var("DISABLE_SSL_VERIFY") {
        match disable_ssl_verify.parse::<bool>() {
            Ok(low_data) if low_data || is_ssl_disable => {
                client_builder = client_builder.danger_accept_invalid_certs(true);
            }
            _ => {}
        }
    }

    let request = match method {
        "delete" => reqwest::Method::DELETE,
        "put" => reqwest::Method::PUT,
        "patch" => reqwest::Method::PATCH,
        "post" => reqwest::Method::POST,
        "get" => reqwest::Method::GET,
        _ => {
            return Err(gen_error_info(
                Position::new(interval, flow_name),
                ERROR_HTTP_UNKNOWN_METHOD.to_string(),
            ));
        }
    };

    let client = match client_builder.build() {
        Ok(client) => client,
        Err(err) => {
            return Err(gen_error_info(
                Position::new(interval, flow_name),
                err.to_string(),
            ));
        }
    };

    Ok(client.request(request, url))
}

const APP_ERROR_TEXT: &str = "Apps service: error";

fn get_status_message(status: StatusCode, is_app_call: bool) -> String {
    if is_app_call {
        APP_ERROR_TEXT.to_string()
    } else {
        status.as_u16().to_string()
    }
}

fn get_error_message(error: &reqwest::Error, is_app_call: bool) -> String {
    if is_app_call {
        APP_ERROR_TEXT.to_string()
    } else {
        error.to_string()
    }
}

pub(crate) fn http_request<S: BuildHasher>(
    object: &HashMap<String, Literal, S>,
    method: &str,
    flow_name: &str,
    interval: Interval,
    is_app_call: bool,
) -> Result<(serde_json::Value, HashMap<String, Literal>), ErrorInfo> {
    let url = get_url(object, flow_name, interval)?;
    let is_ssl_disable = get_ssl_state(object);

    let header: &HashMap<String, Literal> =
        get_value("header", object, flow_name, interval, ERROR_HTTP_GET_VALUE)?;

    let mut request = get_http_request(method, &url, flow_name, interval, is_ssl_disable)?;

    for key in header.keys() {
        let Some(value) = header.get(key) else {
            return Err(gen_error_info(
                Position::new(interval, flow_name),
                format!("'{key}' {ERROR_HTTP_GET_VALUE}"),
            ));
        };
        let value = value.primitive.to_string();

        request = request.header(key, &value);
    }

    tracing::debug!(
        flow_name,
        line = interval.start_line,
        ?request,
        "make http call"
    );

    let response = {
        let request_builder = match object.get("body") {
            Some(body) => request.json(&body.primitive.to_json()),
            None => request,
        };
        request_builder.send()
    };

    let response = response.and_then(Response::error_for_status);

    match response {
        Ok(response) => {
            let response_info = get_request_info(&response, interval);

            let status = response.status();
            if !status.is_success() {
                tracing::debug!(flow_name, line = interval.start_line, %status, "http call failed");

                let response_info = get_request_info(&response, interval);

                let mut error = set_http_error_info(
                    &response_info,
                    get_status_message(status, is_app_call),
                    flow_name,
                    interval,
                );

                let body = response.text().unwrap_or_else(|_| {
                    "Invalid Response format, please send a json or a valid UTF-8 sequence"
                        .to_owned()
                });

                error.add_info("body", PrimitiveString::get_literal(&body, interval));

                return Err(error);
            }

            match response.text() {
                Ok(string_value) => {
                    match serde_json::from_str::<serde_json::Value>(&string_value) {
                        Ok(json_value) => Ok((json_value, response_info)),
                        Err(_) => Ok((serde_json::json!(string_value), response_info)),
                    }
                }
                Err(err) => {
                    tracing::error!(
                        error = &err as &dyn Error,
                        flow_name,
                        line = interval.start_line,
                        "http response Json parsing failed"
                    );

                    let mut error = set_http_error_info(
                        &response_info,
                        ERROR_FAIL_RESPONSE_JSON.to_owned(),
                        flow_name,
                        interval,
                    );

                    let error_body =
                        "Invalid Response format, please send a json or a valid UTF-8 sequence";
                    error.add_info("body", PrimitiveString::get_literal(error_body, interval));
                    Err(error)
                }
            }
        }
        Err(err) => {
            let error_message = get_error_message(&err, is_app_call);
            tracing::error!(
                error = &err as &dyn Error,
                flow_name,
                line = interval.start_line,
                "http call failed"
            );
            Err(gen_error_info(
                Position::new(interval, flow_name),
                error_message,
            ))
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn http(
    args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    if let Some(literal) = args.get("url", 0)
        && literal.primitive.get_type() == PrimitiveType::PrimitiveString
    {
        let mut http: HashMap<String, Literal> = HashMap::new();
        let mut header = HashMap::new();

        header.insert(
            "Content-Type".to_owned(),
            PrimitiveString::get_literal("application/json", interval),
        );
        header.insert(
            "Accept".to_owned(),
            PrimitiveString::get_literal("application/json,text/*", interval),
        );
        header.insert(
            "User-Agent".to_owned(),
            PrimitiveString::get_literal("csml/v1", interval),
        );

        http.insert("url".to_owned(), literal.clone());
        http.insert(
            "method".to_owned(),
            PrimitiveString::get_literal("get", interval),
        );

        let lit_header = PrimitiveObject::get_literal(header, interval);
        http.insert("header".to_owned(), lit_header);

        args.populate(
            &mut http,
            &["url", "header", "query", "body"],
            flow_name,
            interval,
        )?;

        let mut result = PrimitiveObject::get_literal(http, interval);

        result.set_content_type("http");

        return Ok(result);
    }
    Err(gen_error_info(
        Position::new(interval, flow_name),
        ERROR_HTTP.to_owned(),
    ))
}
