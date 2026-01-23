use crate::data::tokens::{LParen, RParen, Token};
use crate::data::{
    ast::Expr,
    tokens::{Span, WHILE},
};
use crate::parser::operator::parse_operator;
use crate::parser::{parse_comments::comment, parse_scope::parse_scope, tools::get_interval};
use nom::{
    IResult, Parser,
    bytes::complete::tag,
    combinator::cut,
    error::{ContextError, ParseError},
    sequence::preceded,
};
////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTION
////////////////////////////////////////////////////////////////////////////////

pub fn parse_while<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = preceded(comment, tag(WHILE)).parse(s)?;
    let (s, mut interval) = get_interval(s)?;

    let (s, _) = cut(preceded(comment, tag(LParen::TOKEN))).parse(s)?;
    let (s, expr) = cut(parse_operator).parse(s)?;
    let (s, _) = cut(preceded(comment, tag(RParen::TOKEN))).parse(s)?;

    let (s, block) = parse_scope(s)?;
    let (s, end) = get_interval(s)?;
    interval.add_end(end);

    Ok((s, Expr::WhileExpr(Box::new(expr), block, interval)))
}
