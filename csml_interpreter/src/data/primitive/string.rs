use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::array::PrimitiveArray;
use crate::data::primitive::boolean::PrimitiveBoolean;
use crate::data::primitive::float::PrimitiveFloat;
use crate::data::primitive::int::PrimitiveInt;
use crate::data::primitive::null::PrimitiveNull;
use crate::data::primitive::object::PrimitiveObject;
use crate::data::primitive::tools::{Integer, check_division_preconditions, get_integer};
use crate::data::primitive::{Primitive, PrimitiveType};
use crate::data::primitive::{Right, common};
use crate::data::{Data, Literal, MSG, MemoryType, MessageData, ast::Interval, message::Message};
use crate::data::{literal, literal::ContentType};
use crate::error_format::{
    ERROR_CONSTANT_MUTABLE_FUNCTION, ERROR_ILLEGAL_OPERATION, ERROR_STRING_APPEND,
    ERROR_STRING_CONTAINS_REGEX, ERROR_STRING_DO_MATCH, ERROR_STRING_END_WITH,
    ERROR_STRING_END_WITH_REGEX, ERROR_STRING_FROM_JSON, ERROR_STRING_MATCH_REGEX,
    ERROR_STRING_NUMERIC, ERROR_STRING_REPLACE, ERROR_STRING_REPLACE_ALL,
    ERROR_STRING_REPLACE_REGEX, ERROR_STRING_RHS, ERROR_STRING_SPLIT, ERROR_STRING_START_WITH,
    ERROR_STRING_START_WITH_REGEX, ERROR_STRING_UNKNOWN_METHOD, ERROR_STRING_VALID_REGEX,
    OVERFLOWING_OPERATION, gen_error_info,
};
use crate::interpreter::json_to_literal;
// use http::Uri;
use crate::data::primitive::common::get_index_args;
use crate::data::primitive::utils::require_n_args;
use num_traits::ToPrimitive;
use num_traits::ops::checked;
use phf::phf_map;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::{collections::HashMap, ops, sync::mpsc};
use url::Url;
use url::form_urlencoded;
use url::form_urlencoded::Parse;
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

