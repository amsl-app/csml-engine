use crate::data::tokens::Brace;
use crate::data::{
    ast::{Expr, InsertStep, Instruction, InstructionScope, Interval, ObjectType},
    primitive::PrimitiveNull,
    tokens::{FROM, INSERT, Span},
};
use crate::error_format::ERROR_INSERT_ARGUMENT;
use crate::parser::parse_group::parse_group;
use crate::parser::{
    get_interval, get_string, get_tag,
    parse_comments::comment,
    parse_idents::{parse_idents_as, parse_idents_assignation},
};
use nom::{
    Err, IResult, Parser,
    branch::alt,
    combinator::cut,
    error::{ContextError, ErrorKind, ParseError},
    sequence::preceded,
};
////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn parse_step_name<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, identifier) = parse_idents_assignation(s)?;

    parse_idents_as(s, Expr::IdentExpr(identifier))
}

fn parse_step_name_as_vec<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Vec<Expr>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, expr) = parse_step_name(s)?;

    Ok((s, vec![expr]))
}

fn parse_insert_params<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Vec<Expr>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    match alt((parse_group(Brace, parse_step_name), parse_step_name_as_vec)).parse(s) {
        Ok(value) => Ok(value),
        Err(Err::Error(e)) => Err(Err::Failure(E::add_context(s, ERROR_INSERT_ARGUMENT, e))),
        Err(Err::Failure(e)) => Err(Err::Failure(E::append(s, ErrorKind::Tag, e))),
        Err(Err::Incomplete(needed)) => Err(Err::Incomplete(needed)),
    }
}

fn parse_from<'a, E>(s: Span<'a>) -> IResult<Span<'a>, String, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, name) = preceded(comment, get_string).parse(s)?;
    let (s, ..) = cut(get_tag(name, FROM)).parse(s)?;
    let (s, name) = cut(preceded(comment, get_string)).parse(s)?;

    Ok((s, name))
}

fn parse_insert_prototype<'a, E>(s: Span<'a>) -> IResult<Span<'a>, (Interval, Vec<Expr>, String), E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, start) = preceded(comment, get_interval).parse(s)?;
    let (s, name) = preceded(comment, get_string).parse(s)?;

    let (s, ..) = get_tag(name, INSERT)(s)?;

    let (s, fn_names) = cut(preceded(comment, parse_insert_params)).parse(s)?;

    let (s, from_flow) = parse_from(s)?;

    Ok((s, (start, fn_names, from_flow)))
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTION
////////////////////////////////////////////////////////////////////////////////

pub fn parse_insert<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Vec<Instruction>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, (interval, fn_names, from_flow)) = parse_insert_prototype(s)?;

    let instructions = fn_names
        .iter()
        .map(|name| {
            let (name, original_name) = match name {
                Expr::IdentExpr(ident) => (ident.ident.clone(), None),
                Expr::ObjectExpr(ObjectType::As(name, expr)) => match &**expr {
                    Expr::IdentExpr(ident) => (name.ident.clone(), Some(ident.ident.clone())),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };

            Instruction {
                instruction_type: InstructionScope::InsertStep(InsertStep {
                    name,
                    original_name,
                    from_flow: from_flow.clone(),
                    interval,
                }),
                actions: Expr::LitExpr {
                    literal: PrimitiveNull::get_literal(interval),
                    in_in_substring: false,
                },
            }
        })
        .collect();

    Ok((s, instructions))
}
