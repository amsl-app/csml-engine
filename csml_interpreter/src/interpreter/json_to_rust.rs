use crate::data::position::Position;
use crate::data::primitive::{
    PrimitiveArray, PrimitiveBoolean, PrimitiveClosure, PrimitiveFloat, PrimitiveInt,
    PrimitiveNull, PrimitiveObject, PrimitiveString,
};
use crate::data::{Data, Literal, MSG, MessageData, ast::Interval};
use crate::error_format::{ERROR_JSON_TO_LITERAL, ErrorInfo, gen_error_info};
use crate::parser::parse_string::interpolate_string;
use std::{collections::HashMap, sync::mpsc};

pub(crate) fn interpolate(
    literal: &serde_json::Value,
    interval: Interval,
    data: &mut Data,
    msg_data: &mut MessageData,
    sender: Option<&mpsc::Sender<MSG>>,
) -> Result<Literal, ErrorInfo> {
    let serde_json::Value::String(val) = literal else {
        return json_to_literal(literal, interval, &data.context.flow);
    };
    interpolate_string(val, data, msg_data, sender)
}

pub fn json_to_literal(
    literal: &serde_json::Value,
    interval: Interval,
    flow_name: &str,
) -> Result<Literal, ErrorInfo> {
    match literal {
        serde_json::Value::String(val) => Ok(PrimitiveString::get_literal(val, interval)),
        serde_json::Value::Bool(val) => Ok(PrimitiveBoolean::get_literal(*val, interval)),
        serde_json::Value::Null => Ok(PrimitiveNull::get_literal(interval)),
        serde_json::Value::Number(val) => {
            if let (true, Some(float)) = (val.is_f64(), val.as_f64()) {
                Ok(PrimitiveFloat::get_literal(float, interval))
            } else if let (true, Some(int)) = (val.is_i64(), val.as_i64()) {
                Ok(PrimitiveInt::get_literal(int, interval))
            } else {
                Err(gen_error_info(
                    Position::new(interval, flow_name),
                    ERROR_JSON_TO_LITERAL.to_owned(),
                ))
            }
        }
        serde_json::Value::Array(val) => {
            let mut vec = vec![];

            for elem in val {
                vec.push(json_to_literal(elem, interval, flow_name)?);
            }

            Ok(PrimitiveArray::get_literal(vec, interval))
        }
        serde_json::Value::Object(val) => {
            let mut map = HashMap::new();

            for (k, v) in val {
                map.insert(k.clone(), json_to_literal(v, interval, flow_name)?);
            }

            Ok(PrimitiveObject::get_literal(map, interval))
        }
    }
}

pub fn memory_to_literal(
    literal: &serde_json::Value,
    interval: Interval,
    flow_name: &str,
) -> Result<Literal, ErrorInfo> {
    match literal {
        serde_json::Value::String(val) => Ok(PrimitiveString::get_literal(val, interval)),
        serde_json::Value::Bool(val) => Ok(PrimitiveBoolean::get_literal(*val, interval)),
        serde_json::Value::Null => Ok(PrimitiveNull::get_literal(interval)),
        serde_json::Value::Number(val) => {
            if let (true, Some(float)) = (val.is_f64(), val.as_f64()) {
                Ok(PrimitiveFloat::get_literal(float, interval))
            } else if let (true, Some(int)) = (val.is_i64(), val.as_i64()) {
                Ok(PrimitiveInt::get_literal(int, interval))
            } else {
                Err(gen_error_info(
                    Position::new(interval, flow_name),
                    ERROR_JSON_TO_LITERAL.to_owned(),
                ))
            }
        }
        serde_json::Value::Array(val) => {
            let mut vec = vec![];

            for elem in val {
                vec.push(memory_to_literal(elem, interval, flow_name)?);
            }

            Ok(PrimitiveArray::get_literal(vec, interval))
        }

        serde_json::Value::Object(map) if map.contains_key("_additional_info") => {
            if let (Some(value), Some(serde_json::Value::Object(additional_info))) =
                (map.get("value"), map.get("_additional_info"))
            {
                let mut literal = memory_to_literal(value, interval, flow_name)?;

                for (k, v) in additional_info {
                    literal.add_info(k, memory_to_literal(v, interval, flow_name)?);
                }

                Ok(literal)
            } else {
                Ok(PrimitiveNull::get_literal(interval))
            }
        }

        serde_json::Value::Object(map)
            if map.contains_key("_content") && map.contains_key("_content_type") =>
        {
            if let (Some(content), Some(serde_json::Value::String(conent_type))) =
                (map.get("_content"), map.get("_content_type"))
            {
                let mut literal = memory_to_literal(content, interval, flow_name)?;
                literal.set_content_type(conent_type);
                Ok(literal)
            } else {
                Ok(PrimitiveNull::get_literal(interval))
            }
        }

        serde_json::Value::Object(map) if map.contains_key("_closure") => {
            if let Some(closure_json) = map.get("_closure") {
                let closure: PrimitiveClosure = serde_json::from_value(closure_json.clone())?;

                Ok(Literal {
                    content_type: "closure".to_owned(),
                    primitive: Box::new(closure),
                    additional_info: None,
                    secure_variable: false,
                    interval,
                })
            } else {
                Ok(PrimitiveNull::get_literal(interval))
            }
        }

        serde_json::Value::Object(map) => {
            let mut obj = HashMap::new();

            for (k, v) in map {
                obj.insert(k.clone(), memory_to_literal(v, interval, flow_name)?);
            }
            Ok(PrimitiveObject::get_literal(obj, interval))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_literal() {
        let value = serde_json::json!({
            "array": [1, 2, 3],
            "object": {
                "key": "value",
                "nested": {
                    "nested_key": "nested_value"
                }
            }
        });
        let lit = json_to_literal(&value, Interval::default(), "flow").unwrap();
        let primitive = PrimitiveObject {
            value: HashMap::from_iter(vec![
                (
                    "array".to_string(),
                    PrimitiveArray::get_literal(
                        vec![
                            PrimitiveInt::get_literal(1, Interval::default()),
                            PrimitiveInt::get_literal(2, Interval::default()),
                            PrimitiveInt::get_literal(3, Interval::default()),
                        ],
                        Interval::default(),
                    ),
                ),
                (
                    "object".to_string(),
                    PrimitiveObject::get_literal(
                        HashMap::from_iter(vec![
                            (
                                "key".to_string(),
                                PrimitiveString::get_literal("value", Interval::default()),
                            ),
                            (
                                "nested".to_string(),
                                PrimitiveObject::get_literal(
                                    HashMap::from_iter(vec![(
                                        "nested_key".to_string(),
                                        PrimitiveString::get_literal(
                                            "nested_value",
                                            Interval::default(),
                                        ),
                                    )]),
                                    Interval::default(),
                                ),
                            ),
                        ]),
                        Interval::default(),
                    ),
                ),
            ]),
        };
        let expected_lit = Literal {
            content_type: "object".to_string(),
            primitive: Box::new(primitive),
            additional_info: None,
            secure_variable: false,
            interval: Interval::default(),
        };
        assert_eq!(lit, expected_lit);
    }
}
