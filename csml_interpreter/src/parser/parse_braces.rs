use crate::data::tokens::{Span, Token};
use crate::error_format::{add_context, escalate_nom_error};
use nom::{
    Parser,
    bytes::complete::tag,
    error::{ContextError, ParseError},
};
////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn parse_brace<'a, T: Token, E>(_kind: T) -> impl Parser<Span<'a>, Output = Span<'a>, Error = E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    |s: Span<'a>| {
        tag(T::TOKEN)(s).map_err(|error| {
            escalate_nom_error(error, |(input, _)| add_context(input, T::MISSING_ERROR))
        })
    }
}
