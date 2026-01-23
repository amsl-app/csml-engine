use super::utils::{illegal_math_ops, impl_do_exec, impl_type_check};
use crate::data::error_info::ErrorInfo;
use crate::data::literal;
use crate::data::position::Position;
use crate::data::primitive::{
    Primitive, PrimitiveType, Right, boolean::PrimitiveBoolean, common, object::PrimitiveObject,
    string::PrimitiveString,
};
use crate::data::{
    Data, Literal, MSG, MemoryType, MessageData, ast::Interval, literal::ContentType,
    message::Message, tokens::NULL,
};
use crate::error_format::{ERROR_NULL_UNKNOWN_METHOD, gen_error_info};
use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cmp::Ordering;
use std::{collections::HashMap, sync::mpsc};
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

type PrimitiveMethod = fn(
    null: &mut PrimitiveNull,
    args: HashMap<String, Literal>,
    additional_info: Option<&HashMap<String, Literal>>,
    data: &mut Data,
    interval: Interval,
) -> Result<Literal, ErrorInfo>;

const FUNCTIONS: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "is_number" => (PrimitiveNull::is_number as PrimitiveMethod, Right::Read),
    "is_int" => (PrimitiveNull::is_int as PrimitiveMethod, Right::Read),
    "is_float" => (PrimitiveNull::is_float as PrimitiveMethod, Right::Read),
    "type_of" => (PrimitiveNull::type_of as PrimitiveMethod, Right::Read),
    "get_info" => (PrimitiveNull::get_info as PrimitiveMethod, Right::Read),
    "is_error" => ((|_, _, additional_info, _, interval| common::is_error(additional_info, interval)) as PrimitiveMethod, Right::Read),
    "to_string" => (PrimitiveNull::to_string as PrimitiveMethod, Right::Read),

};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrimitiveNull {}

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveNull {
    impl_type_check!(&mut Self, is_number, false);
    impl_type_check!(&mut Self, is_int, false);
    impl_type_check!(&mut Self, is_float, false);

    #[allow(clippy::needless_pass_by_value)]
    fn type_of(
        _null: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "type_of() => string";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        Ok(PrimitiveString::get_literal("Null", interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_info(
        _null: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        literal::get_info(&args, additional_info, interval, data)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn to_string(
        null: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "to_string() => string";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        Ok(PrimitiveString::get_literal(&null.to_string(), interval))
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveNull {
    #[must_use]
    pub fn get_literal(interval: Interval) -> Literal {
        let primitive = Box::<Self>::default();

        Literal {
            content_type: "null".to_owned(),
            primitive,
            additional_info: None,
            secure_variable: false,
            interval,
        }
    }
}

#[typetag::serde]
impl Primitive for PrimitiveNull {
    fn is_eq(&self, other: &dyn Primitive) -> bool {
        if let Some(_other) = other.as_any().downcast_ref::<Self>() {
            return true;
        }

        false
    }

    fn is_cmp(&self, _other: &dyn Primitive) -> Option<Ordering> {
        Some(Ordering::Equal)
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
        Box::new(NULL)
    }

    fn get_type(&self) -> PrimitiveType {
        PrimitiveType::PrimitiveNull
    }

    fn as_box_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn format_mem(&self, _content_type: &str, _first: bool) -> serde_json::Value {
        serde_json::Value::Null
    }

    fn to_string(&self) -> String {
        "Null".to_owned()
    }

    fn as_bool(&self) -> bool {
        false
    }

    fn get_value(&self) -> &dyn Any {
        &NULL
    }

    fn get_mut_value(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn to_msg(&self, _content_type: String) -> Message {
        let mut hashmap: HashMap<String, Literal> = HashMap::new();

        hashmap.insert(
            "text".to_owned(),
            Literal {
                content_type: "text".to_owned(),
                primitive: Box::<Self>::default(),
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

    impl_do_exec!(FUNCTIONS, ERROR_NULL_UNKNOWN_METHOD);
}
