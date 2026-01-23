use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::object::PrimitiveObject;
use crate::data::primitive::string::PrimitiveString;
use crate::data::primitive::utils::{
    illegal_math_ops, impl_basic_cmp, impl_do_exec, impl_type_check, require_n_args,
};
use crate::data::primitive::{Primitive, PrimitiveType};
use crate::data::primitive::{Right, common};
use crate::data::{Data, Literal, MSG, MemoryType, MessageData, ast::Interval, message::Message};
use crate::data::{literal, literal::ContentType};
use crate::error_format::{ERROR_BOOLEAN_UNKNOWN_METHOD, gen_error_info};
use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cmp::Ordering;
use std::{collections::HashMap, sync::mpsc};
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

type PrimitiveMethod = fn(
    boolean: &PrimitiveBoolean,
    args: HashMap<String, Literal>,
    additional_info: Option<&HashMap<String, Literal>>,
    data: &mut Data,
    interval: Interval,
) -> Result<Literal, ErrorInfo>;

const FUNCTIONS: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "is_number" => (PrimitiveBoolean::is_number as PrimitiveMethod, Right::Read),
    "is_int" => (PrimitiveBoolean::is_int as PrimitiveMethod, Right::Read),
    "is_float" => (PrimitiveBoolean::is_float as PrimitiveMethod, Right::Read),
    "type_of" => (PrimitiveBoolean::type_of as PrimitiveMethod, Right::Read),
    "is_error" => ((|_, _, additional_info, _, interval| common::is_error(additional_info, interval)) as PrimitiveMethod, Right::Read),
    "get_info" => (PrimitiveBoolean::get_info as PrimitiveMethod, Right::Read),
    "to_string" => (PrimitiveBoolean::to_string as PrimitiveMethod, Right::Read),
};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveBoolean {
    pub value: bool,
}

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveBoolean {
    impl_type_check!(is_number, false);
    impl_type_check!(is_int, false);
    impl_type_check!(is_float, false);

    #[allow(clippy::needless_pass_by_value)]
    fn type_of(
        _self: &Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "type_of() => string")?;

        Ok(PrimitiveString::get_literal("boolean", interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_info(
        _self: &Self,
        args: HashMap<String, Literal>,
        additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        literal::get_info(&args, additional_info, interval, data)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn to_string(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_string() => string")?;

        Ok(PrimitiveString::get_literal(
            &Primitive::to_string(self),
            interval,
        ))
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveBoolean {
    #[must_use]
    pub fn new(value: bool) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn get_literal(boolean: bool, interval: Interval) -> Literal {
        let primitive = Box::new(Self::new(boolean));

        Literal {
            content_type: "boolean".to_owned(),
            primitive,
            additional_info: None,
            secure_variable: false,
            interval,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// TRAIT FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[typetag::serde]
impl Primitive for PrimitiveBoolean {
    impl_do_exec!(FUNCTIONS, ERROR_BOOLEAN_UNKNOWN_METHOD);

    impl_basic_cmp!();

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
        PrimitiveType::PrimitiveBoolean
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
        self.value.to_string()
    }

    fn as_bool(&self) -> bool {
        self.value
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
                content_type: "boolean".to_owned(),
                primitive: Box::new(PrimitiveString::new(Primitive::to_string(self))),
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
}
