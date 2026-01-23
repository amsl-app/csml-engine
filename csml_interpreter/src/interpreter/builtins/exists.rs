use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::{PrimitiveBoolean, PrimitiveType};
use crate::data::{
    ArgsType, Data, Literal,
    ast::{Identifier, Interval},
};
use crate::error_format::{ERROR_VAR_EXISTS, gen_error_info};
use crate::interpreter::variable_handler::memory::search_in_memory_type;

pub(crate) fn exists(
    mut args: ArgsType,
    data: &mut Data,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let Some(literal) = args.remove_typed("string", 0, PrimitiveType::PrimitiveString) else {
        return Err(gen_error_info(
            Position::new(interval, &data.context.flow),
            ERROR_VAR_EXISTS.to_owned(),
        ));
    };
    let value = literal.primitive.to_string();
    let ident = Identifier::new(&value, interval);

    let result = search_in_memory_type(&ident, data);

    Ok(PrimitiveBoolean::get_literal(result.is_ok(), interval))
}
