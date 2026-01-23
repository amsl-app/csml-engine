use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::PrimitiveBoolean;
use crate::data::primitive::{PrimitiveInt, PrimitiveType, object::PrimitiveObject};
use crate::data::{ArgsType, Literal, ast::Interval};
use crate::error_format::{ERROR_SMTP, gen_error_info};
use std::collections::HashMap;

pub(crate) fn smtp(
    mut args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let Some(server) = args.remove_typed("smtp_server", 0, PrimitiveType::PrimitiveString) else {
        return Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_SMTP.to_owned(),
        ));
    };

    let map: HashMap<String, Literal> = HashMap::from([
        ("smtp_server".to_owned(), server),
        // set default port to [465] for TLS connections [RFC8314](https://tools.ietf.org/html/rfc8314)
        ("port".to_owned(), PrimitiveInt::get_literal(465, interval)),
        (
            "tls".to_owned(),
            PrimitiveBoolean::get_literal(true, interval),
        ),
    ]);

    let result = PrimitiveObject::get_literal_with_type("smtp", map, interval);

    Ok(result)
}