type PrimitiveMethod = fn(
    string: &mut PrimitiveString,
    args: HashMap<String, Literal>,
    additional_info: Option<&HashMap<String, Literal>>,
    interval: Interval,
    data: &mut Data,
    msg_data: &mut MessageData,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<Literal, ErrorInfo>;

const FUNCTIONS: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "is_number" => (PrimitiveString::is_number as PrimitiveMethod, Right::Read),
    "is_int" => (PrimitiveString::is_int as PrimitiveMethod, Right::Read),
    "is_float" => (PrimitiveString::is_float as PrimitiveMethod, Right::Read),
    "type_of" => (PrimitiveString::type_of as PrimitiveMethod, Right::Read),
    "get_info" => (PrimitiveString::get_info as PrimitiveMethod, Right::Read),
    "is_error" => ((|_, _, additional_info, interval, _, _, _| common::is_error(additional_info, interval)) as PrimitiveMethod, Right::Read),
    "to_string" => (PrimitiveString::convert_to_string as PrimitiveMethod, Right::Read),
    "to_json" => (PrimitiveString::convert_to_csml_json as PrimitiveMethod, Right::Read),

    "encode_uri" => (PrimitiveString::encode_uri as PrimitiveMethod, Right::Read),
    "decode_uri" => (PrimitiveString::decode_uri as PrimitiveMethod, Right::Read),
    "encode_uri_component" => (PrimitiveString::encode_uri_component as PrimitiveMethod, Right::Read),
    "decode_uri_component" => (PrimitiveString::decode_uri_component as PrimitiveMethod, Right::Read),
    "encode_html_entities" => (PrimitiveString::encode_html_entities as PrimitiveMethod, Right::Read),
    "decode_html_entities" => (PrimitiveString::decode_html_entities as PrimitiveMethod, Right::Read),

    "is_email" => (PrimitiveString::is_email as PrimitiveMethod, Right::Read),
    "append" => (PrimitiveString::append as PrimitiveMethod, Right::Read),
    "contains" => (PrimitiveString::contains as PrimitiveMethod, Right::Read),
    "contains_regex" => (PrimitiveString::contains_regex as PrimitiveMethod, Right::Read),
    "replace_regex" => (PrimitiveString::replace_regex as PrimitiveMethod, Right::Read),
    "replace_all" => (PrimitiveString::replace_all as PrimitiveMethod, Right::Read),
    "replace" => (PrimitiveString::replace as PrimitiveMethod, Right::Read),

    "ends_with" => (PrimitiveString::ends_with as PrimitiveMethod, Right::Read),
    "ends_with_regex" => (PrimitiveString::ends_with_regex as PrimitiveMethod, Right::Read),
    "from_json" => (PrimitiveString::from_json as PrimitiveMethod, Right::Read),
    "is_empty" => (PrimitiveString::is_empty as PrimitiveMethod, Right::Read),
    "length" => (PrimitiveString::length as PrimitiveMethod, Right::Read),
    "match" => (PrimitiveString::do_match as PrimitiveMethod, Right::Read),
    "match_regex" => (PrimitiveString::do_match_regex as PrimitiveMethod, Right::Read),
    "starts_with" => (PrimitiveString::starts_with as PrimitiveMethod, Right::Read),
    "starts_with_regex" => (PrimitiveString::starts_with_regex as PrimitiveMethod, Right::Read),
    "to_lowercase" => (PrimitiveString::convert_to_lowercase as PrimitiveMethod, Right::Read),
    "to_uppercase" => (PrimitiveString::convert_to_uppercase as PrimitiveMethod, Right::Read),
    "capitalize" => (PrimitiveString::capitalize as PrimitiveMethod, Right::Read),
    "slice" => (PrimitiveString::slice as PrimitiveMethod, Right::Read),
    "split" => (PrimitiveString::split as PrimitiveMethod, Right::Read),

    "trim" => (PrimitiveString::trim as PrimitiveMethod, Right::Read),
    "trim_left" => (PrimitiveString::trim_left as PrimitiveMethod, Right::Read),
    "trim_right" => (PrimitiveString::trim_right as PrimitiveMethod, Right::Read),

    "abs" => (PrimitiveString::abs as PrimitiveMethod, Right::Read),
    "cos" => (PrimitiveString::cos as PrimitiveMethod, Right::Read),
    "ceil" =>(PrimitiveString::ceil as PrimitiveMethod, Right::Read),
    "floor" => (PrimitiveString::floor as PrimitiveMethod, Right::Read),
    "pow" => (PrimitiveString::pow as PrimitiveMethod, Right::Read),
    "round" => (PrimitiveString::round as PrimitiveMethod, Right::Read),
    "sin" => (PrimitiveString::sin as PrimitiveMethod, Right::Read),
    "sqrt" => (PrimitiveString::sqrt as PrimitiveMethod, Right::Read),
    "tan" => (PrimitiveString::tan as PrimitiveMethod, Right::Read),
    "to_int" => (PrimitiveString::convert_to_int as PrimitiveMethod, Right::Read),
    "to_float" =>(PrimitiveString::convert_to_float as PrimitiveMethod, Right::Read),
};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveString {
    pub value: String,
}

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn encode_key_value<'a, F>(acc: &mut String, encode: &F, key: &'a str, value: &'a str)
where
    for<'f> F: Fn(&'f str) -> Cow<'f, str>,
{
    acc.push_str(&encode(key));

    if value.is_empty() {
        return;
    }
    let mut split = value.split(',');

    if let Some(first) = split.next() {
        acc.push('=');
        acc.push_str(&encode(first));
        split.for_each(|value| {
            acc.push(',');
            acc.push_str(&encode(value));
        });
    }
}

fn encode_decode<'p, 'a: 'p, F>(pairs: &'p Parse<'a>, encode: F) -> String
where
    for<'f> F: Fn(&'f str) -> Cow<'f, str>,
{
    let mut output = String::new();
    let mut iter = pairs.into_iter();
    if let Some((key, value)) = iter.next() {
        encode_key_value(&mut output, &encode, &key, &value);
        iter.for_each(|(key, value)| {
            output.push('&');
            encode_key_value(&mut output, &encode, &key, &value);
        });
    }

    output
}

