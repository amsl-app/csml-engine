use crate::data::tokens::{LBrace, Paren, RBrace, Token};
use crate::data::{
    ast::{BlockType, Expr},
    primitive::closure::PrimitiveClosure,
    tokens::Span,
};
use crate::parser::parse_group::parse_group;
use crate::parser::{
    parse_braces::parse_brace,
    parse_comments::comment,
    parse_scope::parse_root,
    tools::{get_interval, get_string},
};
use nom::{
    IResult, Parser,
    bytes::complete::tag,
    error::{ContextError, ParseError},
    sequence::preceded,
};

fn parse_closure_args<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Vec<String>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, vec) = parse_group(Paren, preceded(comment, get_string)).parse(s)?;

    Ok((s, vec))
}

pub fn parse_closure<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, mut interval) = preceded(comment, get_interval).parse(s)?;
    let (s, args) = parse_closure_args(s)?;

    let (s, _) = preceded(comment, tag(LBrace::TOKEN)).parse(s)?;

    let result = preceded(comment, parse_root).parse(s);
    let (s, func) = result?;

    let (s, _) = preceded(comment, parse_brace(RBrace)).parse(s)?;

    let (s, end) = get_interval(s)?;
    interval.add_end(end);

    let closure = Expr::LitExpr {
        literal: PrimitiveClosure::get_literal(
            args,
            Box::new(Expr::Scope {
                block_type: BlockType::Function,
                scope: func,
                range: interval,
            }),
            interval,
            None,
        ),
        in_in_substring: false,
    };

    Ok((s, closure))
}
