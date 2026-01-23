use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::common;
use crate::data::primitive::common::{get_args, get_int_args, get_string_args};
use crate::data::primitive::utils::{illegal_math_ops, require_n_args};
use crate::data::{
    Literal, MemoryType,
    ast::Interval,
    literal,
    literal::ContentType,
    message::Message,
    primitive::{
        Data, MSG, MessageData, Primitive, PrimitiveArray, PrimitiveBoolean, PrimitiveInt,
        PrimitiveNull, PrimitiveString, PrimitiveType, Right, tools_crypto, tools_jwt, tools_smtp,
        tools_time,
    },
    tokens::TYPES,
};
use crate::error_format::{
    ERROR_CONSTANT_MUTABLE_FUNCTION, ERROR_DIGEST, ERROR_DIGEST_ALGO, ERROR_HASH, ERROR_HASH_ALGO,
    ERROR_HMAC_KEY, ERROR_HTTP_QUERY, ERROR_HTTP_SEND, ERROR_HTTP_SET, ERROR_HTTP_UNKNOWN_METHOD,
    ERROR_JWT_ALGO, ERROR_JWT_DECODE_ALGO, ERROR_JWT_DECODE_SECRET, ERROR_JWT_SECRET,
    ERROR_JWT_SIGN_ALGO, ERROR_JWT_SIGN_CLAIMS, ERROR_JWT_SIGN_SECRET, ERROR_JWT_TOKEN,
    ERROR_JWT_VALIDATION_ALGO, ERROR_JWT_VALIDATION_CLAIMS, ERROR_JWT_VALIDATION_SECRETE,
    ERROR_OBJECT_ASSIGN, ERROR_OBJECT_CONTAINS, ERROR_OBJECT_GET_GENERICS, ERROR_OBJECT_INSERT,
    ERROR_OBJECT_REMOVE, ERROR_OBJECT_UNKNOWN_METHOD, ERROR_UNREACHABLE, OVERFLOWING_OPERATION,
    gen_error_info,
};
use crate::interpreter::{
    builtins::http_builtin::http_request, json_to_rust::json_to_literal,
    variable_handler::match_literals::match_obj,
};
use base64::Engine;
use chrono::{DateTime, FixedOffset, LocalResult, TimeZone, Timelike, Utc};
use chrono_tz::{Tz, UTC};
use lettre::Transport;
use num_traits::ToPrimitive;
use phf::phf_map;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::any::Any;
use std::cmp::Ordering;
use std::error::Error;
use std::sync::LazyLock;
use std::{collections::HashMap, sync::mpsc};
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

const FUNCTIONS_HTTP: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "set" => (PrimitiveObject::set as PrimitiveMethod, Right::Read),
    "disable_ssl_verify" => (PrimitiveObject::disable_ssl_verify as PrimitiveMethod, Right::Read),
    "auth" => (PrimitiveObject::auth as PrimitiveMethod, Right::Read),
    "query" => (PrimitiveObject::query as PrimitiveMethod, Right::Read),
    "get" => (PrimitiveObject::get_http as PrimitiveMethod, Right::Read),
    "post" => (PrimitiveObject::post as PrimitiveMethod, Right::Read),
    "put" => (PrimitiveObject::put as PrimitiveMethod, Right::Read),
    "delete" => (PrimitiveObject::delete as PrimitiveMethod, Right::Read),
    "patch" => (PrimitiveObject::patch as PrimitiveMethod, Right::Read),
    "send" => (PrimitiveObject::send as PrimitiveMethod, Right::Read),
};

const FUNCTIONS_SMTP: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "auth" => (PrimitiveObject::credentials as PrimitiveMethod, Right::Read),
    "port" => (PrimitiveObject::port as PrimitiveMethod, Right::Read),
    "tls" => (PrimitiveObject::smtp_tls as PrimitiveMethod, Right::Read),
    "starttls" => (PrimitiveObject::starttls as PrimitiveMethod, Right::Read),
    "set_auth_mechanism" => (PrimitiveObject::set_auth_mechanism as PrimitiveMethod, Right::Read),
    "send" => (PrimitiveObject::smtp_send as PrimitiveMethod, Right::Read),
};

const FUNCTIONS_TIME: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "at" => (PrimitiveObject::set_date_at as PrimitiveMethod, Right::Write),
    "with_timezone" => (PrimitiveObject::with_timezone as PrimitiveMethod, Right::Write),
    "unix" => (PrimitiveObject::unix as PrimitiveMethod, Right::Write),
    "add" => (PrimitiveObject::add_time as PrimitiveMethod, Right::Write),
    "sub" => (PrimitiveObject::sub_time as PrimitiveMethod, Right::Write),
    "format" => (PrimitiveObject::date_format as PrimitiveMethod, Right::Read),
    "parse" => (PrimitiveObject::parse_date as PrimitiveMethod, Right::Read),
};

const FUNCTIONS_JWT: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "sign" => (PrimitiveObject::jwt_sign as PrimitiveMethod, Right::Read),
    "decode" => (PrimitiveObject::jwt_decode as PrimitiveMethod, Right::Read),
    "verify" => (PrimitiveObject::jwt_verity as PrimitiveMethod, Right::Read),
};

const FUNCTIONS_CRYPTO: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "create_hmac" => (PrimitiveObject::create_hmac as PrimitiveMethod, Right::Read),
    "create_hash" => (PrimitiveObject::create_hash as PrimitiveMethod, Right::Read),
    "digest" => (PrimitiveObject::digest as PrimitiveMethod, Right::Read),
};

const FUNCTIONS_BASE64: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "encode" => (PrimitiveObject::base64_encode as PrimitiveMethod, Right::Read),
    "decode" => (PrimitiveObject::base64_decode as PrimitiveMethod, Right::Read),
};

const FUNCTIONS_HEX: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "encode" => (PrimitiveObject::hex_encode as PrimitiveMethod, Right::Read),
    "decode" => (PrimitiveObject::hex_decode as PrimitiveMethod, Right::Read),
};

