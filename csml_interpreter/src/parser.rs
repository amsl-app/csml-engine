pub(crate) mod operator;
pub(crate) mod parse_actions;
pub(crate) mod parse_braces;
pub(crate) mod parse_built_in;
pub(crate) mod parse_closure;
pub(crate) mod parse_comments;
pub(crate) mod parse_constant;
pub(crate) mod parse_foreach;
pub(crate) mod parse_functions;
pub(crate) mod parse_goto;
pub(crate) mod parse_group;
pub(crate) mod parse_idents;
pub(crate) mod parse_if;
pub(crate) mod parse_import;
pub(crate) mod parse_insert;
pub(crate) mod parse_literal;
pub(crate) mod parse_object;
pub(crate) mod parse_path;
pub(crate) mod parse_previous;
pub(crate) mod parse_scope;
pub(crate) mod parse_string;
pub(crate) mod parse_var_types;
pub(crate) mod parse_while_loop;
pub(crate) mod state_context;
pub(crate) mod step_checksum;
pub(crate) mod tools;

use crate::parser::parse_idents::parse_idents_assignation;
pub(crate) use state_context::ExitCondition;

use crate::data::position::Position;
use crate::data::{
    ast::{
        BlockType, Expr, Flow, FlowType, GotoType, GotoValueType, Identifier, Instruction,
        InstructionScope, Interval,
    },
    tokens::{COLON, Span},
};
use crate::error_format::{
    CustomError, ERROR_PARSING, ErrorInfo, convert_error_from_span, gen_error_info, gen_nom_error,
    gen_nom_failure,
};
use crate::interpreter::variable_handler::interval::interval_from_expr;
use parse_comments::comment;
use parse_constant::{constant_expr_to_lit, parse_constant};
use parse_functions::parse_function;
use parse_import::parse_import;
use parse_insert::parse_insert;
use parse_scope::parse_root;
use tools::{get_interval, get_string, get_tag};

use nom::error::{ContextError, ParseError};
use nom::{
    Err, IResult, Parser, branch::alt, bytes::complete::tag, multi::fold_many0, sequence::preceded,
};
use std::collections::HashMap;

pub(crate) fn parse_step_name<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Identifier, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    // this will save the location of the keyword to display the error correctly
    let (command_span, _) = comment(s)?;

    let (s2, ident) = match parse_idents_assignation(command_span) {
        Ok((s2, ident)) => (s2, ident),
        Err(Err::Error((s, _err)) | Err::Failure((s, _err))) => {
            return if s.fragment().is_empty() {
                Err(gen_nom_error(s, ERROR_PARSING))
            } else {
                Err(gen_nom_failure(s, ERROR_PARSING))
            };
        }
        Err(Err::Incomplete(needed)) => return Err(Err::Incomplete(needed)),
    };

    match tag(COLON)(s2) {
        Ok((rest, _)) => Ok((rest, ident)),
        Err(Err::Error((_, _err)) | Err::Failure((_, _err))) => {
            Err(gen_nom_failure(command_span, ERROR_PARSING))
        }
        Err(Err::Incomplete(needed)) => Err(Err::Incomplete(needed)),
    }
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn parse_flow<'a>(slice: &'a str, flow_name: &'a str) -> Result<Flow, ErrorInfo> {
    match start_parsing::<CustomError<Span<'a>>>(Span::new(slice)) {
        Ok((_, (instructions, flow_type))) => {
            let mut flow_instructions = HashMap::new();
            let mut constants = HashMap::new();
            // let mut inserts = vec![];

            for instruction in instructions {
                if let Instruction {
                    instruction_type: InstructionScope::Constant(name),
                    actions: expr,
                } = instruction
                {
                    let lit = constant_expr_to_lit(&expr, flow_name)?;

                    constants.insert(name, lit);
                } else {
                    let instruction_interval = interval_from_expr(&instruction.actions);
                    let instruction_info = instruction.instruction_type.get_info();

                    if let Some(old_instruction) =
                        flow_instructions.insert(instruction.instruction_type, instruction.actions)
                    {
                        // This is done to store all duplicated instructions during parsing
                        // and use by the linter to display them all as errors
                        flow_instructions.insert(
                            InstructionScope::DuplicateInstruction(
                                instruction_interval,
                                instruction_info,
                            ),
                            old_instruction,
                        );
                    }
                }
            }

            Ok(Flow {
                flow_instructions,
                flow_type,
                constants,
            })
        }
        Err(e) => match e {
            Err::Error(err) | Err::Failure(err) => {
                let (end_line, end_column) = match err.end {
                    Some(end) => (
                        Some(end.location_line()),
                        Some(u32::try_from(end.get_column())?),
                    ),
                    None => (None, None),
                };

                Err(gen_error_info(
                    Position::new(
                        Interval::new_as_u32(
                            err.input.location_line(),
                            u32::try_from(err.input.get_column())?,
                            err.input.location_offset(),
                            end_line,
                            end_column,
                        ),
                        flow_name,
                    ),
                    convert_error_from_span(&Span::new(slice), &err),
                ))
            }
            Err::Incomplete(_err) => unreachable!(),
        },
    }
}

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTION
////////////////////////////////////////////////////////////////////////////////

fn parse_step<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Vec<Instruction>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, mut interval) = preceded(comment, get_interval).parse(s)?;
    let (s, ident) = parse_step_name(s)?;

    let (s, actions) = preceded(comment, parse_root).parse(s)?;
    let (s, end) = get_interval(s)?;
    interval.add_end(end);

    Ok((
        s,
        vec![Instruction {
            instruction_type: InstructionScope::StepScope(ident.ident),
            actions: Expr::Scope {
                block_type: BlockType::Step,
                scope: actions,
                range: interval,
            },
        }],
    ))
}

fn start_parsing<'a, E>(s: Span<'a>) -> IResult<Span<'a>, (Vec<Instruction>, FlowType), E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let flow_type = FlowType::Normal;

    let (s, flow) = fold_many0(
        alt((
            parse_constant,
            parse_import,
            parse_insert,
            parse_function,
            parse_step,
        )),
        Vec::new,
        |mut acc, mut item| {
            acc.append(&mut item);
            acc
        },
    )
    .parse(s)?;

    let (last, _) = comment(s)?;
    if last.fragment().is_empty() {
        Ok((s, (flow, flow_type)))
    } else {
        Err(gen_nom_failure(last, ERROR_PARSING))
    }
}
