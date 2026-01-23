use crate::data::primitive::{Primitive, PrimitiveType};
use crate::data::{Data, Interval, Literal, Position};
use crate::error_format::{ERROR_ILLEGAL_OPERATION, ERROR_NUMBER_POW, ErrorInfo, gen_error_info};
use std::collections::HashMap;
macro_rules! impl_do_exec {
    ($functions:ident, $err:ident) => {
        fn do_exec(
            &mut self,
            name: &str,
            args: HashMap<String, Literal>,
            mem_type: &MemoryType,
            additional_info: Option<&HashMap<String, Literal>>,
            interval: Interval,
            _content_type: &ContentType,
            data: &mut Data,
            _msg_data: &mut MessageData,
            _sender: Option<&mpsc::Sender<MSG>>,
        ) -> Result<(Literal, Right), ErrorInfo> {
            let Some((f, right)) = $functions.get(name) else {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    format!("[{name}] {}", $err),
                ));
            };

            if *mem_type == MemoryType::Constant && *right == Right::Write {
                return Err(gen_error_info(
                    Position::new(interval, &data.context.flow),
                    crate::error_format::ERROR_CONSTANT_MUTABLE_FUNCTION.to_string(),
                ));
            }
            let res = f(self, args, additional_info, data, interval)?;

            Ok((res, *right))
        }
    };
}

pub(crate) use impl_do_exec;

macro_rules! impl_type_check {
    ($self:ty, $name:ident, $res:literal) => {
        fn $name(
            self: $self,
            args: HashMap<String, Literal>,
            _additional_info: Option<&HashMap<String, Literal>>,
            data: &mut Data,
            interval: Interval,
        ) -> Result<Literal, ErrorInfo> {
            crate::data::primitive::utils::require_n_args(
                0,
                &args,
                interval,
                data,
                concat!(stringify!($name), "() => boolean"),
            )?;

            Ok(PrimitiveBoolean::get_literal($res, interval))
        }
    };
    ($name:ident, $res:literal) => {
        impl_type_check!(&Self, $name, $res);
    };
}

pub(crate) use impl_type_check;

fn literal_to_f64(
    exponent: &Literal,
    data: &mut Data,
    interval: Interval,
) -> Result<f64, ErrorInfo> {
    if exponent.primitive.get_type() == PrimitiveType::PrimitiveInt {
        #[allow(clippy::cast_precision_loss)]
        return Literal::get_value::<i64, _>(
            &exponent.primitive,
            &data.context.flow,
            interval,
            ERROR_NUMBER_POW,
        )
        .copied()
        .map(|res| res as f64);
    }
    if exponent.primitive.get_type() == PrimitiveType::PrimitiveFloat {
        return Literal::get_value::<f64, _>(
            &exponent.primitive,
            &data.context.flow,
            interval,
            ERROR_NUMBER_POW,
        )
        .copied();
    }
    if exponent.primitive.get_type() == PrimitiveType::PrimitiveString {
        let exponent = Literal::get_value::<String, _>(
            &exponent.primitive,
            &data.context.flow,
            interval,
            ERROR_NUMBER_POW,
        )?;

        if let Ok(res) = exponent.parse::<f64>() {
            return Ok(res);
        }
    }
    Err(gen_error_info(
        Position::new(interval, &data.context.flow),
        ERROR_NUMBER_POW.to_owned(),
    ))
}

pub(crate) fn pow_f64(
    value: f64,
    args: &HashMap<String, Literal>,
    data: &mut Data,
    interval: Interval,
) -> Result<f64, ErrorInfo> {
    require_n_args(1, args, interval, data, "pow(exponent: number) => float")?;

    let Some(exponent) = args.get("arg0") else {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_NUMBER_POW.to_owned(),
        ));
    };
    let exponent = literal_to_f64(exponent, data, interval)?;
    let result = value.powf(exponent);
    Ok(result)
}

pub(crate) fn illegal_op<T: Primitive>(
    this: &T,
    other: &dyn Primitive,
    op: &str,
) -> Result<Box<dyn Primitive>, String> {
    Err(format!(
        "{} {:?} {op} {:?}",
        ERROR_ILLEGAL_OPERATION,
        this.get_type(),
        other.get_type()
    ))
}

macro_rules! illegal_math_ops {
    () => {
        fn do_add(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
            crate::data::primitive::utils::illegal_op(self, other, "+")
        }

        fn do_sub(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
            crate::data::primitive::utils::illegal_op(self, other, "-")
        }

        fn do_div(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
            crate::data::primitive::utils::illegal_op(self, other, "/")
        }

        fn do_mul(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
            crate::data::primitive::utils::illegal_op(self, other, "*")
        }

        fn do_rem(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, String> {
            crate::data::primitive::utils::illegal_op(self, other, "%")
        }
    };
}

pub(crate) use illegal_math_ops;

macro_rules! impl_basic_cmp {
    () => {
        fn is_eq(&self, other: &dyn Primitive) -> bool {
            if let Some(other) = other.as_any().downcast_ref::<Self>() {
                return self.value == other.value;
            }

            false
        }

        fn is_cmp(&self, other: &dyn Primitive) -> Option<Ordering> {
            if let Some(other) = other.as_any().downcast_ref::<Self>() {
                return self.value.partial_cmp(&other.value);
            }

            None
        }
    };
}

pub(crate) use impl_basic_cmp;

macro_rules! arg_name {
    ($name:literal) => {
        concat!("arg", $name)
    };
}

pub(crate) use arg_name;

pub(crate) fn require_n_args(
    n: usize,
    args: &HashMap<String, Literal>,
    interval: Interval,
    data: &Data,
    usage: &str,
) -> Result<(), ErrorInfo> {
    if args.len() != n {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            format!("usage: {usage}"),
        ));
    }

    Ok(())
}