const FUNCTIONS_EVENT: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "get_type" => (PrimitiveObject::get_type as PrimitiveMethod, Right::Read),
    "get_content" => (PrimitiveObject::get_content as PrimitiveMethod, Right::Read),
    "is_email" => (PrimitiveObject::is_email as PrimitiveMethod, Right::Read),
    "is_secure" => (PrimitiveObject::is_secure as PrimitiveMethod, Right::Read),
    "match" => (PrimitiveObject::match_args as PrimitiveMethod, Right::Read),
    "match_array" => (PrimitiveObject::match_array as PrimitiveMethod, Right::Read),
};

const FUNCTIONS_READ: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "is_number" => (PrimitiveObject::is_number as PrimitiveMethod, Right::Read),
    "is_int" => (PrimitiveObject::is_int as PrimitiveMethod, Right::Read),
    "is_float" => (PrimitiveObject::is_float as PrimitiveMethod, Right::Read),
    "type_of" => (PrimitiveObject::type_of as PrimitiveMethod, Right::Read),
    "get_info" => (PrimitiveObject::get_info as PrimitiveMethod, Right::Read),
    "is_error" => ((|_, _, additional_info, _, interval, _| common::is_error(additional_info, interval)) as PrimitiveMethod, Right::Read),
    "to_xml" => (PrimitiveObject::convert_to_xml as PrimitiveMethod, Right::Read),
    "to_yaml" => (PrimitiveObject::convert_to_yaml as PrimitiveMethod, Right::Read),
    "to_string" => (PrimitiveObject::convert_to_string as PrimitiveMethod, Right::Read),

    "contains" => (PrimitiveObject::contains as PrimitiveMethod, Right::Read),
    "is_empty" => (PrimitiveObject::is_empty as PrimitiveMethod, Right::Read),
    "length" => (PrimitiveObject::length as PrimitiveMethod, Right::Read),
    "keys" => (PrimitiveObject::keys as PrimitiveMethod, Right::Read),
    "values" => (PrimitiveObject::values as PrimitiveMethod, Right::Read),
    "get" => (PrimitiveObject::get_generics as PrimitiveMethod, Right::Read),

};

const FUNCTIONS_WRITE: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "clear_values" => (PrimitiveObject::clear_values as PrimitiveMethod, Right::Write),
    "insert" => (PrimitiveObject::insert as PrimitiveMethod, Right::Write),
    "assign" => (PrimitiveObject::assign as PrimitiveMethod, Right::Write),
    "remove" => (PrimitiveObject::remove as PrimitiveMethod, Right::Write),
};

type PrimitiveMethod = fn(
    object: &mut PrimitiveObject,
    args: HashMap<String, Literal>,
    additional_info: Option<&HashMap<String, Literal>>,
    data: &mut Data,
    interval: Interval,
    content_type: &str,
) -> Result<Literal, ErrorInfo>;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveObject {
    pub value: HashMap<String, Literal>,
}

