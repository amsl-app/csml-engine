use crate::data::primitive::{PrimitiveInt, PrimitiveObject};
use crate::data::{Literal, ast::Interval};
use chrono::Utc;
use std::collections::HashMap;

pub(crate) fn time(interval: Interval) -> Literal {
    let date = Utc::now();

    let time: HashMap<String, Literal> = HashMap::from([(
        "milliseconds".to_owned(),
        PrimitiveInt::get_literal(date.timestamp_millis(), interval),
    )]);

    let mut result = PrimitiveObject::get_literal(time, interval);

    result.set_content_type("time");

    result
}
