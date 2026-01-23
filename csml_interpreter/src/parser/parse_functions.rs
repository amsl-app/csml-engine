use crate::parser::parse_idents::parse_idents_assignation;

use crate::data::{
    ast::{Block, BlockType, Expr, Instruction, InstructionScope},
    tokens::{COLON, Span},
};
use crate::error_format::{ERROR_FN_COLON, gen_nom_error};
use crate::parser::{
    parse_braces::parse_brace, parse_comments::comment, parse_scope::parse_root,
    parse_var_types::parse_fn_args, tools::get_interval,
};

use crate::data::tokens::{LBrace, RBrace, Token};
use nom::error::{ContextError, ParseError};
use nom::{
    Err, IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    sequence::{delimited, preceded},
};
////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn parse_function_scope_colon<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Block, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = match preceded(comment, tag(COLON)).parse(s) {
        Ok((s, colon)) if *colon.fragment() == COLON => (s, colon),
        Ok(_) => return Err(gen_nom_error(s, ERROR_FN_COLON)),

        Err(Err::Error((_s, _err)) | Err::Failure((_s, _err))) => {
            return Err(gen_nom_error(s, ERROR_FN_COLON));
        }
        Err(Err::Incomplete(needed)) => return Err(Err::Incomplete(needed)),
    };

    preceded(comment, parse_root).parse(s)
}

fn parse_function_scope<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Block, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    delimited(
        preceded(comment, tag(LBrace::TOKEN)),
        parse_root,
        preceded(comment, parse_brace(RBrace)),
    )
    .parse(s)
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn parse_function<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Vec<Instruction>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, mut interval) = preceded(comment, get_interval).parse(s)?;

    let (s, _) = preceded(comment, tag("fn")).parse(s)?;
    let (s, ident) = preceded(comment, parse_idents_assignation).parse(s)?;
    let (s, args) = parse_fn_args(s)?;

    let (s, scope) = alt((parse_function_scope_colon, parse_function_scope)).parse(s)?;

    let (s, end) = get_interval(s)?;

    interval.add_end(end);

    Ok((
        s,
        vec![Instruction {
            instruction_type: InstructionScope::FunctionScope {
                name: ident.ident,
                args,
            },
            actions: Expr::Scope {
                block_type: BlockType::Function,
                scope,
                range: interval,
            },
        }],
    ))
}