static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap());

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveObject {
    fn set(
        object: &mut Self,
        mut args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "set(header: object) => http object";

        if args.len() != 1 {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let Some(literal) = args.remove("arg0") else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        };

        let mut object = object.clone();

        let header = Literal::get_value::<HashMap<String, Literal>, _>(
            &literal.primitive,
            &data.context.flow,
            interval,
            ERROR_HTTP_SET,
        )?;

        insert_to_object(
            header,
            &mut object,
            "header",
            &data.context.flow,
            literal.clone(),
        );

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("http");

        Ok(result)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn disable_ssl_verify(
        object: &mut Self,
        _args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        _data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let mut object = object.clone();

        let value = PrimitiveBoolean::get_literal(true, interval);
        object.value.insert("disable_ssl_verify".to_owned(), value);

        let mut result = Self::get_literal(object.value, interval);
        result.set_content_type("http");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn auth(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "auth(username, password) => http object";

        if args.len() < 2 {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let [username, password] = get_string_args(&args, data, interval, usage, usage)?;

        let user_password = format!("{username}:{password}");
        let authorization = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(user_password.as_bytes())
        );

        let mut object = object.clone();

        let mut header = HashMap::new();
        header.insert(
            "Authorization".to_owned(),
            PrimitiveString::get_literal(&authorization, interval),
        );
        let literal = Self::get_literal(header.clone(), interval);

        insert_to_object(&header, &mut object, "header", &data.context.flow, literal);

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("http");

        Ok(result)
    }

    fn query(
        object: &mut Self,
        mut args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "query(parameters: object) => http object";

        require_n_args(1, &args, interval, data, usage)?;

        let Some(literal) = args.remove("arg0") else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        };

        let mut object = object.clone();

        let header = Literal::get_value::<HashMap<String, Literal>, _>(
            &literal.primitive,
            &data.context.flow,
            interval,
            ERROR_HTTP_QUERY,
        )?;
        insert_to_object(
            header,
            &mut object,
            "query",
            &data.context.flow,
            literal.clone(),
        );

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("http");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_http(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "get() => http object")?;

        let mut object = object.clone();

        object.value.insert(
            "method".to_owned(),
            PrimitiveString::get_literal("get", interval),
        );

        object.value.remove("body");

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("http");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    fn post(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        _data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        match args.get("arg0") {
            Some(body) => object.value.insert("body".to_owned(), body.clone()),
            _ => object.value.remove("body"),
        };

        let mut object = object.clone();

        object.value.insert(
            "method".to_owned(),
            PrimitiveString::get_literal("post", interval),
        );

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("http");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    fn put(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        _data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        match args.get("arg0") {
            Some(body) => object.value.insert("body".to_owned(), body.clone()),
            _ => object.value.remove("body"),
        };

        let mut object = object.clone();

        object.value.insert(
            "method".to_owned(),
            PrimitiveString::get_literal("put", interval),
        );

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("http");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    fn delete(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        _data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        match args.get("arg0") {
            Some(body) => object.value.insert("body".to_owned(), body.clone()),
            _ => object.value.remove("body"),
        };

        let mut object = object.clone();

        object.value.insert(
            "method".to_owned(),
            PrimitiveString::get_literal("delete", interval),
        );

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("http");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    fn patch(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        _data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let body = match args.get("arg0") {
            Some(res) => res.clone(),
            _ => PrimitiveNull::get_literal(Interval::default()),
        };

        let mut object = object.clone();

        object.value.insert(
            "method".to_owned(),
            PrimitiveString::get_literal("patch", interval),
        );

        object.value.insert("body".to_owned(), body);

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("http");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn send(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "send() => http object")?;

        if let Some(literal) = object.value.get("method") {
            let method = match Literal::get_value::<String, _>(
                &literal.primitive,
                &data.context.flow,
                interval,
                ERROR_HTTP_UNKNOWN_METHOD.to_string(),
            ) {
                Ok(delete) if delete == "delete" => "delete",
                Ok(put) if put == "put" => "put",
                Ok(patch) if patch == "patch" => "patch",
                Ok(post) if post == "post" => "post",
                Ok(get) if get == "get" => "get",
                _ => {
                    return Err(gen_error_info(
                        Position::new(interval, &data.context.flow),
                        ERROR_HTTP_UNKNOWN_METHOD.to_string(),
                    ));
                }
            };

            let (value, response_info) =
                http_request(&object.value, method, &data.context.flow, interval, false)?;
            let mut literal = json_to_literal(&value, interval, &data.context.flow)?;
            // add additional information about the http request response: status and headers
            literal.add_info_block(response_info);

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_HTTP_SEND.to_owned(),
        ))
    }
}

impl PrimitiveObject {
    #[allow(clippy::needless_pass_by_value)]
    fn credentials(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "credentials(username, password) => smtp object";

        let [username, password] = get_string_args(&args, data, interval, usage, usage)?;

        let mut object = object.clone();

        object.value.insert(
            "username".to_owned(),
            PrimitiveString::get_literal(username, interval),
        );

        object.value.insert(
            "password".to_owned(),
            PrimitiveString::get_literal(password, interval),
        );

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("smtp");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn port(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "port(port) => smtp object";

        let [port]: [i64; 1] =
            get_int_args(&args, data, interval, format!("usage: {usage}"), usage)?;

        let mut object = object.clone();

        object
            .value
            .insert("port".to_owned(), PrimitiveInt::get_literal(port, interval));

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("smtp");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn smtp_tls(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "tls(BOOLEAN) => smtp object";
        require_n_args(1, &args, interval, data, usage)?;

        let [tls]: [&bool; 1] = get_args(&args, data, interval, usage, usage)?;

        let mut object = object.clone();

        object.value.insert(
            "tls".to_owned(),
            PrimitiveBoolean::get_literal(*tls, interval),
        );

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("smtp");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn set_auth_mechanism(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "Available mechanisms: PLAIN, AUTH LOGIN, XOAUTH2. set_auth_mechanism(String || Array<String>) => smtp object";

        let auth_mechanisms = match args.get("arg0") {
            Some(lit) if lit.content_type == "string" => {
                let value = tools_smtp::get_auth_mechanism(lit, data, interval, usage)?;

                let mut map = HashMap::new();
                map.insert(value, PrimitiveNull::get_literal(interval));

                map
            }
            Some(lit) if lit.content_type == "array" => {
                let vec = Literal::get_value::<Vec<Literal>, _>(
                    &lit.primitive,
                    &data.context.flow,
                    lit.interval,
                    format!("usage: {usage}"),
                )?;

                let map = vec
                    .iter()
                    .filter_map(|lit| {
                        tools_smtp::get_auth_mechanism(lit, data, interval, usage).ok()
                    })
                    .map(|val| (val, PrimitiveNull::get_literal(interval)))
                    .collect::<HashMap<String, Literal>>();

                if map.is_empty() {
                    return Err(gen_error_info(
                        Position::new(interval, &data.context.flow),
                        format!("usage: {usage}"),
                    ));
                }

                map
            }
            _ => {
                tracing::error!(?args, "set_auth_mechanism wrong mechanism");

                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    format!("usage: {usage}"),
                ));
            }
        };

        let mut object = object.clone();

        object.value.insert(
            "auth_mechanisms".to_owned(),
            Self::get_literal(auth_mechanisms, interval),
        );

        let mut result = Self::get_literal(object.value, interval);

        result.set_content_type("smtp");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn starttls(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "starttls(BOOLEAN) => smtp object";

        let [tls]: [&bool; 1] = get_args(&args, data, interval, usage, usage)?;

        let mut object = object.clone();

        object.value.insert(
            "starttls".to_owned(),
            PrimitiveBoolean::get_literal(*tls, interval),
        );

        let mut result = Self::get_literal(object.value.clone(), interval);

        result.set_content_type("smtp");

        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn smtp_send(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "send(email) => smtp object";

        let [csml_email]: [&HashMap<String, Literal>; 1] =
            get_args(&args, data, interval, usage, usage)?;

        let email = tools_smtp::format_email(csml_email, data, interval)?;
        tracing::debug!(?email, mailer = ?object.value, flow = data.context.flow, line = interval.start_line, "send email");
        let mailer = tools_smtp::get_mailer(&mut object.value, data, interval)?;

        match mailer.send(&email) {
            Ok(_) => Ok(PrimitiveBoolean::get_literal(true, interval)),
            Err(error) => {
                tracing::error!(error = &error as &dyn Error, "send email failed");
                Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    format!("Could not send email: {error:?}"),
                ))
            }
        }
    }
}

impl PrimitiveObject {
    #[allow(clippy::needless_pass_by_value)]
    fn set_date_at(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let date = tools_time::get_date(&args);

        let get_error = || {
            gen_error_info(
                Position::new(interval, &data.context.flow),
                "usage: at(Y, M, D, h, m, s, n) => date".to_string(),
            )
        };

        let cast_u32 = |val: i64| val.to_u32().ok_or_else(get_error);

        let nanos = cast_u32(date[6])?.checked_mul(1_000_000);

        let date = nanos
            .map(|nanos| -> Result<_, ErrorInfo> {
                let date = Utc.with_ymd_and_hms(
                    date[0].to_i32().ok_or_else(get_error)?, // year
                    cast_u32(date[1])?,                      // month
                    cast_u32(date[2])?,                      // day
                    cast_u32(date[3])?,                      // hour
                    cast_u32(date[4])?,                      // min
                    cast_u32(date[5])?,                      // sec
                );
                Ok(date.map(|dt| dt.with_nanosecond(nanos)))
            })
            .transpose()?;

        match date {
            Some(
                LocalResult::Single(Some(date))
                | LocalResult::Ambiguous(Some(date), _)
                | LocalResult::Ambiguous(_, Some(date)),
            ) => {
                object.value.insert(
                    "milliseconds".to_owned(),
                    PrimitiveInt::get_literal(date.timestamp_millis(), interval),
                );
                let mut lit = Self::get_literal(object.value.clone(), interval);
                lit.set_content_type("time");

                Ok(lit)
            }
            _ => Ok(PrimitiveBoolean::get_literal(false, interval)),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn with_timezone(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let [tz_name]: [&String; 1] = get_string_args(
            &args,
            data,
            interval,
            String::new(),
            "with_timezone(timezone_name: string) => Time Object. Example: with_timezone(\"Europe/Paris\") ",
        )?;
        let tz: Tz = match tz_name.parse() {
            Ok(tz) => tz,
            Err(_) => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    format!("invalid timezone {tz_name}"),
                ));
            }
        };

        let timezone = tz.to_string();

        object.value.insert(
            "timezone".to_owned(),
            PrimitiveString::get_literal(&timezone, interval),
        );

        let mut lit = Self::get_literal(object.value.clone(), interval);
        lit.set_content_type("time");

        Ok(lit)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn unix(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "unix(type_of_time) expect string argument \"m\" || \"s\" => int(time in seconds or milliseconds)";

        let time_type = match args.get("arg0") {
            Some(lit) if lit.primitive.get_type() == PrimitiveType::PrimitiveString => {
                let time_value = Literal::get_value::<String, _>(
                    &lit.primitive,
                    &data.context.flow,
                    interval,
                    String::new(),
                )?;

                match time_value {
                    t_val if t_val == "s" => t_val.clone(),
                    _ => "m".to_owned(),
                }
            }
            _ => "m".to_owned(),
        };

        match object.value.get("milliseconds") {
            Some(lit) if lit.primitive.get_type() == PrimitiveType::PrimitiveInt => {
                let millis = Literal::get_value::<i64, _>(
                    &lit.primitive,
                    &data.context.flow,
                    interval,
                    String::new(),
                )?;

                let LocalResult::Single(date) = Utc.timestamp_millis_opt(*millis) else {
                    return Err(gen_error_info(
                        Position::new(interval, &data.context.flow),
                        "Invalid milliseconds".to_string(),
                    ));
                };

                let duration = match time_type {
                    t_val if t_val == "s" => date.timestamp(),
                    _ => date.timestamp_millis(),
                };

                Ok(PrimitiveInt::get_literal(duration, interval))
            }
            _ => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                usage.to_string(),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn add_time(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let mut final_time = 0;

        if let Some(time_value) = object.value.get_mut("milliseconds") {
            let time = Literal::get_value::<i64, _>(
                &time_value.primitive,
                &data.context.flow,
                interval,
                String::new(),
            )?;

            final_time += *time;
        }

        let [add_time]: [i64; 1] = get_int_args(
            &args,
            data,
            interval,
            String::new(),
            "add(time_in_seconds: int) => Time Object",
        )?;
        final_time += add_time * 1000;

        object.value.insert(
            "milliseconds".to_owned(),
            PrimitiveInt::get_literal(final_time, interval),
        );
        let mut lit = Self::get_literal(object.value.clone(), interval);
        lit.set_content_type("time");

        Ok(lit)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn sub_time(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let mut final_time = 0;

        if let Some(time_value) = object.value.get_mut("milliseconds") {
            let time = Literal::get_value::<i64, _>(
                &time_value.primitive,
                &data.context.flow,
                interval,
                String::new(),
            )?;

            final_time += *time;
        }

        let [add_time]: [i64; 1] = get_int_args(
            &args,
            data,
            interval,
            String::new(),
            "sub(time_in_seconds: int) => Time Object",
        )?;
        final_time -= add_time * 1000;

        object.value.insert(
            "milliseconds".to_owned(),
            PrimitiveInt::get_literal(final_time, interval),
        );
        let mut lit = Self::get_literal(object.value.clone(), interval);
        lit.set_content_type("time");

        Ok(lit)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn parse_date(
        _object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        match args.len() {
            1 => tools_time::parse_rfc3339(&args, data, interval),
            len if len >= 2 => tools_time::pasre_from_str(&args, data, interval),
            _ => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "usage: expect one ore two arguments :
                Time().parse(\"2020-08-13\")   or
                Time().parse(\"1983-08-13 12:09:14.274\", \"%Y-%m-%d %H:%M:%S%.3f\")"
                    .to_string(),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn date_format(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "Time().format(format: String)";

        let offset = object.value.get("offset").and_then(|offset| {
            Literal::get_value::<i64, _>(
                &offset.primitive,
                &data.context.flow,
                interval,
                String::new(),
            )
            .ok()
        });

        let invalid_time = |what: &str| {
            gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("invalid {what}"),
            )
        };

        if let Some(lit) = object.value.get("milliseconds")
            && lit.primitive.get_type() == PrimitiveType::PrimitiveInt
        {
            let millis = Literal::get_value::<i64, _>(
                &lit.primitive,
                &data.context.flow,
                interval,
                String::new(),
            )?;
            let formatted_date = match (object.value.get("timezone"), offset) {
                (None, None) => {
                    let date: DateTime<Utc> = Utc
                        .timestamp_millis_opt(*millis)
                        .earliest()
                        .ok_or_else(|| invalid_time("milliseconds"))?;

                    tools_time::format_date(&args, &date, data, interval, true)?
                }
                (Some(timezone), _) => {
                    let tz = Literal::get_value::<String, _>(
                        &timezone.primitive,
                        &data.context.flow,
                        interval,
                        String::new(),
                    )
                    .and_then(|tz_string| {
                        tz_string.parse::<Tz>().map_err(|_| {
                            gen_error_info(
                                Position::new(interval, &data.context.flow),
                                format!("invalid timezone {tz_string}"),
                            )
                        })
                    })?;

                    let local_date = Utc
                        .timestamp_millis_opt(*millis)
                        .earliest()
                        .ok_or_else(|| invalid_time("milliseconds"))?;
                    let date = UTC
                        .from_local_datetime(&local_date.naive_local())
                        .earliest()
                        .ok_or_else(|| invalid_time("milliseconds"))?
                        .with_timezone(&tz);
                    tools_time::format_date(&args, &date, data, interval, false)?
                }
                (None, Some(offset)) => {
                    let date: DateTime<FixedOffset> =
                        FixedOffset::east_opt(i32::try_from(*offset)?)
                            .ok_or_else(|| invalid_time("offset"))?
                            .timestamp_millis_opt(*millis)
                            .earliest()
                            .ok_or_else(|| invalid_time("milliseconds"))?;

                    tools_time::format_date(&args, &date, data, interval, false)?
                }
            };
            return Ok(PrimitiveString::get_literal(&formatted_date, interval));
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("usage: {usage}"),
        ))
    }
}

impl PrimitiveObject {
    #[allow(clippy::needless_pass_by_value)]
    fn jwt_sign(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let mut headers = jsonwebtoken::Header::default();

        match args.get("arg0") {
            Some(algo) if algo.primitive.get_type() == PrimitiveType::PrimitiveString => {
                headers.alg = tools_jwt::get_algorithm(algo, &data.context.flow, interval)?;
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_SIGN_ALGO.to_string(),
                ));
            }
        }

        let claims = match object.value.get("jwt") {
            Some(literal) => literal.primitive.to_json(),
            None => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_SIGN_CLAIMS.to_string(),
                ));
            }
        };

        let key = match args.get("arg1") {
            Some(key) if key.primitive.get_type() == PrimitiveType::PrimitiveString => {
                let key = Literal::get_value::<String, _>(
                    &key.primitive,
                    &data.context.flow,
                    interval,
                    ERROR_JWT_SIGN_SECRET,
                )?;

                jsonwebtoken::EncodingKey::from_secret(key.as_ref())
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_ALGO.to_string(),
                ));
            }
        };

        if let Some(lit) = args.get("arg2") {
            tools_jwt::get_headers(lit, &data.context.flow, interval, &mut headers)?;
        }

        match jsonwebtoken::encode(&headers, &claims, &key) {
            Ok(value) => Ok(PrimitiveString::get_literal(&value, interval)),
            Err(e) => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("Invalid JWT encode {:?}", e.kind()),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn jwt_decode(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let token = match object.value.get("jwt") {
            Some(literal) => Literal::get_value::<String, _>(
                &literal.primitive,
                &data.context.flow,
                interval,
                ERROR_JWT_TOKEN,
            )?,
            None => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_TOKEN.to_string(),
                ));
            }
        };

        let algo = match args.get("arg0") {
            Some(algo) if algo.primitive.get_type() == PrimitiveType::PrimitiveString => {
                tools_jwt::get_algorithm(algo, &data.context.flow, interval)?
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_DECODE_ALGO.to_string(),
                ));
            }
        };

        let key = match args.get("arg1") {
            Some(key) if key.primitive.get_type() == PrimitiveType::PrimitiveString => {
                let key = Literal::get_value::<String, _>(
                    &key.primitive,
                    &data.context.flow,
                    interval,
                    ERROR_JWT_DECODE_SECRET,
                )?;

                jsonwebtoken::DecodingKey::from_secret(key.as_ref())
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_DECODE_SECRET.to_string(),
                ));
            }
        };

        match jsonwebtoken::decode::<serde_json::Value>(
            token,
            &key,
            &jsonwebtoken::Validation::new(algo),
        ) {
            Ok(token_message) => {
                tools_jwt::token_data_to_literal(&token_message, &data.context.flow, interval)
            }
            Err(e) => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("Invalid JWT decode {:?}", e.kind()),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn jwt_verity(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let mut validation = jsonwebtoken::Validation::default();

        let token = match object.value.get("jwt") {
            Some(literal) => Literal::get_value::<String, _>(
                &literal.primitive,
                &data.context.flow,
                interval,
                ERROR_JWT_TOKEN,
            )?,
            None => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_TOKEN.to_string(),
                ));
            }
        };

        match args.get("arg0") {
            Some(lit) => {
                tools_jwt::get_validation(lit, &data.context.flow, interval, &mut validation)?;
            }
            None => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_VALIDATION_CLAIMS.to_string(),
                ));
            }
        }

        match args.get("arg1") {
            Some(algo) if algo.primitive.get_type() == PrimitiveType::PrimitiveString => {
                validation.algorithms = vec![tools_jwt::get_algorithm(
                    algo,
                    &data.context.flow,
                    interval,
                )?];
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_VALIDATION_ALGO.to_string(),
                ));
            }
        }

        let key = match args.get("arg2") {
            Some(key) if key.primitive.get_type() == PrimitiveType::PrimitiveString => {
                let key = Literal::get_value::<String, _>(
                    &key.primitive,
                    &data.context.flow,
                    interval,
                    ERROR_JWT_SECRET,
                )?;

                jsonwebtoken::DecodingKey::from_secret(key.as_ref())
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_JWT_VALIDATION_SECRETE.to_string(),
                ));
            }
        };

        match jsonwebtoken::decode::<serde_json::Value>(token, &key, &validation) {
            Ok(token_message) => {
                tools_jwt::token_data_to_literal(&token_message, &data.context.flow, interval)
            }
            Err(e) => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("Invalid JWT verify {:?}", e.kind()),
            )),
        }
    }
}

