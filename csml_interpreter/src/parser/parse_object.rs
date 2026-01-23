use crate::data::{
    ast::Expr,
    tokens::{BACKSLASH_DOUBLE_QUOTE, COLON, COMMA, DOUBLE_QUOTE, Span},
};
use crate::parser::{parse_comments::comment, tools::get_interval};

use crate::data::tokens::{LBrace, RBrace, Token};
use crate::parser::operator::parse_operator;
use nom::{
    IResult, Parser,
    bytes::complete::tag,
    bytes::complete::take_till1,
    combinator::{cut, map, opt},
    error::{ContextError, ParseError, context},
    multi::separated_list0,
    sequence::{preceded, separated_pair, terminated},
};
use std::collections::HashMap;
////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn string<'a, E>(s: Span<'a>) -> IResult<Span<'a>, (Span<'a>, bool), E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let token = match (
        tag(DOUBLE_QUOTE)(s) as IResult<Span<'a>, Span<'a>, E>,
        tag(BACKSLASH_DOUBLE_QUOTE)(s) as IResult<Span<'a>, Span<'a>, E>,
    ) {
        (Ok(_), ..) => DOUBLE_QUOTE,
        (.., Ok(_)) => BACKSLASH_DOUBLE_QUOTE,
        (Err(err), ..) => return Err(err), // set error to failure
    };

    let (s, key) = context(
        "string must start with '\"' ",
        preceded(
            tag(token),
            cut(terminated(
                take_till1(|c: char| token.contains(c)),
                tag(token),
            )),
        ),
    )
    .parse(s)?;

    // the is in sub string param is used to determine if the key string was declared inside a string or not
    let is_sub_string = matches!(token, BACKSLASH_DOUBLE_QUOTE);

    Ok((s, (key, is_sub_string)))
}

#[allow(clippy::type_complexity)]
fn parse_arguments<'a, E>(
    s: Span<'a>,
) -> IResult<Span<'a>, (Vec<((Span<'a>, bool), Expr)>, bool), E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, result) = separated_list0(
        preceded(comment, tag(COMMA)),
        separated_pair(
            preceded(comment, string),
            cut(preceded(comment, tag(COLON))),
            parse_operator,
        ),
    )
    .parse(s)?;

    Ok((s, (result, false)))
}

fn key_value<'a, E>(s: Span<'a>) -> IResult<Span<'a>, (HashMap<String, Expr>, bool), E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    map(parse_arguments, |(tuple_vec, mut is_in_sub_string)| {
        let args_map = tuple_vec
            .into_iter()
            .map(|((key, token_type), value)| {
                if token_type {
                    is_in_sub_string = true;
                }

                (String::from(*key.fragment()), value)
            })
            .collect();

        (args_map, is_in_sub_string)
    })
    .parse(s)
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTION
////////////////////////////////////////////////////////////////////////////////

pub fn parse_object<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, mut interval) = preceded(comment, get_interval).parse(s)?;
    // the 'is_in_sub_string' param is use to determine if this object was declare inside a string or not
    let (s, ((object, is_in_sub_string), _trailing_comma)) = preceded(
        tag(LBrace::TOKEN),
        terminated(
            (key_value, opt(preceded(comment, tag(COMMA)))),
            preceded(comment, tag(RBrace::TOKEN)),
        ),
    )
    .parse(s)?;

    let (s, end) = preceded(comment, get_interval).parse(s)?;
    interval.add_end(end);

    Ok((
        s,
        Expr::MapExpr {
            object,
            is_in_sub_string,
            interval,
        },
    ))
}
