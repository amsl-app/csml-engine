pub mod array;
pub mod boolean;
pub mod closure;
pub mod float;
pub mod int;
pub mod null;
pub mod object;
pub mod string;

mod common;
pub mod tools;
pub mod tools_crypto;
pub mod tools_jwt;
pub mod tools_smtp;
pub mod tools_time;
mod utils;

use crate::data::literal::ContentType;
pub use array::PrimitiveArray;
pub use boolean::PrimitiveBoolean;
pub use closure::PrimitiveClosure;
pub use float::PrimitiveFloat;
pub use int::PrimitiveInt;
pub use null::PrimitiveNull;
pub use object::PrimitiveObject;
pub use string::PrimitiveString;

use crate::data::primitive::tools::{Integer, get_integer};
use crate::data::{Data, Interval, Literal, MSG, MemoryType, Message, MessageData};
use crate::error_format::{ERROR_ILLEGAL_OPERATION, ErrorInfo};

use std::any::Any;
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::{collections::HashMap, sync::mpsc};
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Copy, Clone)]
pub enum Right {
    Read,
    Write,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PrimitiveType {
    PrimitiveArray,
    PrimitiveBoolean,
    PrimitiveFloat,
    PrimitiveInt,
    PrimitiveNull,
    PrimitiveObject,
    PrimitiveString,
    PrimitiveClosure,
}

#[typetag::serde(tag = "primitive")]
pub trait Primitive: Send {
    fn is_eq(&self, other: &dyn Primitive) -> bool;
    fn is_cmp(&self, other: &dyn Primitive) -> Option<Ordering>;
    fn do_add(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String>;
    fn do_sub(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String>;
    fn do_div(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String>;
    fn do_mul(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String>;
    fn do_rem(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String>;

    fn as_debug(&self) -> &dyn std::fmt::Debug;
    fn as_any(&self) -> &dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn into_value(self: Box<Self>) -> Box<dyn Any>;
    fn get_type(&self) -> PrimitiveType;
    fn as_box_clone(&self) -> Box<dyn Primitive>;
    fn to_json(&self) -> serde_json::Value;
    fn format_mem(&self, content_type: &str, first: bool) -> serde_json::Value;
    fn to_string(&self) -> String;
    fn as_bool(&self) -> bool;
    fn get_value(&self) -> &dyn Any;
    fn get_mut_value(&mut self) -> &mut dyn Any;
    fn to_msg(&self, content_type: String) -> Message;
    #[allow(clippy::too_many_arguments)]
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
    ) -> Result<(Literal, Right), ErrorInfo>;
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[allow(clippy::inherent_to_string)]
impl PrimitiveType {
    #[must_use]
    pub fn to_string(&self) -> String {
        match self {
            Self::PrimitiveArray => "array".to_owned(),
            Self::PrimitiveBoolean => "boolean".to_owned(),
            Self::PrimitiveFloat => "float".to_owned(),
            Self::PrimitiveInt => "int".to_owned(),
            Self::PrimitiveNull => "null".to_owned(),
            Self::PrimitiveObject => "object".to_owned(),
            Self::PrimitiveString => "string".to_owned(),
            Self::PrimitiveClosure => "closure".to_owned(),
        }
    }
}

impl dyn Primitive {
    #[allow(clippy::too_many_arguments)]
    pub fn exec(
        &mut self,
        name: &str,
        args: HashMap<String, Literal>,
        mem_type: &MemoryType,
        additional_info: Option<&HashMap<String, Literal>>,
        interval: Interval,
        content_type: &ContentType,
        mem_update: &mut bool,
        data: &mut Data,
        msg_data: &mut MessageData,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Result<Literal, ErrorInfo> {
        *mem_update = false;

        let (res, right) = self.do_exec(
            name,
            args,
            mem_type,
            additional_info,
            interval,
            content_type,
            data,
            msg_data,
            sender,
        )?;
        if right == Right::Write {
            *mem_update = true;
        }

        Ok(res)
    }
}

////////////////////////////////////////////////////////////////////////////////
// TRAIT FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl std::fmt::Debug for dyn Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{{\n\t{:?}\n}}", self.as_debug())
    }
}

impl Clone for Box<dyn Primitive> {
    fn clone(&self) -> Self {
        self.as_box_clone()
    }
}

fn to_float_value(primitive: &dyn Primitive) -> Option<f64> {
    let ty = primitive.get_type();
    match ty {
        PrimitiveType::PrimitiveFloat => Some(
            primitive
                .as_any()
                .downcast_ref::<PrimitiveFloat>()
                .unwrap()
                .value,
        ),
        PrimitiveType::PrimitiveInt => {
            let int = primitive.as_any().downcast_ref::<PrimitiveInt>().unwrap();
            #[allow(clippy::cast_precision_loss)]
            Some(int.value as f64)
        }
        PrimitiveType::PrimitiveString => {
            let string = primitive
                .as_any()
                .downcast_ref::<PrimitiveString>()
                .unwrap();
            get_integer(&string.value).map(Integer::to_f64)
        }
        _ => None,
    }
}

impl PartialEq for dyn Primitive {
    fn eq(&self, other: &Self) -> bool {
        let left_type = self.get_type();
        let right_type = other.get_type();
        if left_type == right_type {
            return self.is_eq(other);
        }
        let left = to_float_value(self);
        let right = to_float_value(other);
        let (Some(left), Some(right)) = (left, right) else {
            return false;
        };
        left == right
    }
}

impl PartialOrd for dyn Primitive {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left_type = self.get_type();
        let right_type = other.get_type();
        if left_type == right_type {
            return self.is_cmp(other);
        }
        let left = to_float_value(self);
        let right = to_float_value(other);
        let (Some(left), Some(right)) = (left, right) else {
            return None;
        };
        left.partial_cmp(&right)
    }
}

fn to_float(primitive: Box<dyn Primitive>) -> Option<Box<PrimitiveFloat>> {
    let ty = primitive.get_type();
    let primitive = primitive.into_any();
    match ty {
        PrimitiveType::PrimitiveFloat => Some(primitive.downcast::<PrimitiveFloat>().unwrap()),
        PrimitiveType::PrimitiveInt => {
            let int = primitive.downcast::<PrimitiveInt>().unwrap();
            #[allow(clippy::cast_precision_loss)]
            let float = PrimitiveFloat::new(int.value as f64);
            Some(Box::new(float))
        }
        PrimitiveType::PrimitiveString => {
            let string = primitive.downcast::<PrimitiveString>().unwrap();
            get_integer(&string.value)
                .map(Integer::to_f64)
                .map(|float| Box::new(PrimitiveFloat::new(float)))
        }
        _ => None,
    }
}

fn do_op<F: FnOnce(&dyn Primitive, &dyn Primitive) -> Result<Box<dyn Primitive>, String>>(
    left: Box<dyn Primitive>,
    right: Box<dyn Primitive>,
    f: F,
    op: &str,
) -> Result<Box<dyn Primitive>, String> {
    let left_type = left.get_type();
    let right_type = right.get_type();
    if left_type == right_type {
        return f(&*left, &*right);
    }
    let left_float = to_float(left);
    let right_float = to_float(right);
    let (Some(left_float), Some(right_float)) = (left_float, right_float) else {
        return Err(format!(
            "{ERROR_ILLEGAL_OPERATION} {left_type:?} {op} {right_type:?}"
        ));
    };
    f(&*left_float, &*right_float)
}

impl Add for Box<dyn Primitive> {
    type Output = Result<Self, String>;

    fn add(self, other: Self) -> Result<Self, String> {
        do_op(
            self,
            other,
            |left, right| Primitive::do_add(left, right),
            "+",
        )
    }
}

impl Sub for Box<dyn Primitive> {
    type Output = Result<Self, String>;

    fn sub(self, other: Self) -> Result<Self, String> {
        do_op(
            self,
            other,
            |left, right| Primitive::do_sub(left, right),
            "-",
        )
    }
}

impl Div for Box<dyn Primitive> {
    type Output = Result<Self, String>;

    fn div(self, other: Self) -> Result<Self, String> {
        do_op(
            self,
            other,
            |left, right| Primitive::do_div(left, right),
            "/",
        )
    }
}

impl Mul for Box<dyn Primitive> {
    type Output = Result<Self, String>;

    fn mul(self, other: Self) -> Result<Self, String> {
        do_op(
            self,
            other,
            |left, right| Primitive::do_mul(left, right),
            "*",
        )
    }
}

impl Rem for Box<dyn Primitive> {
    type Output = Result<Self, String>;

    fn rem(self, other: Self) -> Result<Self, String> {
        do_op(
            self,
            other,
            |left, right| Primitive::do_rem(left, right),
            "%",
        )
    }
}
