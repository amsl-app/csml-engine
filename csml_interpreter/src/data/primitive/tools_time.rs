use crate::data::{
    Literal,
    ast::Interval,
    error_info::ErrorInfo,
    position::Position,
    primitive::PrimitiveType,
    primitive::{Data, PrimitiveInt, PrimitiveNull, PrimitiveObject},
};
use crate::error_format::gen_error_info;
use chrono::{DateTime, NaiveDate, NaiveDateTime, SecondsFormat, TimeZone, Utc};
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn get_date_string<S: BuildHasher, E: Into<Cow<'static, str>>>(
    args: &HashMap<String, Literal, S>,
    index: usize,
    data: &mut Data,
    interval: Interval,
    error: E,
) -> Result<String, ErrorInfo> {
    match args.get(&format!("arg{index}")) {
        Some(literal) if literal.primitive.get_type() == PrimitiveType::PrimitiveString => {
            let value = Literal::get_value::<String, _>(
                &literal.primitive,
                &data.context.flow,
                literal.interval,
                error,
            )?;

            Ok(value.clone())
        }
        _ => Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            error.into().into_owned(),
        )),
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[must_use]
pub fn get_date<S: BuildHasher>(args: &HashMap<String, Literal, S>) -> [i64; 7] {
    let mut date: [i64; 7] = [0; 7];

    // set default month, day, and hour to 1 year does not need to have a default
    // value because set_date_at expect at least the year value as parameter
    date[1] = 1; // month
    date[2] = 1; // day
    date[3] = 1; // hour

    let len = args.len();

    for (index, item) in date.iter_mut().enumerate().take(len) {
        let Some(lit) = args.get(&format!("arg{index}")) else {
            continue;
        };
        if lit.primitive.get_type() == PrimitiveType::PrimitiveInt {
            let value = serde_json::from_str(&lit.primitive.to_string()).unwrap();

            *item = value;
        }
    }

    date
}

pub fn parse_rfc3339<S: BuildHasher>(
    args: &HashMap<String, Literal, S>,
    data: &mut Data,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let usage = "invalid value, 'parse(String)' expect a valid RFC 3339 and ISO 8601 date and time string such as '1996-12-19T00:00:00Z'";

    let date_str = get_date_string(args, 0, data, interval, usage)?;

    // autocomplete format with default values
    let date_str = match date_str.len() {
        4 => format!(
            "{}-{a1}-{a1}T{a2}:{a2}:{a2}Z",
            date_str,
            a1 = "01",
            a2 = "00"
        ),
        7 => format!("{}-{a1}T{a2}:{a2}:{a2}Z", date_str, a1 = "01", a2 = "00"),
        10 => format!("{}T{a2}:{a2}:{a2}Z", date_str, a2 = "00"),
        13 => format!("{}:{a2}:{a2}Z", date_str, a2 = "00"),
        16 => format!("{}:{a2}Z", date_str, a2 = "00"),
        19 => format!("{date_str}Z"),
        _ => date_str.clone(),
    };

    let date = DateTime::parse_from_rfc3339(&date_str).map_err(|_| {
        gen_error_info(
            Position::new(interval, &data.context.flow),
            usage.to_string(),
        )
    })?;

    let mut object = HashMap::new();

    object.insert(
        "milliseconds".to_owned(),
        PrimitiveInt::get_literal(date.to_utc().timestamp_millis(), interval),
    );

    let offset: i32 = date.timezone().local_minus_utc();
    if offset != 0 {
        object.insert(
            "offset".to_owned(),
            PrimitiveInt::get_literal(i64::from(offset), interval),
        );
    }

    let mut lit = PrimitiveObject::get_literal(object, interval);
    lit.set_content_type("time");

    Ok(lit)
}

pub fn pasre_from_str<S: BuildHasher>(
    args: &HashMap<String, Literal, S>,
    data: &mut Data,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let usage = "invalid value,
    expect date and a specified format to parse the date example:
    parse(\"1983 08 13 12:09:14.274\", \"%Y %m %d %H:%M:%S%.3f\")";

    let date_str = get_date_string(args, 0, data, interval, usage)?;

    let format_str = get_date_string(args, 1, data, interval, usage)?;

    let date_millis = if let Ok(date) = DateTime::parse_from_str(&date_str, &format_str) {
        date.to_utc().timestamp_millis()
    } else if let Ok(naive_datetime) = NaiveDateTime::parse_from_str(&date_str, &format_str) {
        let date = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
        date.to_utc().timestamp_millis()
    } else if let Ok(naive_date) = NaiveDate::parse_from_str(&date_str, &format_str) {
        let naive_datetime: NaiveDateTime = naive_date.and_hms_opt(0, 0, 0).unwrap();
        let date = DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc);
        date.to_utc().timestamp_millis()
    } else {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            usage.to_string(),
        ));
    };

    let mut object = HashMap::new();
    object.insert(
        "milliseconds".to_owned(),
        PrimitiveInt::get_literal(date_millis, interval),
    );

    let mut lit = PrimitiveObject::get_literal(object, interval);
    lit.set_content_type("time");

    Ok(lit)
}

pub fn format_date<Tz, S: BuildHasher>(
    args: &HashMap<String, Literal, S>,
    date: &DateTime<Tz>,
    data: &mut Data,
    interval: Interval,
    use_z: bool,
) -> Result<String, ErrorInfo>
where
    Tz: TimeZone,
    Tz::Offset: core::fmt::Display,
{
    if args.is_empty() {
        return Ok(date.to_rfc3339_opts(SecondsFormat::Millis, use_z));
    }

    let format_lit = args
        .get("arg0")
        .cloned()
        .unwrap_or_else(|| PrimitiveNull::get_literal(Interval::default()));

    let format = Literal::get_value::<String, _>(
        &format_lit.primitive,
        &data.context.flow,
        interval,
        "format parameter must be of type string",
    )?;

    Ok(date.format(format).to_string())
}
