use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::boolean::PrimitiveBoolean;
use crate::data::primitive::string::PrimitiveString;
use crate::data::{literal, literal::ContentType};

use crate::data::primitive::utils::{
    illegal_math_ops, impl_do_exec, impl_type_check, require_n_args,
};
use crate::data::primitive::{Primitive, PrimitiveType};
use crate::data::primitive::{Right, common};
use crate::data::{
    Data, Literal, MSG, MemoryType, MessageData,
    ast::{Expr, Interval},
    message::Message,
};
use crate::error_format::{ERROR_CLOSURE_UNKNOWN_METHOD, gen_error_info};
use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cmp::Ordering;
use std::{collections::HashMap, sync::mpsc};

#[allow(clippy::implicit_hasher)]
pub fn capture_variables(
    literal: &mut Literal,
    memories: HashMap<String, Literal>,
    flow_name: &str,
) {
    if literal.content_type == "closure" {
        let closure = Literal::get_mut_value::<PrimitiveClosure>(
            &mut literal.primitive,
            flow_name,
            literal.interval,
            String::new(),
        )
        .unwrap();
        closure.enclosed_variables = Some(memories);
    }
}

////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

const FUNCTIONS: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "is_number" => (PrimitiveClosure::is_number as PrimitiveMethod, Right::Read),
    "is_int" => (PrimitiveClosure::is_int as PrimitiveMethod, Right::Read),
    "is_float" => (PrimitiveClosure::is_float as PrimitiveMethod, Right::Read),
    "type_of" => (PrimitiveClosure::type_of as PrimitiveMethod, Right::Read),
    "is_error" => ((|_, _, additional_info, _, interval| common::is_error(additional_info, interval)) as PrimitiveMethod, Right::Read),
    "get_info" => (PrimitiveClosure::get_info as PrimitiveMethod, Right::Read),
    "to_string" => (PrimitiveClosure::to_string as PrimitiveMethod, Right::Read),
};

type PrimitiveMethod = fn(
    int: &mut PrimitiveClosure,
    args: HashMap<String, Literal>,
    additional_info: Option<&HashMap<String, Literal>>,
    data: &mut Data,
    interval: Interval,
) -> Result<Literal, ErrorInfo>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveClosure {
    pub args: Vec<String>,
    pub func: Box<Expr>,
    pub enclosed_variables: Option<HashMap<String, Literal>>,
}

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveClosure {
    impl_type_check!(&mut Self, is_number, false);
    impl_type_check!(&mut Self, is_int, false);
    impl_type_check!(&mut Self, is_float, false);

    #[allow(clippy::needless_pass_by_value)]
    fn type_of(
        _int: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "type_of() => string")?;

        Ok(PrimitiveString::get_literal("closure", interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_info(
        _closure: &mut Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        literal::get_info(&args, additional_info, interval, data)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn to_string(
        closure: &mut Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_string() => string")?;

        Ok(PrimitiveString::get_literal(&closure.to_string(), interval))
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveClosure {
    #[must_use]
    pub fn new(
        args: Vec<String>,
        func: Box<Expr>,
        enclosed_variables: Option<HashMap<String, Literal>>,
    ) -> Self {
        Self {
            args,
            func,
            enclosed_variables,
        }
    }

    #[must_use]
    pub fn get_literal(
        args: Vec<String>,
        func: Box<Expr>,
        interval: Interval,
        enclosed_variables: Option<HashMap<String, Literal>>,
    ) -> Literal {
        let primitive = Box::new(Self::new(args, func, enclosed_variables));

        Literal {
            content_type: "closure".to_owned(),
            primitive,
            additional_info: None,
            secure_variable: false,
            interval,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
/// TRAIT FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[typetag::serde]
impl Primitive for PrimitiveClosure {
    fn is_eq(&self, _other: &dyn Primitive) -> bool {
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

    fn into_value(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn get_type(&self) -> PrimitiveType {
        PrimitiveType::PrimitiveClosure
    }

    fn to_json(&self) -> serde_json::Value {
        let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        map.insert("_closure".to_owned(), serde_json::json!(self));

        serde_json::Value::Object(map)
    }

    fn format_mem(&self, _content_type: &str, _first: bool) -> serde_json::Value {
        let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        map.insert("_closure".to_owned(), serde_json::json!(self));

        serde_json::Value::Object(map)
    }

    fn to_string(&self) -> String {
        "Null".to_owned()
    }

    fn as_bool(&self) -> bool {
        false
    }

    fn get_value(&self) -> &dyn Any {
        self
    }

    fn get_mut_value(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_box_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn to_msg(&self, content_type: String) -> Message {
        Message {
            content_type,
            content: self.to_json(),
        }
    }

    impl_do_exec!(FUNCTIONS, ERROR_CLOSURE_UNKNOWN_METHOD);
}
