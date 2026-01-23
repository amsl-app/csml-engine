use crate::data::{
    ast::{AssignType, Expr, ObjectType},
    primitive::PrimitiveInt,
    tokens::{ASSIGN, Span},
};
use crate::parser::{
    operator::{parse_operator, tools::parse_item_operator},
    parse_built_in::parse_built_in,
    parse_closure::parse_closure,
    parse_comments::comment,
    parse_idents::{parse_arg_idents_assignation, parse_idents_as, parse_idents_usage},
    parse_literal::parse_literal_expr,
    parse_object::parse_object,
    parse_path::parse_path,
    parse_string::parse_string,
    tools::{get_interval, get_string, parse_error},
};

use crate::data::tokens::{Bracket, LParen, Paren, RParen, Token};
use crate::parser::parse_braces::parse_brace;
use crate::parser::parse_group::parse_group;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    combinator::opt,
    error::{ContextError, ParseError},
    sequence::{delimited, preceded},
};
////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn parse_condition_group<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, interval) = get_interval(s)?;

    let (s, opt) = opt(preceded(comment, parse_item_operator)).parse(s)?;
    let (s, expr) = delimited(
        preceded(comment, tag(LParen::TOKEN)),
        parse_operator,
        parse_brace(RParen),
    )
    .parse(s)?;

    match opt {
        Some(infix) => {
            let zero = Expr::LitExpr {
                literal: PrimitiveInt::get_literal(0, interval),
                in_in_substring: false,
            };
            Ok((s, Expr::InfixExpr(infix, Box::new(zero), Box::new(expr))))
        }
        None => Ok((s, expr)),
    }
}

fn parse_assignation_without_path<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, name) = parse_arg_idents_assignation(s)?;
    let (s, _) = preceded(comment, tag(ASSIGN)).parse(s)?;
    let (s, expr) = preceded(comment, parse_operator).parse(s)?;

    Ok((
        s,
        Expr::ObjectExpr(ObjectType::Assign(
            AssignType::Assignment,
            Box::new(Expr::IdentExpr(name)),
            Box::new(expr),
        )),
    ))
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn parse_idents_expr_usage<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, idents) = parse_idents_usage(s)?;

    Ok((s, Expr::IdentExpr(idents)))
}

pub fn parse_fn_args<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Vec<String>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (start, _) = preceded(comment, get_interval).parse(s)?;
    let (s, vec) = parse_error(start, s, parse_group(Paren, preceded(comment, get_string)))?;

    Ok((s, vec))
}

pub fn parse_expr_list<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (start, mut interval) = preceded(comment, get_interval).parse(s)?;
    let (s, vec) = parse_error(
        start,
        s,
        parse_group(Paren, alt((parse_assignation_without_path, parse_operator))),
    )?;
    let (s, end) = get_interval(s)?;
    interval.add_end(end);

    Ok((s, Expr::VecExpr(vec, interval)))
}

pub fn parse_expr_array<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (start, mut interval) = preceded(comment, get_interval).parse(s)?;

    let (s, vec) = parse_error(start, s, parse_group(Bracket, parse_operator))?;
    let (s, end) = get_interval(s)?;
    interval.add_end(end);

    Ok((s, Expr::VecExpr(vec, interval)))
}

pub fn parse_basic_expr<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = comment(s)?;

    let (s, expr) = alt((
        parse_closure,
        parse_condition_group,
        parse_object,
        parse_expr_array,
        parse_literal_expr,
        parse_built_in,
        parse_string,
        parse_idents_expr_usage,
    ))
    .parse(s)?;

    let (s, expr) = parse_path(s, expr)?;

    let (s, expr) = parse_idents_as(s, expr)?;

    let (s, _) = comment(s)?;
    Ok((s, expr))
}
