use crate::data::{ArgsType, Literal, ast::Interval};
use crate::error_format::{ERROR_CRYPTO, ErrorInfo};
use crate::interpreter::builtins::typed_object;

pub(crate) fn crypto(
    args: ArgsType,
    flow_name: &str,
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    typed_object(args, "value", "crypto", flow_name, interval, ERROR_CRYPTO)
}