fn encode_value(pairs: &Parse) -> String {
    encode_decode(pairs, urlencoding::encode)
}

fn decode(value: &str) -> Cow<'_, str> {
    match urlencoding::decode(value) {
        Ok(decoded) => decoded,
        Err(_) => Cow::Borrowed(value),
    }
}

fn decode_value(pairs: &Parse) -> String {
    encode_decode(pairs, decode)
}

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveString {
    #[allow(clippy::needless_pass_by_value)]
    fn is_number(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_number() => boolean")?;

        let result = string.value.parse::<f64>().is_ok();

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_int(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_int() => boolean")?;

        let result = string.value.parse::<i64>().is_ok();

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_float(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_float() => boolean")?;

        let result = string
            .value
            .parse::<f64>()
            .is_ok_and(|_| string.value.find('.').is_some());

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_email(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_email() => boolean")?;

        let email_regex = Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap();

        let result = email_regex.is_match(&string.value);

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn type_of(
        _string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "type_of() => string")?;

        Ok(Self::get_literal("string", interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_info(
        _string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        literal::get_info(&args, additional_info, interval, data)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn convert_to_string(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_string() => string")?;

        Ok(Self::get_literal(&string.value, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn convert_to_csml_json(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_json() => object")?;

        let config = roxmltree_to_serde::Config::new_with_custom_values(
            true,
            "@",
            "$text",
            roxmltree_to_serde::NullValue::Ignore,
        );

        let xml: Option<serde_json::Value> =
            roxmltree_to_serde::xml_str_to_json(&string.value, &config).ok();

        let yaml: Option<serde_json::Value> = serde_yml::from_str(&string.value).ok();

        match (&yaml, &xml) {
            (_, Some(json)) | (Some(json), _) => {
                json_to_literal(json, interval, &data.context.flow)
            }
            _ => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "Invalid format string is not a valid yaml or xml".to_string(),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn encode_uri(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "encode_uri() => String")?;

        let vec: Vec<&str> = string.value.split('?').collect();

        let (q_separator, query, separator, fragment) = if vec.len() > 1 {
            let url = Url::parse(&string.value).unwrap();

            let query_pairs = url.query_pairs();
            let (q_separator, query) = if query_pairs.count() == 0 {
                ("", String::new())
            } else {
                let query = encode_value(&query_pairs);
                ("?", query)
            };

            let (f_serparatorm, fragment) = match url.fragment() {
                Some(frag) => {
                    let pairs = form_urlencoded::parse(frag.as_bytes());
                    let fragment = encode_value(&pairs);

                    ("#", fragment)
                }
                None => ("", String::new()),
            };

            (q_separator, query, f_serparatorm, fragment)
        } else {
            ("", String::new(), "", String::new())
        };

        Ok(Self::get_literal(
            &format!("{}{q_separator}{query}{separator}{fragment}", vec[0]),
            interval,
        ))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn decode_uri(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "decode_uri() => String")?;

        let mut split = string.value.split('?');
        // Split always has at least one element so unwrap is safe
        let path = split.next().unwrap();

        let (q_separator, query, separator, fragment) = if split.next().is_some() {
            let url = Url::parse(&string.value).unwrap();

            let query_pairs = url.query_pairs();
            let (q_separator, query) = if query_pairs.count() == 0 {
                ("", String::new())
            } else {
                let query = decode_value(&query_pairs);
                ("?", query)
            };

            let (f_serparatorm, fragment) = match url.fragment() {
                Some(frag) => {
                    let pairs = form_urlencoded::parse(frag.as_bytes());
                    let fragment = decode_value(&pairs);

                    ("#", fragment)
                }
                None => ("", String::new()),
            };

            (q_separator, query, f_serparatorm, fragment)
        } else {
            ("", String::new(), "", String::new())
        };

        Ok(Self::get_literal(
            &format!("{path}{q_separator}{query}{separator}{fragment}"),
            interval,
        ))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn encode_uri_component(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "encode_uri_component() => String")?;

        let encoded = urlencoding::encode(&string.value);

        Ok(Self::get_literal(&encoded, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn decode_uri_component(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "decode_uri_component() => String")?;

        match urlencoding::decode(&string.value) {
            Ok(decoded) => Ok(Self::get_literal(&decoded, interval)),
            Err(_) => Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "Invalid UTF8 string".to_string(),
            )),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn decode_html_entities(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "decode_html_entities() => String")?;

        let decoded = html_escape::decode_html_entities(&string.value);

        Ok(Self::get_literal(&decoded, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn encode_html_entities(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "encode_html_entities() => String")?;

        let decoded = html_escape::encode_text(&string.value);

        Ok(Self::get_literal(&decoded, interval))
    }
}

impl PrimitiveString {
    #[allow(clippy::needless_pass_by_value)]
    fn append(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [value] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_APPEND,
            "append(value: string) => string",
        )?;

        let result = format!("{}{}", string.value, value);

        Ok(Self::get_literal(&result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn contains(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [value] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_DO_MATCH,
            "contains(value: string) => boolean",
        )?;

        let result = string.value.contains(value);

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn contains_regex(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [value] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_CONTAINS_REGEX,
            "contains_regex(value: string) => boolean",
        )?;

        let Ok(action) = Regex::new(value) else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_STRING_CONTAINS_REGEX.to_owned(),
            ));
        };

        let result = action.is_match(&string.value);

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn replace(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [to_replace, replace_by] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_REPLACE,
            "replace(value_to_replace: string, replace_by: string) => string",
        )?;

        let new_string = string.value.replacen(to_replace, replace_by, 1);

        Ok(Self::get_literal(&new_string, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn replace_all(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [to_replace, replace_by] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_REPLACE_ALL,
            "replace_all(value_to_replace: string, replace_by: string) => string",
        )?;

        let new_string = string.value.replace(to_replace, replace_by);

        Ok(Self::get_literal(&new_string, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn replace_regex(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [regex, replace_by] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_REPLACE_ALL,
            "replace_regex(regex: string, replace_by: string) => string",
        )?;

        let Ok(reg) = Regex::new(regex) else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_STRING_REPLACE_REGEX.to_owned(),
            ));
        };

        let new_string = reg.replace_all(&string.value, replace_by);

        Ok(Self::get_literal(&new_string, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn ends_with(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [value] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_END_WITH,
            "ends_with(value: string) => boolean",
        )?;

        let result = string.value.ends_with(value);

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn ends_with_regex(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [value] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_END_WITH_REGEX,
            "ends_with_regex(value: string) => boolean",
        )?;

        let Ok(action) = Regex::new(value) else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_STRING_END_WITH_REGEX.to_owned(),
            ));
        };

        for key in action.find_iter(&string.value) {
            if key.end() == string.value.len() {
                return Ok(PrimitiveBoolean::get_literal(true, interval));
            }
        }

        Ok(PrimitiveBoolean::get_literal(false, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn from_json(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "from_json() => object")?;

        let Ok(object) = serde_json::from_str(&string.value) else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_STRING_FROM_JSON.to_owned(),
            ));
        };

        json_to_literal(&object, interval, &data.context.flow)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn is_empty(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "is_empty() => boolean")?;

        let result = string.value.is_empty();

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn length(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "length() => int")?;

        let result = string.value.len();

        Ok(PrimitiveInt::get_literal(
            result.to_i64().ok_or_else(|| {
                gen_error_info(
                    Position::new(interval, &data.context.flow),
                    OVERFLOWING_OPERATION.to_owned(),
                )
            })?,
            interval,
        ))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn do_match(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [value] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_DO_MATCH,
            "match(value: string) => array",
        )?;

        let vector: Vec<Literal> = string
            .value
            .matches(value)
            .map(|result| Self::get_literal(result, interval))
            .collect();

        if vector.is_empty() {
            return Ok(PrimitiveNull::get_literal(interval));
        }

        Ok(PrimitiveArray::get_literal(vector, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn do_match_regex(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [value] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_MATCH_REGEX,
            "match_regex(value: string) => array",
        )?;

        let mut s: &str = &string.value;
        let mut vector: Vec<Literal> = Vec::new();

        let Ok(action) = Regex::new(value) else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_STRING_VALID_REGEX.to_owned(),
            ));
        };

        while let Some(result) = action.find(s) {
            vector.push(Self::get_literal(
                &s[result.start()..result.end()],
                interval,
            ));
            s = &s[result.end()..];
        }

        if vector.is_empty() {
            return Ok(PrimitiveNull::get_literal(interval));
        }

        Ok(PrimitiveArray::get_literal(vector, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn starts_with(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [value] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_START_WITH,
            "starts_with(value: string) => boolean",
        )?;

        let result = string.value.starts_with(value);

        Ok(PrimitiveBoolean::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn starts_with_regex(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [value] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_START_WITH_REGEX,
            "starts_with_regex(value: string) => boolean",
        )?;

        let Ok(action) = Regex::new(value) else {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                ERROR_STRING_VALID_REGEX.to_owned(),
            ));
        };

        if let Some(res) = action.find(&string.value)
            && res.start() == 0
        {
            return Ok(PrimitiveBoolean::get_literal(true, interval));
        }

        Ok(PrimitiveBoolean::get_literal(false, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn convert_to_lowercase(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_lowercase() => string")?;

        let s = &string.value;
        Ok(Self::get_literal(&s.to_lowercase(), interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn convert_to_uppercase(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_uppercase() => string")?;

        let s = &string.value;
        Ok(Self::get_literal(&s.to_uppercase(), interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn capitalize(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "capitalize() => string")?;

        let s = &string.value;

        let mut c = s.chars();
        let string = match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        };

        Ok(Self::get_literal(&string, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn slice(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let text_vec = string.value.chars().collect::<Vec<_>>();

        let (start_index, end_index) = get_index_args(
            text_vec.len(),
            &args,
            interval,
            data,
            "usage: slice(start: Integer, end: Optional<Integer>) => string",
        )?;
        let value: String = text_vec[start_index..end_index].iter().collect();

        Ok(Self::get_literal(&value, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn split(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        let [separator] = common::get_string_args(
            &args,
            data,
            interval,
            ERROR_STRING_SPLIT,
            "split(separator: string) => array",
        )?;

        let vector: Vec<Literal> = string
            .value
            .split(separator)
            .map(|part| Self::get_literal(part, interval))
            .collect();

        Ok(PrimitiveArray::get_literal(vector, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn trim(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "trim() => string")?;

        let s = &string.value;
        Ok(Self::get_literal(s.trim(), interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn trim_left(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "trim_left() => string")?;

        let s = &string.value;
        Ok(Self::get_literal(s.trim_start(), interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn trim_right(
        string: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        _msg_data: &mut MessageData,
        _sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "trim_right() => string")?;

        let s = &string.value;
        Ok(Self::get_literal(s.trim_end(), interval))
    }
}

// memory type can be set tu 'use' because the result of the operation will create a new literal.
impl PrimitiveString {
    fn abs(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "abs",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "abs",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "abs", ERROR_STRING_NUMERIC),
        ))
    }

    fn cos(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "cos",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "cos",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "cos", ERROR_STRING_NUMERIC),
        ))
    }

    fn ceil(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "ceil",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "ceil",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "ceil", ERROR_STRING_NUMERIC),
        ))
    }

    fn pow(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "pow",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "pow",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "pow", ERROR_STRING_NUMERIC),
        ))
    }

    fn floor(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "floor",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "floor",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "floor", ERROR_STRING_NUMERIC),
        ))
    }

    fn round(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "round",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "round",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "round", ERROR_STRING_NUMERIC),
        ))
    }

    fn sin(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "sin",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "sin",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "sin", ERROR_STRING_NUMERIC),
        ))
    }

    fn sqrt(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "sqrt",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "sqrt",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "sqrt", ERROR_STRING_NUMERIC),
        ))
    }

    fn tan(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "tan",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "tan",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "tan", ERROR_STRING_NUMERIC),
        ))
    }

    fn convert_to_int(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "to_int",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "to_int",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "to_int", ERROR_STRING_NUMERIC),
        ))
    }

    fn convert_to_float(
        string: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        if let Ok(int) = string.value.parse::<i64>() {
            let mut primitive = PrimitiveInt::new(int);

            let (literal, _right) = primitive.do_exec(
                "to_float",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }
        if let Ok(float) = string.value.parse::<f64>() {
            let mut primitive = PrimitiveFloat::new(float);

            let (literal, _right) = primitive.do_exec(
                "to_float",
                args,
                &MemoryType::Use,
                additional_info,
                interval,
                &ContentType::Primitive,
                data,
                msg_data,
                sender,
            )?;

            return Ok(literal);
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{}] {}", "to_float", ERROR_STRING_NUMERIC),
        ))
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveString {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn to_literal(self, interval: Interval) -> Literal {
        Literal {
            content_type: "string".to_owned(),
            primitive: Box::new(self),
            additional_info: None,
            secure_variable: false,
            interval,
        }
    }

    #[must_use]
    pub fn get_literal(string: &str, interval: Interval) -> Literal {
        let primitive = Box::new(Self::new(string.to_owned()));

        Literal {
            content_type: "string".to_owned(),
            primitive,
            additional_info: None,
            secure_variable: false,
            interval,
        }
    }

    #[must_use]
    pub fn get_array_char(string: &str, interval: Interval) -> Vec<Literal> {
        string
            .chars()
            .map(|c| {
                let mut buffer = [0; 4];
                Self::get_literal(c.encode_utf8(&mut buffer), interval)
            })
            .collect::<Vec<Literal>>()
    }
}

impl PrimitiveString {
    fn do_op<FI: FnOnce(&i64, &i64) -> Option<i64>, FF: FnOnce(f64, f64) -> f64>(
        &self,
        right: &dyn Primitive,
        fi: FI,
        ff: FF,
        op: &str,
    ) -> Result<Box<dyn Primitive>, String> {
        let Some(rhs) = right.as_any().downcast_ref::<Self>() else {
            return Err(ERROR_STRING_RHS.to_owned());
        };

        let lhs = get_integer(&self.value);
        let rhs = get_integer(&rhs.value);
        let args = lhs.zip(rhs);

        if let Some((left, right)) = args {
            if let (Integer::Int(left), Integer::Int(right)) = (left, right)
                && let Some(result) = fi(&left, &right)
            {
                return Ok(Box::new(PrimitiveInt::new(result)));
            }
            return Ok(Box::new(PrimitiveFloat::new(ff(
                left.to_f64(),
                right.to_f64(),
            ))));
        }

        Err(format!(
            "{} {:?} {op} {:?}",
            ERROR_ILLEGAL_OPERATION,
            self.get_type(),
            right.get_type()
        ))
    }
}

#[typetag::serde]
impl Primitive for PrimitiveString {
    fn is_eq(&self, other: &dyn Primitive) -> bool {
        if let Some(rhs) = other.as_any().downcast_ref::<Self>() {
            return match (get_integer(&self.value), get_integer(&rhs.value)) {
                #[allow(clippy::cast_precision_loss)]
                (Some(Integer::Int(lhs)), Some(Integer::Float(rhs))) => (lhs as f64) == rhs,
                #[allow(clippy::cast_precision_loss)]
                (Some(Integer::Float(lhs)), Some(Integer::Int(rhs))) => lhs == (rhs as f64),
                _ => self.value == rhs.value,
            };
        }

        false
    }

    fn is_cmp(&self, other: &dyn Primitive) -> Option<Ordering> {
        if let Some(rhs) = other.as_any().downcast_ref::<Self>() {
            return match (get_integer(&self.value), get_integer(&rhs.value)) {
                #[allow(clippy::cast_precision_loss)]
                (Some(Integer::Int(lhs)), Some(Integer::Float(rhs))) => {
                    (lhs as f64).partial_cmp(&rhs)
                }
                #[allow(clippy::cast_precision_loss)]
                (Some(Integer::Float(lhs)), Some(Integer::Int(rhs))) => {
                    lhs.partial_cmp(&(rhs as f64))
                }
                _ => self.value.partial_cmp(&rhs.value),
            };
        }

        None
    }

    fn do_add(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        match self.do_op(other, checked::CheckedAdd::checked_add, ops::Add::add, "+") {
            Ok(res) => Ok(res),
            Err(_) => {
                if let Some(rhs) = other.as_any().downcast_ref::<Self>() {
                    return Ok(Box::new(PrimitiveString::new(format!(
                        "{}{}",
                        self.value, rhs.value
                    ))));
                }

                Err(ERROR_STRING_RHS.to_owned())
            }
        }
    }

    fn do_sub(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        self.do_op(other, checked::CheckedSub::checked_sub, ops::Sub::sub, "-")
    }

    fn do_div(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        let Some(rhs) = other.as_any().downcast_ref::<Self>() else {
            return Err(ERROR_STRING_RHS.to_owned());
        };

        let args = get_integer(&self.value)
            .map(Integer::to_i64)
            .zip(get_integer(&rhs.value).map(Integer::to_i64));
        if let Some((left, right)) = args {
            check_division_preconditions(left, right)?;
            #[allow(clippy::cast_precision_loss)]
            return Ok(Box::new(PrimitiveFloat::new((left / right) as f64)));
        }

        Err(format!(
            "{} {:?} / {:?}",
            ERROR_ILLEGAL_OPERATION,
            self.get_type(),
            other.get_type()
        ))
    }

    fn do_mul(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        self.do_op(other, checked::CheckedMul::checked_mul, ops::Mul::mul, "*")
    }

    fn do_rem(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        self.do_op(other, checked::CheckedRem::checked_rem, ops::Rem::rem, "%")
    }

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
        PrimitiveType::PrimitiveString
    }

    fn as_box_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!(self.value)
    }

    fn format_mem(&self, _content_type: &str, _first: bool) -> serde_json::Value {
        serde_json::json!(self.value)
    }

    fn to_string(&self) -> String {
        self.value.clone()
    }

    fn as_bool(&self) -> bool {
        true
    }

    fn get_value(&self) -> &dyn Any {
        &self.value
    }

    fn get_mut_value(&mut self) -> &mut dyn Any {
        &mut self.value
    }

    fn to_msg(&self, _content_type: String) -> Message {
        let mut hashmap: HashMap<String, Literal> = HashMap::new();

        hashmap.insert(
            "text".to_owned(),
            Literal {
                content_type: "string".to_owned(),
                primitive: Box::new(Self::new(self.value.clone())),
                additional_info: None,
                secure_variable: false,
                interval: Interval {
                    start_column: 0,
                    start_line: 0,
                    offset: 0,
                    end_line: None,
                    end_column: None,
                },
            },
        );

        let mut result = PrimitiveObject::get_literal(
            hashmap,
            Interval {
                start_column: 0,
                start_line: 0,
                offset: 0,
                end_line: None,
                end_column: None,
            },
        );
        result.set_content_type("text");

        Message {
            content_type: result.content_type,
            content: result.primitive.to_json(),
        }
    }

    fn do_exec(
        &mut self,
        name: &str,
        args: HashMap<String, Literal>,
        mem_type: &MemoryType,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        _content_type: &ContentType,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<(Literal, Right), ErrorInfo> {
        if let Some((f, right)) = FUNCTIONS.get(name) {
            if *mem_type == MemoryType::Constant && *right == Right::Write {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    ERROR_CONSTANT_MUTABLE_FUNCTION.to_string(),
                ));
            }
            let res = f(
                self,
                args,
                additional_info,
                interval,
                data,
                msg_data,
                sender,
            )?;

            return Ok((res, *right));
        }

        Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("[{name}] {ERROR_STRING_UNKNOWN_METHOD}"),
        ))
    }
}
