use crate::data::primitive::common;
use crate::data::primitive::common::get_int_args;
use crate::data::primitive::tools::check_division_by_zero_f64;
use crate::data::primitive::utils::{impl_basic_cmp, impl_do_exec, impl_type_check, pow_f64};
use crate::data::{
    Data, Literal, MSG, MemoryType, MessageData,
    ast::Interval,
    error_info::ErrorInfo,
    literal,
    literal::ContentType,
    message::Message,
    position::Position,
    primitive::{
        Primitive, PrimitiveBoolean, PrimitiveInt, PrimitiveObject, PrimitiveString, PrimitiveType,
        Right,
    },
};
use crate::error_format::{
    ERROR_FLOAT_UNKNOWN_METHOD, ERROR_ILLEGAL_OPERATION, OVERFLOWING_OPERATION, gen_error_info,
};
use num_traits::ToPrimitive;
use phf::phf_map;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cmp::Ordering;
use std::{collections::HashMap, sync::mpsc};
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

type PrimitiveMethod = fn(
    float: &PrimitiveFloat,
    args: HashMap<String, Literal>,
    additional_info: Option<&HashMap<String, Literal>>,
    data: &mut Data,
    interval: Interval,
) -> Result<Literal, ErrorInfo>;

const FUNCTIONS: phf::Map<&'static str, (PrimitiveMethod, Right)> = phf_map! {
    "is_number" => (PrimitiveFloat::is_number as PrimitiveMethod, Right::Read),
    "is_int" => (PrimitiveFloat::is_int as PrimitiveMethod, Right::Read),
    "is_float" => (PrimitiveFloat::is_float as PrimitiveMethod, Right::Read),
    "type_of" => (PrimitiveFloat::type_of as PrimitiveMethod, Right::Read),
    "is_error" => ((|_, _, additional_info, _, interval| common::is_error(additional_info, interval)) as PrimitiveMethod, Right::Read),
    "get_info" => (PrimitiveFloat::get_info as PrimitiveMethod, Right::Read),
    "to_string" => (PrimitiveFloat::to_string as PrimitiveMethod, Right::Read),

    "precision" => (PrimitiveFloat::precision as PrimitiveMethod, Right::Read),
    "abs" => (PrimitiveFloat::abs as PrimitiveMethod, Right::Read),
    "cos" => (PrimitiveFloat::cos as PrimitiveMethod, Right::Read),
    "ceil" => (PrimitiveFloat::ceil as PrimitiveMethod, Right::Read),
    "floor" => (PrimitiveFloat::floor as PrimitiveMethod, Right::Read),
    "pow" => (PrimitiveFloat::pow as PrimitiveMethod, Right::Read),
    "round" => (PrimitiveFloat::round as PrimitiveMethod, Right::Read),
    "sin" => (PrimitiveFloat::sin as PrimitiveMethod, Right::Read),
    "sqrt" => (PrimitiveFloat::sqrt as PrimitiveMethod, Right::Read),
    "tan" => (PrimitiveFloat::tan as PrimitiveMethod, Right::Read),
    "to_int" => (PrimitiveFloat::to_int as PrimitiveMethod, Right::Read),
    "to_float" => (PrimitiveFloat::to_float as PrimitiveMethod, Right::Read),
};
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveFloat {
    pub value: f64,
}

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveFloat {
    impl_type_check!(is_number, true);
    impl_type_check!(is_int, false);
    impl_type_check!(is_float, true);

    #[allow(clippy::needless_pass_by_value)]
    fn type_of(
        _self: &PrimitiveFloat,
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

        Ok(PrimitiveString::get_literal("float", interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn get_info(
        _self: &PrimitiveFloat,
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
        let usage = "to_string() => string";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        Ok(PrimitiveString::get_literal(
            &Primitive::to_string(self),
            interval,
        ))
    }
}

impl PrimitiveFloat {
    #[allow(clippy::needless_pass_by_value)]
    fn abs(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "abs() => float";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let result = self.value.abs();

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
        let usage = "cos() => float";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let result = self.value.cos();

        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn ceil(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "ceil() => float";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let result = self.value.ceil();

        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn precision(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "precision(value) => float";

        let [precision]: [usize; 1] =
            get_int_args(&args, data, interval, format!("usage {usage}"), usage)?;

        let result = format!("{:.*}", precision, self.value)
            .parse::<f64>()
            .unwrap_or(self.value);

        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn floor(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "floor() => float";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let result = self.value.floor();

        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn pow(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let result = pow_f64(self.value, &args, data, interval)?;
        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn round(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "round() => float";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let result = self.value.round();

        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn sin(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "sin() => float";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let result = self.value.sin();

        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn sqrt(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        let usage = "sqrt() => float";

        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                format!("usage: {usage}"),
            ));
        }

        let result = self.value.sqrt();

        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn tan(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "usage: tan() => float".to_string(),
            ));
        }

        let result = self.value.tan();

        Ok(Self::get_literal(result, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn to_int(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "usage: to_int() => int".to_string(),
            ));
        }

        // Truncation is intentional
        #[allow(clippy::cast_possible_truncation)]
        Ok(PrimitiveInt::get_literal(self.value as i64, interval))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn to_float(
        &self,
        args: HashMap<String, Literal>,
        _additional_info: Option<&HashMap<String, Literal>>,
        data: &mut Data,
        interval: Interval,
    ) -> Result<Literal, ErrorInfo> {
        if !args.is_empty() {
            return Err(gen_error_info(
                Position::new(interval, &data.context.flow),
                "usage: to_float() => float".to_string(),
            ));
        }

        Ok(Self::get_literal(self.value, interval))
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveFloat {
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn get_literal(float: f64, interval: Interval) -> Literal {
        let primitive = Box::new(Self::new(float));

        Literal {
            content_type: "float".to_owned(),
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
impl Primitive for PrimitiveFloat {
    impl_basic_cmp!();

    fn do_add(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        let mut error_msg = ERROR_ILLEGAL_OPERATION;

        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let lhs = self.value.ceil().to_i64();
            let rhs = other.value.ceil().to_i64();

            if let (Some(lhs), Some(rhs)) = (lhs, rhs)
                && lhs.checked_add(rhs).is_some()
            {
                return Ok(Box::new(Self::new(self.value + other.value)));
            }

            error_msg = OVERFLOWING_OPERATION;
        }

        Err(format!(
            "{} {:?} + {:?}",
            error_msg,
            self.get_type(),
            other.get_type()
        ))
    }

    fn do_sub(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        let mut error_msg = ERROR_ILLEGAL_OPERATION;

        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let lhs = self.value.ceil().to_i64();
            let rhs = other.value.ceil().to_i64();

            if let (Some(lhs), Some(rhs)) = (lhs, rhs)
                && lhs.checked_sub(rhs).is_some()
            {
                return Ok(Box::new(Self::new(self.value - other.value)));
            }

            error_msg = OVERFLOWING_OPERATION;
        }

        Err(format!(
            "{} {:?} - {:?}",
            error_msg,
            self.get_type(),
            other.get_type()
        ))
    }

    fn do_div(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        let mut error_msg = ERROR_ILLEGAL_OPERATION;

        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            check_division_by_zero_f64(self.value, other.value)?;

            // Pessimize (make a large as possible -> fail fast)
            // the div result by maximizing the quotient and minimizing the divisor
            let lhs = self.value.ceil().to_i64();
            let rhs = other.value.floor().to_i64();

            if let (Some(lhs), Some(rhs)) = (lhs, rhs)
                && lhs.checked_div(rhs).is_some()
            {
                return Ok(Box::new(Self::new(self.value / other.value)));
            }

            error_msg = OVERFLOWING_OPERATION;
        }

        Err(format!(
            "{} {:?} / {:?}",
            error_msg,
            self.get_type(),
            other.get_type()
        ))
    }

    fn do_mul(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        let mut error_msg = ERROR_ILLEGAL_OPERATION;

        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let lhs = self.value.ceil().to_i64();
            let rhs = other.value.ceil().to_i64();

            if let (Some(lhs), Some(rhs)) = (lhs, rhs)
                && lhs.checked_mul(rhs).is_some()
            {
                return Ok(Box::new(Self::new(self.value * other.value)));
            }

            error_msg = OVERFLOWING_OPERATION;
        }

        Err(format!(
            "{} {:?} * {:?}",
            error_msg,
            self.get_type(),
            other.get_type()
        ))
    }

    fn do_rem(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
        let mut error_msg = ERROR_ILLEGAL_OPERATION;

        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            // Pessimize (make a large as possible -> fail fast)
            // the div result by maximizing the quotient and minimizing the divisor
            let lhs = self.value.ceil().to_i64();
            let rhs = other.value.floor().to_i64();

            if let (Some(lhs), Some(rhs)) = (lhs, rhs)
                && lhs.checked_rem(rhs).is_some()
            {
                return Ok(Box::new(Self::new(self.value % other.value)));
            }

            error_msg = OVERFLOWING_OPERATION;
        }

        Err(format!(
            "{} {:?} % {:?}",
            error_msg,
            self.get_type(),
            other.get_type()
        ))
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
        PrimitiveType::PrimitiveFloat
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
        self.value.is_normal()
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
                content_type: "float".to_owned(),
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

    impl_do_exec!(FUNCTIONS, ERROR_FLOAT_UNKNOWN_METHOD);
}
