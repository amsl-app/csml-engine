use crate::data::primitive::object::PrimitiveObject;
use crate::data::{ArgsType, Literal, ast::Interval};
use crate::error_format::{ERROR_BASE64, ERROR_HEX, ErrorInfo};
use crate::interpreter::builtins;
use std::collections::HashMap;

pub(crate) fn debug(args: ArgsType, interval: Interval) -> Literal {
    args.args_to_debug(interval)
}

// TODO: old builtin need to be rm when no one use it
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn object(
    object: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    let mut map = HashMap::new();

    object.populate(&mut map, &[], flow_name, interval)?;

    Ok(PrimitiveObject::get_literal(map, interval))
}

pub(crate) fn base64(
    args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    builtins::typed_object(args, "string", "base64", flow_name, interval, ERROR_BASE64)
}

pub(crate) fn hex(
    args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    builtins::typed_object(args, "string", "hex", flow_name, interval, ERROR_HEX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::primitive::{PrimitiveBoolean, PrimitiveString};

    fn test_typed_object<F: Fn(ArgsType, &str, Interval) -> Result<Literal, ErrorInfo>>(
        content_type: &str,
        f: F,
    ) {
        let args = ArgsType::Normal(HashMap::from([(
            "arg0".to_owned(),
            PrimitiveString::get_literal("test", Interval::default()),
        )]));

        let result = f(args, "test-flow", Interval::default()).unwrap();
        let expected = PrimitiveObject::get_literal_with_type(
            content_type,
            HashMap::from([(
                "string".to_owned(),
                PrimitiveString::get_literal("test", Interval::default()),
            )]),
            Interval::default(),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_base64() {
        test_typed_object("base64", base64);
    }

    #[test]
    fn test_hex() {
        test_typed_object("hex", hex);
    }

    #[test]
    fn test_with_empty_args() {
        let args = ArgsType::Normal(HashMap::new());
        let result = base64(args, "test-flow", Interval::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_with_invalid_args() {
        let args = ArgsType::Normal(HashMap::from([(
            "arg0".to_owned(),
            PrimitiveBoolean::get_literal(true, Interval::default()),
        )]));

        let result = base64(args, "test-flow", Interval::default());
        assert!(result.is_err());
    }
}
