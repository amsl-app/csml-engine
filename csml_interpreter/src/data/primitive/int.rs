use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::boolean::PrimitiveBoolean;
use crate::data::primitive::float::PrimitiveFloat;
use crate::data::primitive::object::PrimitiveObject;
use crate::data::primitive::string::PrimitiveString;
use crate::data::primitive::tools::check_division_preconditions;
use crate::data::primitive::utils::{
    impl_basic_cmp, impl_do_exec, impl_type_check, pow_f64, require_n_args,
};
use crate::data::primitive::{Primitive, PrimitiveType};
use crate::data::primitive::{Right, common};
use crate::data::{Data, Literal, MSG, MemoryType, MessageData, ast::Interval, message::Message};
use crate::data::{literal, literal::ContentType};
use crate::error_format::{
    ERROR_ILLEGAL_OPERATION, ERROR_INT_UNKNOWN_METHOD, OVERFLOWING_OPERATION, gen_error_info,
};
use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cmp::Ordering;
use std::{collections::HashMap, sync::mpsc};
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

type PrimitiveMethod = fn(
    &PrimitiveInt,
    args: HashMap<String, Literal>,
    additional_info: Option<&HashMap<String, Literal>>,
    data: &mut Data,
    interval: Interval,
) -> Result<Literal, ErrorInfo>;

const FUNCTIONS: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "is_number" => (PrimitiveInt::is_number as PrimitiveMethod, Right::Read),
    "is_int" => (PrimitiveInt::is_int as PrimitiveMethod, Right::Read),
    "is_float" => (PrimitiveInt::is_float as PrimitiveMethod, Right::Read),
    "type_of" => (PrimitiveInt::type_of as PrimitiveMethod, Right::Read),
    "is_error" => ((|_, _, additional_info, _, interval| common::is_error(additional_info, interval)) as PrimitiveMethod, Right::Read),
    "get_info" => (PrimitiveInt::get_info as PrimitiveMethod, Right::Read),
    "to_string" => (PrimitiveInt::to_string as PrimitiveMethod, Right::Read),

    "precision" => (PrimitiveInt::precision as PrimitiveMethod, Right::Read),
    "abs" => (PrimitiveInt::abs as PrimitiveMethod, Right::Read),
    "cos" => (PrimitiveInt::cos as PrimitiveMethod, Right::Read),
    "ceil" => (PrimitiveInt::ceil as PrimitiveMethod, Right::Read),
    "floor" => (PrimitiveInt::floor as PrimitiveMethod, Right::Read),
    "pow" => (PrimitiveInt::pow as PrimitiveMethod, Right::Read),
    "round" => (PrimitiveInt::round as PrimitiveMethod, Right::Read),
    "sin" => (PrimitiveInt::sin as PrimitiveMethod, Right::Read),
    "sqrt" => (PrimitiveInt::sqrt as PrimitiveMethod, Right::Read),
    "tan" => (PrimitiveInt::tan as PrimitiveMethod, Right::Read),
    "to_int" => (PrimitiveInt::to_int as PrimitiveMethod, Right::Read),
    "to_float" => (PrimitiveInt::to_float as PrimitiveMethod, Right::Read),
};
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveInt {
    pub value: i64,
}

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveInt {
    impl_type_check!(is_number, true);
    impl_type_check!(is_int, true);
    impl_type_check!(is_float, false);

    #[allow(clippy::needless_pass_by_value)]
    fn type_of(
        _self: &Self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "usage: type_of() => string".to_string(),
            ));
        }

        Ok(PrimitiveString::get_literal("int", interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_info(
        _self: &PrimitiveInt,
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
        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "usage: to_string() => string".to_string(),
            ));
        }

        Ok(PrimitiveString::get_literal(
            &Primitive::to_string(self),
            interval,
        ))
    }
}

impl PrimitiveInt {
    #[allow(clippy::needless_pass_by_value)]
    fn abs(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "abs() => int")?;

