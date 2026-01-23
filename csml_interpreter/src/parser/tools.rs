use crate::data::{ast::Interval, tokens::Span};
use nom::{
    Err, IResult, Input, Parser,
    bytes::complete::take_while1,
    error::{ContextError, ErrorKind, ParseError},
};

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTION
////////////////////////////////////////////////////////////////////////////////

fn position<'a, E, T>(s: T) -> IResult<T, T, E>
where
    T: Input,
    E: ParseError<T>,
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    nom::bytes::complete::take(0usize)(s)
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn get_interval<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Interval, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, pos) = position(s)?;
    Ok((s, Interval::new_as_span(pos)))
}

#[must_use]
pub fn get_range_interval(vector_interval: &[Interval]) -> Interval {
    let mut start = Interval::new_as_u32(0, 0, 0, None, None);
    let mut end = Interval::new_as_u32(0, 0, 0, None, None);

    for (index, interval) in vector_interval.iter().enumerate() {
        if index == 0 {
            start = *interval;
        }

        end = *interval;
    }

    start.add_end(end);
    start
}

// generate range error
pub fn parse_error<'a, O, E, F>(
    start: Span<'a>,
    span: Span<'a>,
    mut parser: F,
) -> IResult<Span<'a>, O, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
    F: Parser<Span<'a>, Output = O, Error = E>,
{
    match parser.parse(span) {
        Ok(value) => Ok(value),
        Err(Err::Error(e)) => Err(Err::Error(e)),
        Err(Err::Failure(e)) => Err(Err::Failure(E::append(start, ErrorKind::Tag, e))),
        Err(Err::Incomplete(needed)) => Err(Err::Incomplete(needed)),
    }
}

pub fn get_string<'a, E>(s: Span<'a>) -> IResult<Span<'a>, String, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (rest, string) =
        take_while1(|c: char| c == '-' || c == '_' || c == '\\' || c.is_alphanumeric())(s)?;

    Ok((rest, (*string.fragment()).to_string()))
}

pub fn get_tag<I, E: ParseError<I>>(
    var: String,
    tag: &str,
) -> impl FnMut(I) -> IResult<I, (), E> + '_ {
    move |input: I| {
        if var == tag {
            Ok((input, ()))
        } else {
            Err(Err::Error(E::from_error_kind(input, ErrorKind::Tag)))
        }
    }
}
