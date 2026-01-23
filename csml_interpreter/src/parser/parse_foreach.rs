use crate::data::tokens::{LParen, RParen, Token};
use crate::data::{
    ast::{Expr, Identifier},
    tokens::{COMMA, FOREACH, IN, Span},
};
use crate::parser::operator::parse_operator;
use crate::parser::parse_idents::parse_idents_assignation;
use crate::parser::{
    parse_comments::comment,
    parse_scope::parse_scope,
    tools::{get_interval, get_string, get_tag},
};
use nom::{
    IResult, Parser,
    bytes::complete::tag,
    combinator::{cut, opt},
    error::{ContextError, ParseError},
    sequence::preceded,
};
////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTION
////////////////////////////////////////////////////////////////////////////////

fn pars_args<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Identifier, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = preceded(comment, tag(COMMA)).parse(s)?;
    let (s, idents) = parse_idents_assignation(s)?;

    Ok((s, idents))
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTION
////////////////////////////////////////////////////////////////////////////////

pub fn parse_foreach<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = preceded(comment, tag(FOREACH)).parse(s)?;
    let (s, mut interval) = get_interval(s)?;

    let (s, _) = cut(preceded(comment, tag(LParen::TOKEN))).parse(s)?;
    let (s, idents) = cut(parse_idents_assignation).parse(s)?;
    let (s, opt) = opt(pars_args).parse(s)?;
    let (s, _) = cut(preceded(comment, tag(RParen::TOKEN))).parse(s)?;

    let (s, value) = cut(preceded(comment, get_string)).parse(s)?;
    let (s, ..) = cut(get_tag(value, IN)).parse(s)?;

    let (s, expr) = cut(parse_operator).parse(s)?;

    let (s, block) = parse_scope(s)?;
    let (s, end) = get_interval(s)?;
    interval.add_end(end);

    Ok((
        s,
        Expr::ForEachExpr(idents, opt, Box::new(expr), block, interval),
    ))
}