impl PrimitiveObject {
    #[allow(clippy::needless_pass_by_value)]
    fn create_hmac(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let flow_name = &data.context.flow;

        let data = match object.value.get("value") {
            Some(literal) => Literal::get_value::<String, _>(
                &literal.primitive,
                flow_name,
                interval,
                ERROR_HASH,
            )?,
            None => {
                return Err(gen_error_info(
                    Position::new(interval, flow_name),
                    ERROR_HASH.to_string(),
                ));
            }
        };

        let algo = match args.get("arg0") {
            Some(algo) if algo.primitive.get_type() == PrimitiveType::PrimitiveString => {
                let algo = Literal::get_value::<String, _>(
                    &algo.primitive,
                    flow_name,
                    interval,
                    ERROR_HASH_ALGO,
                )?;
                tools_crypto::get_hash_algorithm(algo, flow_name, interval)?
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, flow_name),
                    ERROR_HASH_ALGO.to_string(),
                ));
            }
        };

        let key = match args.get("arg1") {
            Some(algo) if algo.primitive.get_type() == PrimitiveType::PrimitiveString => {
                let secret = Literal::get_value::<String, _>(
                    &algo.primitive,
                    flow_name,
                    interval,
                    ERROR_HMAC_KEY,
                )?;
                openssl::pkey::PKey::hmac(secret.as_bytes()).unwrap()
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, flow_name),
                    ERROR_HMAC_KEY.to_string(),
                ));
            }
        };

        let sign = openssl::sign::Signer::new(algo, &key);
        match sign {
            Ok(mut signer) => {
                signer.update(data.as_bytes()).unwrap();

                let vec = signer
                    .sign_to_vec()
                    .unwrap()
                    .iter()
                    .map(|val| PrimitiveInt::get_literal(i64::from(*val), interval))
                    .collect::<Vec<Literal>>();

                let mut map = HashMap::new();
                map.insert(
                    "hash".to_string(),
                    PrimitiveArray::get_literal(vec, interval),
                );

                let mut lit = Self::get_literal(map, interval);
                lit.set_content_type("crypto");
                Ok(lit)
            }
            Err(e) => Err(gen_error_info(
                Position::new(interval, flow_name),
                format!("{e}"),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn create_hash(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let flow_name = &data.context.flow;

        let data = match object.value.get("value") {
            Some(literal) => Literal::get_value::<String, _>(
                &literal.primitive,
                &data.context.flow,
                interval,
                ERROR_HASH,
            )?,
            None => {
                return Err(gen_error_info(
                    Position::new(interval, flow_name),
                    ERROR_HASH.to_string(),
                ));
            }
        };

        let algo = match args.get("arg0") {
            Some(algo) if algo.primitive.get_type() == PrimitiveType::PrimitiveString => {
                let algo = Literal::get_value::<String, _>(
                    &algo.primitive,
                    flow_name,
                    interval,
                    ERROR_HASH_ALGO,
                )?;
                tools_crypto::get_hash_algorithm(algo, flow_name, interval)?
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, flow_name),
                    ERROR_HASH_ALGO.to_string(),
                ));
            }
        };

        match openssl::hash::hash(algo, data.as_bytes()) {
            Ok(digest_bytes) => {
                let vec = digest_bytes
                    .iter()
                    .map(|val| PrimitiveInt::get_literal(i64::from(*val), interval))
                    .collect::<Vec<Literal>>();

                let map = HashMap::from([(
                    "hash".to_string(),
                    PrimitiveArray::get_literal(vec, interval),
                )]);

                Ok(Self::get_literal_with_type("crypto", map, interval))
            }
            Err(e) => Err(gen_error_info(
                Position::new(interval, flow_name),
                format!("{e}"),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn digest(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let Some(literal) = object.value.get("hash") else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_DIGEST.to_string(),
            ));
        };
        let vec = Literal::get_value::<Vec<Literal>, _>(
            &literal.primitive,
            &data.context.flow,
            interval,
            ERROR_DIGEST,
        )?;

        let algo = match args.get("arg0") {
            Some(algo) if algo.primitive.get_type() == PrimitiveType::PrimitiveString => {
                Literal::get_value::<String, _>(
                    &algo.primitive,
                    &data.context.flow,
                    interval,
                    ERROR_DIGEST_ALGO,
                )?
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_DIGEST_ALGO.to_string(),
                ));
            }
        };

        let digest_data = vec
            .iter()
            .map(|value| {
                // The hash is stored as a vec of u8 so this cast should never fail
                Literal::get_value::<i64, _>(
                    &value.primitive,
                    &data.context.flow,
                    interval,
                    "ERROR_hash_TOKEN",
                )?
                .to_u8()
                .ok_or_else(|| {
                    gen_error_info(
                        Position::new(interval, &data.context.flow),
                        "ERROR_hash_TOKEN".to_string(),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let value = tools_crypto::digest_data(algo, &digest_data, &data.context.flow, interval)?;

        Ok(PrimitiveString::get_literal(&value, interval))
    }
}

impl PrimitiveObject {
    fn base64_encode(
        object: &mut Self,
        _args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "Base64(\"...\").encode() => String";

        let string = match object.value.get("string") {
            Some(lit) => lit.primitive.to_string(),
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    format!("usage: {usage}"),
                ));
            }
        };

        let result = base64::engine::general_purpose::STANDARD.encode(string.as_bytes());

        Ok(PrimitiveString::get_literal(&result, interval))
    }

    fn base64_decode(
        object: &mut Self,
        _args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "Base64(\"...\").decode() => String";

        let string = match object.value.get("string") {
            Some(lit) => lit.primitive.to_string(),
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    format!("usage: {usage}"),
                ));
            }
        };

        let result = match base64::engine::general_purpose::STANDARD.decode(string.as_bytes()) {
            Ok(buf) => format!("{}", String::from_utf8_lossy(&buf)),
            Err(_) => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    format!("Base64 invalid value: {string}, can't be decode"),
                ));
            }
        };

        Ok(PrimitiveString::get_literal(&result, interval))
    }
}