        let result = self.value.checked_abs().ok_or_else(|| {
            gen_error_info(
                Position::new(interval, &data.context.flow),
                OVERFLOWING_OPERATION.to_string(),
            )
        })?;

        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn cos(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "usage: cos() => number".to_string(),
            ));
        }

        #[allow(clippy::cast_precision_loss)]
        let float = self.value as f64;

        let result = float.cos();

        Ok(f64_to_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn ceil(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "ceil() => int")?;
        Ok(Self::get_literal(self.value, interval))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn precision(
        &self,
        _args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        _data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        Ok(Self::get_literal(self.value, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn floor(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "floor() => int")?;
        Ok(Self::get_literal(self.value, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn pow(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        #[allow(clippy::cast_precision_loss)]
        let result = pow_f64(self.value as f64, &args, data, interval)?;
        Ok(f64_to_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn round(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "round() => int")?;
        Ok(Self::get_literal(self.value, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn sin(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "sin() => number")?;

        #[allow(clippy::cast_precision_loss)]
        let float = self.value as f64;

        let result = float.sin();

        Ok(f64_to_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn sqrt(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "sqrt() => number")?;

        #[allow(clippy::cast_precision_loss)]
        let float = self.value as f64;

        let result = float.sqrt();

        Ok(f64_to_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn tan(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "tan() => number")?;

        #[allow(clippy::cast_precision_loss)]
        let float = self.value as f64;

        let result = float.tan();

        Ok(f64_to_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn to_int(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_int() => int")?;

        Ok(Self::get_literal(self.value, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn to_float(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        require_n_args(0, &args, interval, data, "to_float() => float")?;

        #[allow(clippy::cast_precision_loss)]
        Ok(PrimitiveFloat::get_literal(self.value as f64, interval))
    }
}

fn f64_to_literal(value: f64, interval: Interval) -> Literal {
    // The comparison is fine because we just want to see if we can represent
    // value as an exact i64
    #[allow(
        clippy::float_cmp,
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation
    )]
    if value == (value as i64) as f64 {
        PrimitiveInt::get_literal(value as i64, interval)
    } else {
        PrimitiveFloat::get_literal(value, interval)
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveInt {
    #[must_use]
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn get_literal(int: i64, interval: Interval) -> Literal {
        let primitive = Box::new(Self::new(int));

        Literal {
            content_type: "int".to_owned(),
            primitive,
            additional_info: None,
            secure_variable: false,
            interval,
        }
    }

    fn do_op(
        &self,
        other: &dyn Primitive,
        op: fn(i64, i64) -> Option<i64>,
        op_str: &str,
    ) -> Result<Box<dyn Primitive>, String> {
        let error = if let Some(other) = other.as_any().downcast_ref::<Self>() {
            if let Some(value) = op(self.value, other.value) {
                return Ok(Box::new(Self::new(value)));
            }
            OVERFLOWING_OPERATION
        } else {
            ERROR_ILLEGAL_OPERATION
        };

        Err(format!(
            "{} {:?} {op_str} {:?}",
            error,
            self.get_type(),
            other.get_type()
        ))
    }
}

////////////////////////////////////////////////////////////////////////////////
/// TRAIT FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[typetag::serde]
impl Primitive for PrimitiveInt {
    impl_basic_cmp!();

    fn do_add(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        self.do_op(other, i64::checked_add, "+")
    }

    fn do_sub(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        self.do_op(other, i64::checked_sub, "-")
    }

    fn do_div(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            check_division_preconditions(self.value, other.value)?;

            let value: Box<dyn Primitive> = if self.value % other.value != 0 {
                #[allow(clippy::cast_precision_loss)]
                let value = self.value as f64 / other.value as f64;

                Box::new(PrimitiveFloat::new(value))
            } else {
                Box::new(Self::new(self.value / other.value))
            };
            return Ok(value);
        }

        Err(format!(
            "{} {:?} / {:?}",
            ERROR_ILLEGAL_OPERATION,
            self.get_type(),
            other.get_type()
        ))
    }

    fn do_mul(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        self.do_op(other, i64::checked_mul, "*")
    }

    fn do_rem(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        self.do_op(other, i64::checked_rem, "%")
    }

    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn into_value(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self.value)
    }

    fn get_type(&self) -> PrimitiveType {
        PrimitiveType::PrimitiveInt
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
        self.value.is_positive()
    }

    fn get_value(&self) -> &dyn std::any::Any {
        &self.value
    }

    fn get_mut_value(&mut self) -> &mut dyn std::any::Any {
        &mut self.value
    }

    fn to_msg(&self, _content_type: String) -> Message {
        let mut hashmap: HashMap<String, Literal> = HashMap::new();

        hashmap.insert(
            "text".to_owned(),
            Literal {
                content_type: "int".to_owned(),
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

    impl_do_exec!(FUNCTIONS, ERROR_INT_UNKNOWN_METHOD);
}
