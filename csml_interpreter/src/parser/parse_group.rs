use crate::data::tokens::{Braces, COMMA, Span, Token};
use crate::parser::parse_comments::comment;
use nom::Parser;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::error::{ContextError, ParseError};
use nom::multi::separated_list0;
use nom::sequence::{preceded, terminated};

pub fn parse_group<'a, B: Braces, E, T, P: Parser<Span<'a>, Output = T, Error = E>>(
    _kind: B,
    mut inner: P,
) -> impl Parser<Span<'a>, Output = Vec<T>, Error = E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    move |input: Span<'a>| {
        let (input, (vec, ..)) = preceded(
            tag(B::Left::TOKEN),
            terminated(
                (
                    separated_list0(preceded(comment, tag(COMMA)), |input| inner.parse(input)),
                    opt(preceded(comment, tag(COMMA))),
                ),
                preceded(comment, tag(B::Right::TOKEN)),
            ),
        )
        .parse(input)?;

        Ok((input, vec))
    }
}
