use crate::data::position::Position;
use crate::data::primitive::{PrimitiveObject, PrimitiveString};
use std::collections::HashMap;

use crate::data::{ArgsType, Literal, ast::Interval};
use crate::error_format::{ERROR_JWT, ErrorInfo, gen_error_info};

pub(crate) fn jwt(
    mut args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let Some(jwt) = args.remove("jwt", 0) else {
        return Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_JWT.to_owned(),
        ));
    };

    let header = HashMap::from([
        (
            "typ".to_owned(),
            PrimitiveString::get_literal("JWT", interval),
        ),
        (
            "alg".to_owned(),
            PrimitiveString::get_literal("HS256", interval),
        ),
    ]);

    let lit_header = PrimitiveObject::get_literal(header, interval);

    let jwt_map: HashMap<String, Literal> =
        HashMap::from([("jwt".to_owned(), jwt), ("header".to_owned(), lit_header)]);

    let result = PrimitiveObject::get_literal_with_type("jwt", jwt_map, interval);

    Ok(result)
}
