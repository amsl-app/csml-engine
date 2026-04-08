use crate::data::position::Position;
use crate::data::primitive::{
    PrimitiveArray, PrimitiveBoolean, PrimitiveFloat, PrimitiveInt, PrimitiveString,
};
use num_traits::ToPrimitive;
use std::array;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::RngExt;
use crate::data::{ArgsType, Literal, ast::Interval};
use crate::error_format::{
    ERROR_FIND, ERROR_FLOOR, ERROR_LENGTH, ERROR_LENGTH_OVERFLOW, ERROR_ONE_OF, ERROR_SHUFFLE,
    ERROR_UUID, ErrorInfo, gen_error_info,
};
use uuid::{ContextV1, Uuid};
use uuid::v1::{Timestamp};

use rand::seq::SliceRandom;

pub(crate) fn one_of(
    mut args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let Some(literal) = args.remove("array", 0) else {
        return Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_ONE_OF.to_owned(),
        ));
    };

    let (mut res, _) = literal
        .cast_into_value::<Vec<Literal>>()
        .map_err(|interval| {
            gen_error_info(Position::new(interval, flow_name), ERROR_ONE_OF.to_owned())
        })?;
    Ok(res.remove(rand::rng().random_range(0..res.len())))
}

pub(crate) fn or(
    mut args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let (Some(first_val), Some(optional_value)) = (args.remove("arg0", 0), args.remove("arg1", 1))
    else {
        return Err(gen_error_info(
            Position::new(interval, flow_name),
            "ERROR_SHUFFLE".to_owned(),
        ));
    };

    if let Some(map) = &first_val.additional_info
        && map.contains_key("error")
    {
        return Ok(optional_value);
    }
    Ok(first_val)
}

pub(crate) fn shuffle(
    mut args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    if let Some(literal) = args.remove("array", 0) {
        let (mut literal, interval) = literal.cast_into_value::<Vec<Literal>>().map_err(|_| {
            gen_error_info(Position::new(interval, flow_name), ERROR_SHUFFLE.to_owned())
        })?;
        literal.shuffle(&mut rand::rng());
        return Ok(PrimitiveArray::get_literal(*literal, interval));
    }

    Err(gen_error_info(
        Position::new(interval, flow_name),
        ERROR_SHUFFLE.to_owned(),
    ))
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn length(
    args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let Some(literal) = args.get("length", 0) else {
        return Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_LENGTH.to_owned(),
        ));
    };
    let len = if let Ok(res) =
        Literal::get_value::<Vec<Literal>, _>(&literal.primitive, flow_name, interval, ERROR_LENGTH)
    {
        res.len()
    } else {
        let Ok(res) =
            Literal::get_value::<String, _>(&literal.primitive, flow_name, interval, ERROR_LENGTH)
        else {
            return Err(gen_error_info(
                Position::new(interval, flow_name),
                ERROR_LENGTH.to_owned(),
            ));
        };

        res.len()
    };
    let len = len.to_i64().ok_or(gen_error_info(
        Position::new(interval, flow_name),
        ERROR_LENGTH_OVERFLOW.to_owned(),
    ))?;
    Ok(PrimitiveInt::get_literal(len, literal.interval))
}

pub(crate) fn find(
    mut args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let (Some(string_literal), Some(pattern_literal)) =
        (args.remove("in", 1), args.remove("value", 0))
    else {
        return Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_FIND.to_owned(),
        ));
    };

    let string = Literal::get_value::<String, _>(
        &string_literal.primitive,
        flow_name,
        interval,
        ERROR_FIND,
    )?;

    let pattern = Literal::get_value::<String, _>(
        &pattern_literal.primitive,
        flow_name,
        interval,
        ERROR_FIND,
    )?;

    let case =
        Literal::get_value::<bool, _>(&string_literal.primitive, flow_name, interval, ERROR_FIND)
            .is_ok_and(|val| *val);

    if case {
        Ok(PrimitiveBoolean::get_literal(
            string.contains(pattern),
            interval,
        ))
    } else {
        Ok(PrimitiveBoolean::get_literal(
            string.to_lowercase().contains(&pattern.to_lowercase()),
            interval,
        ))
    }
}

pub(crate) fn random(interval: Interval) -> Literal {
    let mut rng = rand::rng();

    let random: f64 = rng.random();

    PrimitiveFloat::get_literal(random, interval)
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn floor(
    args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let Some(literal) = args.get("float", 0) else {
        return Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_FLOOR.to_owned(),
        ));
    };
    let res = Literal::get_value::<f64, _>(&literal.primitive, flow_name, interval, ERROR_FLOOR)?;
    Ok(PrimitiveFloat::get_literal(res.floor(), literal.interval))
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn uuid_command(
    args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    if args.is_empty() {
        return Ok(PrimitiveString::get_literal(
            &Uuid::new_v4().to_string(),
            interval,
        ));
    }

    let Some(literal) = args.get("value", 0) else {
        return Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_UUID.to_owned(),
        ));
    };

    let arg =
        Literal::get_value::<String, _>(&literal.primitive, flow_name, interval, ERROR_FLOOR)?;

    match arg.as_str() {
        "v1" => {
            let mut rng = rand::rng();
            let time = SystemTime::now().duration_since(UNIX_EPOCH)?;
            let context = ContextV1::new(rng.random());
            let ts = Timestamp::from_unix(context, time.as_secs(), time.subsec_nanos());

            let node_id: [u8; 6] = array::from_fn(|_| rng.random());
            Ok(PrimitiveString::get_literal(
                &Uuid::new_v1(ts, &node_id).hyphenated().to_string(),
                interval,
            ))
        }
        "v4" => Ok(PrimitiveString::get_literal(
            &Uuid::new_v4().to_string(),
            interval,
        )),
        _ => Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_UUID.to_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_shuffle() {
        let args = ArgsType::Normal(HashMap::from([(
            "arg0".to_owned(),
            PrimitiveArray::get_literal(
                vec![
                    PrimitiveInt::get_literal(1, Interval::default()),
                    PrimitiveInt::get_literal(2, Interval::default()),
                ],
                Interval::default(),
            ),
        )]));

        let literal = shuffle(args, "test", Interval::default()).unwrap();
        let vec = Literal::cast_value::<Vec<Literal>>(literal.primitive.as_ref()).unwrap();
        let mut vec = vec.clone();
        let v1 = vec.pop().unwrap();
        let v2 = vec.pop().unwrap();
        assert!(vec.is_empty());
        let v1 = Literal::cast_value::<i64>(v1.primitive.as_ref()).unwrap();
        let v2 = Literal::cast_value::<i64>(v2.primitive.as_ref()).unwrap();
        assert_eq!((v1 - v2).abs(), 1);
    }

    #[test]
    fn test_uuid_command() {
        let args = ArgsType::Named(HashMap::from([(
            "value".to_owned(),
            PrimitiveString::get_literal("v4", Interval::default()),
        )]));

        let literal = uuid_command(args, "test", Interval::default()).unwrap();
        let uuid = Literal::cast_value::<String>(literal.primitive.as_ref()).unwrap();
        assert_eq!(uuid.len(), 36);
    }
}
