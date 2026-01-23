use crate::data::tokens::Brace;
use crate::data::{
    ast::{Expr, FromFlow, ImportScope, Instruction, InstructionScope, Interval, ObjectType},
    primitive::PrimitiveNull,
    tokens::{FROM, IMPORT, Span},
};
use crate::error_format::ERROR_IMPORT_ARGUMENT;
use crate::parser::parse_group::parse_group;
use crate::parser::{
    get_interval, get_string, get_tag,
    parse_comments::comment,
    parse_idents::{parse_idents_as, parse_idents_assignation},
};
use nom::{
    Err, IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    combinator::opt,
    error::{ContextError, ErrorKind, ParseError},
    sequence::preceded,
};
////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn parse_fn_name<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, identifier) = parse_idents_assignation(s)?;

    parse_idents_as(s, Expr::IdentExpr(identifier))
}

fn parse_import_params<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Vec<Expr>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    match alt((
        parse_group(Brace, parse_fn_name),
        parse_fn_name.map(|name| vec![name]),
    ))
    .parse(s)
    {
        Ok(value) => Ok(value),
        Err(Err::Error(e)) => Err(Err::Failure(E::add_context(s, ERROR_IMPORT_ARGUMENT, e))),
        Err(Err::Failure(e)) => Err(Err::Failure(E::append(s, ErrorKind::Tag, e))),
        Err(Err::Incomplete(needed)) => Err(Err::Incomplete(needed)),
    }
}

fn parse_from<'a, E>(s: Span<'a>) -> IResult<Span<'a>, FromFlow, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, name) = preceded(comment, get_string).parse(s)?;
    let (s, ..) = get_tag(name, FROM)(s)?;
    let (s, name) = preceded(comment, get_string).parse(s)?;

    Ok((s, FromFlow::Normal(name)))
}

fn parse_from_extern_module<'a, E>(s: Span<'a>) -> IResult<Span<'a>, FromFlow, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, name) = preceded(comment, get_string).parse(s)?;
    let (s, ..) = get_tag(name, FROM)(s)?;

    let (s, name) = preceded(comment, preceded(tag("modules/"), get_string)).parse(s)?;

    Ok((s, FromFlow::Extern(name)))
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTION
////////////////////////////////////////////////////////////////////////////////

pub fn parse_import_prototype<'a, E>(
    s: Span<'a>,
) -> IResult<Span<'a>, (Interval, Vec<Expr>, FromFlow), E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, start) = preceded(comment, get_interval).parse(s)?;
    let (s, name) = preceded(comment, get_string).parse(s)?;

    let (s, ..) = get_tag(name, IMPORT)(s)?;

    let (s, fn_names) = preceded(comment, parse_import_params).parse(s)?;

    let (s, from_flow) = match opt(alt((parse_from_extern_module, parse_from))).parse(s)? {
        (s, Some(from_flow)) => (s, from_flow),
        (s, None) => (s, FromFlow::None),
    };

    Ok((s, (start, fn_names, from_flow)))
}

pub fn parse_import<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Vec<Instruction>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, (interval, fn_names, from_flow)) = parse_import_prototype(s)?;

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
                instruction_type: InstructionScope::ImportScope(ImportScope {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ast::Identifier;

    #[test]
    fn test_parse_import_params() {
        let input = Span::new("fn_name");
        let (_, out) = parse_import_params::<()>(input).unwrap();
        let out: &[_] = &out;
        assert!(matches!(
            out, [
                Expr::IdentExpr(
                    Identifier { ident, .. }
                )
            ] if ident == "fn_name"
        ));

        let (_, out) = parse_import_params::<()>(Span::new("{fn_name, fn_name2}")).unwrap();
        let out: &[_] = &out;
        assert!(matches!(
            out, [
                Expr::IdentExpr(
                    Identifier { ident, .. }
                ),
                Expr::IdentExpr(
                    Identifier { ident: ident2, .. }
                )
            ] if ident == "fn_name" && ident2 == "fn_name2"
        ));
    }
}