impl PrimitiveObject {
    fn hex_encode(
        object: &mut Self,
        _args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "Hex(\"...\").encode() => String";

        let string = match object.value.get("string") {
            Some(lit) => lit.primitive.to_string(),
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    format!("usage: {usage}"),
                ));
            }
        };

        let result = hex::encode(string.as_bytes());

        Ok(PrimitiveString::get_literal(&result, interval))
    }

    fn hex_decode(
        object: &mut Self,
        _args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "Hex(\"...\").decode() => String";

        let Some(literal) = object.value.get("string") else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        };
        let string = literal.primitive.to_string();

        let result = match hex::decode(string.as_bytes()) {
            Ok(buf) => format!("{}", String::from_utf8_lossy(&buf)),
            Err(_) => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    format!("Hex invalid value: {string}, can't be decode"),
                ));
            }
        };

        Ok(PrimitiveString::get_literal(&result, interval))
    }
}

impl PrimitiveObject {
    #[allow(clippy::needless_pass_by_value)]
    fn get_type(
        _object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "get_type() => string";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        Ok(PrimitiveString::get_literal(content_type, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_content(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "get_content() => object";

        require_n_args(0, &args, interval, data, usage)?;

        Ok(Literal {
            content_type: content_type.to_owned(),
            primitive: Box::new(object.clone()),
            additional_info: None,
            secure_variable: false,
            interval,
        })
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_email(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "is_email() => boolean";

        let text = match object.value.get("text") {
            Some(lit) if lit.content_type == "string" => lit.primitive.to_string(),
            _ => return Ok(PrimitiveBoolean::get_literal(false, interval)),
        };

        require_n_args(0, &args, interval, data, usage)?;

        let email_regex = LazyLock::force(&EMAIL_REGEX);

        let lit = PrimitiveBoolean::get_literal(email_regex.is_match(&text), interval);

        Ok(lit)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn is_secure(
        _object: &mut Self,
        _args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let lit = PrimitiveBoolean::get_literal(data.event.secure, interval);

        Ok(lit)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn match_args(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "match(a) => a";

        let lit = match (object.value.get("text"), object.value.get("payload")) {
            (Some(lit), _) | (_, Some(lit)) if lit.content_type == "string" => lit,
            _ => return Ok(PrimitiveNull::get_literal(interval)),
        };

        if args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let is_match = args
            .iter()
            .find_map(|(_, arg)| match_obj(lit, arg).then(|| arg.clone()));

        Ok(is_match.unwrap_or_else(|| PrimitiveNull::get_literal(interval)))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn match_array(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "match_array([a,b,c]) => a";

        let lit = match (object.value.get("text"), object.value.get("payload")) {
            (Some(lit), _) | (_, Some(lit)) if lit.content_type == "string" => lit,
            _ => return Ok(PrimitiveNull::get_literal(interval)),
        };

        let [array]: [&Vec<Literal>; 1] = get_args(
            &args,
            data,
            interval,
            format!("expect Array value as argument usage: {usage}"),
            usage,
        )?;

        let is_match = array
            .iter()
            .find_map(|arg| match_obj(lit, arg).then(|| arg.clone()));

        Ok(is_match.unwrap_or_else(|| PrimitiveNull::get_literal(interval)))
    }
}

impl PrimitiveObject {
    #[allow(clippy::needless_pass_by_value)]
    fn is_number(
        _object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_number() => boolean")?;

        Ok(PrimitiveBoolean::get_literal(false, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_int(
        _object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_int() => boolean")?;

        Ok(PrimitiveBoolean::get_literal(false, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_float(
        _object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_float() => boolean")?;

        Ok(PrimitiveBoolean::get_literal(false, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn type_of(
        _object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "type_of() => string")?;

        Ok(PrimitiveString::get_literal("object", interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_info(
        _object: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        literal::get_info(&args, additional_info, interval, data)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn convert_to_xml(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_xml() => string")?;

        match serde_xml_rs::to_string(&object.to_json()) {
            Ok(string) => Ok(PrimitiveString::get_literal(&string, interval)),
            Err(_) => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "Object can not be converted to xml".to_string(),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn convert_to_yaml(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_yaml() => string")?;

        let Ok(string) = serde_yml::to_string(&object.to_json()) else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "Object can not be converted to yaml".to_string(),
            ));
        };
        Ok(PrimitiveString::get_literal(&string, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn convert_to_string(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_string() => string")?;

        Ok(PrimitiveString::get_literal(&object.to_string(), interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn contains(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let [key] = get_string_args(
            &args,
            data,
            interval,
            ERROR_OBJECT_CONTAINS,
            "contains(key: string) => boolean",
        )?;

        let result = object.value.contains_key(key);

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_empty(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_empty() => boolean")?;

        let result = object.value.is_empty();

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn length(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "length() => int")?;

        let result = object.value.len();

        Ok(PrimitiveInt::get_literal(
            result.to_i64().ok_or_else(|| {
                gen_error_info(
                    Position::new(interval, &data.context.flow),
                    OVERFLOWING_OPERATION.to_string(),
                )
            })?,
            interval,
        ))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn keys(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "keys() => array")?;

        let result = object
            .value
            .keys()
            .map(|key| PrimitiveString::get_literal(key, interval))
            .collect();

        Ok(PrimitiveArray::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn values(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "values() => array")?;

        let result = object.value.values().cloned().collect();

        Ok(PrimitiveArray::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_generics(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let [key] = get_string_args(
            &args,
            data,
            interval,
            ERROR_OBJECT_GET_GENERICS,
            "get(key: string) => primitive",
        )?;

        match object.value.get(key) {
            Some(res) => Ok(res.clone()),
            None => Ok(PrimitiveNull::get_literal(interval)),
        }
    }
}

impl PrimitiveObject {
    #[allow(clippy::needless_pass_by_value)]
    fn clear_values(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "clear_values() => null")?;

        object.value.iter_mut().for_each(|(_, value)| {
            *value = PrimitiveNull::get_literal(interval);
        });

        Ok(PrimitiveNull::get_literal(interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn insert(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "insert(key: string, value: primitive) => null";

        require_n_args(2, &args, interval, data, usage)?;

        let key = match args.get("arg0") {
            Some(res) if res.primitive.get_type() == PrimitiveType::PrimitiveString => {
                Literal::get_value::<String, _>(
                    &res.primitive,
                    &data.context.flow,
                    interval,
                    ERROR_OBJECT_INSERT,
                )?
            }
            _ => {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_OBJECT_INSERT.to_owned(),
                ));
            }
        };

        let Some(value) = args.get("arg1") else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        };

        object.value.insert(key.clone(), value.clone());

        Ok(PrimitiveNull::get_literal(interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn assign(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let [obj]: [&HashMap<String, Literal>; 1] = get_args(
            &args,
            data,
            interval,
            ERROR_OBJECT_ASSIGN,
            "assign(obj: Object) => null",
        )?;

        object
            .value
            .extend(obj.iter().map(|(k, v)| (k.clone(), v.clone())));

        Ok(PrimitiveNull::get_literal(interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn remove(
        object: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
        _content_type: &str,
    ) -> Result<Literal, ErrorInfo> {
        let [key] = get_string_args(
            &args,
            data,
            interval,
            ERROR_OBJECT_REMOVE,
            "remove(key: string) => primitive",
        )?;

        match object.value.remove(key) {
            Some(value) => Ok(value),
            None => Ok(PrimitiveNull::get_literal(interval)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTION
////////////////////////////////////////////////////////////////////////////////

fn insert_to_object(
    src: &HashMap<String, Literal>,
    dst: &mut PrimitiveObject,
    key_name: &str,
    flow_name: &str,
    literal: Literal,
) {
    dst.value
        .entry(key_name.to_owned())
        .and_modify(|tmp: &mut Literal| {
            if let Ok(tmp) = Literal::get_mut_value::<HashMap<String, Literal>>(
                &mut tmp.primitive,
                flow_name,
                literal.interval,
                ERROR_UNREACHABLE.to_owned(),
            ) {
                for (key, value) in src {
                    tmp.insert(key.clone(), value.clone());
                }
            }
        })
        .or_insert_with(|| literal);
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveObject {
    #[must_use]
    pub fn new(value: HashMap<String, Literal>) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn get_literal(object: HashMap<String, Literal>, interval: Interval) -> Literal {
        Self::get_literal_with_type("object", object, interval)
    }

    #[must_use]
    pub fn get_literal_with_type(
        content_type: &str,
        object: HashMap<String, Literal>,
        interval: Interval,
    ) -> Literal {
        let primitive = Box::new(Self::new(object));

        Literal {
            content_type: content_type.to_owned(),
            primitive,
            additional_info: None,
            secure_variable: false,
            interval,
        }
    }

    #[must_use]
    pub fn obj_literal_to_json(map: &HashMap<String, Literal>) -> serde_json::Value {
        serde_json::Value::Object(
            map.iter()
                .map(|(key, literal)| {
                    (
                        key.clone(),
                        literal.primitive.format_mem(&literal.content_type, false),
                    )
                })
                .collect(),
        )
    }
}

////////////////////////////////////////////////////////////////////////////////
// TRAIT FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[typetag::serde]
impl Primitive for PrimitiveObject {
    fn is_eq(&self, other: &dyn Primitive) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            return self.value == other.value;
        }

        false
    }

    fn is_cmp(&self, _other: &dyn Primitive) -> Option<Ordering> {
        None
    }

    illegal_math_ops!();

    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn into_value(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self.value)
    }

    fn get_type(&self) -> PrimitiveType {
        PrimitiveType::PrimitiveObject
    }

    fn as_box_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn to_json(&self) -> serde_json::Value {
        let mut object: serde_json::map::Map<String, serde_json::Value> =
            serde_json::map::Map::new();

        for (key, literal) in &self.value {
            if TYPES.contains(&&(*literal.content_type)) {
                object.insert(key.clone(), literal.primitive.to_json());
            } else {
                let mut map = serde_json::Map::new();
                map.insert(
                    "content_type".to_owned(),
                    serde_json::json!(literal.content_type),
                );
                map.insert("content".to_owned(), literal.primitive.to_json());

                object.insert(key.clone(), serde_json::json!(map));
            }
        }

        serde_json::Value::Object(object)
    }

    fn format_mem(&self, content_type: &str, first: bool) -> serde_json::Value {
        let content = Self::obj_literal_to_json(&self.value);
        match (content_type, first) {
            ("object", false) => content,
            (content_type, _) => {
                json!({
                    "_content_type": content_type,
                    "_content": content
                })
            }
        }
    }

    fn to_string(&self) -> String {
        self.to_json().to_string()
    }

    fn as_bool(&self) -> bool {
        true
    }

    fn get_value(&self) -> &dyn std::any::Any {
        &self.value
    }

    fn get_mut_value(&mut self) -> &mut dyn std::any::Any {
        &mut self.value
    }

    fn to_msg(&self, content_type: String) -> Message {
        Message {
            content_type,
            content: self.to_json(),
        }
    }

    fn do_exec(
        &mut self,
        name: &str,
        args: HashMap<String, Literal>,
        mem_type: &MemoryType,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        content_type: &ContentType,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<(Literal, Right), ErrorInfo> {
        let event = vec![FUNCTIONS_EVENT];
        let http = vec![FUNCTIONS_HTTP, FUNCTIONS_READ, FUNCTIONS_WRITE];
        let smtp = vec![FUNCTIONS_SMTP];
        let base64 = vec![FUNCTIONS_BASE64];
        let hex = vec![FUNCTIONS_HEX];
        let jwt = vec![FUNCTIONS_JWT];
        let crypto = vec![FUNCTIONS_CRYPTO];
        let time = vec![FUNCTIONS_TIME];
        let generics = vec![FUNCTIONS_READ, FUNCTIONS_WRITE];

        let mut is_event = false;

        let (content_type, vector) = match content_type {
            ContentType::Event(event_type) => {
                is_event = true;

                (event_type.as_ref(), event)
            }
            ContentType::Http => ("", http),
            ContentType::Smtp => ("", smtp),
            ContentType::Base64 => ("", base64),
            ContentType::Hex => ("", hex),
            ContentType::Jwt => ("", jwt),
            ContentType::Crypto => ("", crypto),
            ContentType::Time => ("", time),
            ContentType::Primitive => ("", generics),
        };

        for function in &vector {
            if let Some((f, right)) = function.get(name) {
                if *mem_type == MemoryType::Constant && *right == Right::Write {
                    return Err(gen_error_info(
                        Position::new(interval, &data.context.flow),
                        ERROR_CONSTANT_MUTABLE_FUNCTION.to_string(),
                    ));
                }
                let result = f(self, args, additional_info, data, interval, content_type)?;

                return Ok((result, *right));
            }
        }

        if is_event {
            let vec = ["text", "payload"];
            for value in &vec {
                if let Some(res) = self.value.get_mut(*value) {
                    return res.primitive.do_exec(
                        name,
                        args,
                        mem_type,
                        additional_info,
                        interval,
                        &ContentType::Primitive,
                        data,
                        msg_data,
                        sender,
                    );
                }
            }
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{name}] {ERROR_OBJECT_UNKNOWN_METHOD}"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_mem() {
        let object = PrimitiveObject::new(HashMap::from([(
            "a".to_owned(),
            PrimitiveInt::get_literal(1, Interval::default()),
        )]));

        let result = object.format_mem("object", false);
        let expected = json! {
            {
                "a": 1
            }
        };

        assert_eq!(result, expected);

        let result = object.format_mem("not-object", true);
        let expected = json! {
            {
                "_content_type": "not-object",
                "_content": {
                    "a": 1
                }
            }
        };
        assert_eq!(result, expected);
    }
}
