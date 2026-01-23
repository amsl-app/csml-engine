use crate::data::primitive::{PrimitiveString, PrimitiveType};
use crate::data::{Literal, Position};
use crate::error_format::{
    ERROR_OPS_DIV_FLOAT, ERROR_OPS_DIV_INT, ErrorInfo, OVERFLOWING_OPERATION, gen_error_info,
};
use std::borrow::Cow;

////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURE
////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub enum Integer {
    Int(i64),
    Float(f64),
}

impl Integer {
    pub(crate) fn to_f64(self) -> f64 {
        match self {
            #[allow(clippy::cast_precision_loss)]
            Integer::Int(int) => int as f64,
            Integer::Float(float) => float,
        }
    }

    pub(crate) fn to_i64(self) -> i64 {
        match self {
            Integer::Int(int) => int,
            #[allow(clippy::cast_possible_truncation)]
            Integer::Float(float) => float as i64,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[must_use]
pub fn get_integer(text: &str) -> Option<Integer> {
    if let Ok(int) = text.parse::<i64>() {
        return Some(Integer::Int(int));
    }
    if let Ok(float) = text.parse::<f64>() {
        return Some(Integer::Float(float));
    }
    None
}

pub fn get_array<E: Into<Cow<'static, str>>>(
    literal: &Literal,
    flow_name: &str,
    error_message: E,
) -> Result<Vec<Literal>, ErrorInfo> {
    match literal.primitive.get_type() {
        PrimitiveType::PrimitiveString => {
            let string = Literal::get_value::<String, _>(
                &literal.primitive,
                flow_name,
                literal.interval,
                error_message,
            )?;

            Ok(PrimitiveString::get_array_char(
                string.as_str(),
                literal.interval,
            ))
        }
        PrimitiveType::PrimitiveArray => Ok(Literal::get_value::<Vec<Literal>, _>(
            &literal.primitive,
            flow_name,
            literal.interval,
            error_message,
        )?
        .clone()),
        _ => Err(gen_error_info(
            Position::new(literal.interval, flow_name),
            error_message.into().into_owned(),
        )),
    }
}

pub fn check_division_preconditions(lhs: i64, rhs: i64) -> Result<(), String> {
    if rhs == 0 {
        return Err(ERROR_OPS_DIV_INT.to_owned());
    }
    if lhs == i64::MIN && rhs == -1 {
        return Err(OVERFLOWING_OPERATION.to_owned());
    }

    Ok(())
}

pub fn check_division_by_zero_f64(lhs: f64, rhs: f64) -> Result<f64, String> {
    if rhs == 0.0 {
        return Err(ERROR_OPS_DIV_FLOAT.to_owned());
    }

    Ok(lhs)
}
