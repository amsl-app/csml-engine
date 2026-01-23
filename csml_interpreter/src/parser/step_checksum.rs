use crate::data::{
    ast::{Flow, InstructionScope},
    tokens::Span,
};
use crate::error_format::CustomError;
use crate::interpreter::variable_handler::interval::interval_from_expr;
use crate::parser::parse_comments::comment;

use nom::error::ErrorKind;
use nom::{
    Err, IResult, Input, Parser,
    error::{ContextError, ParseError},
    multi::fold_many0,
    sequence::preceded,
};
////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn get_text<'a, E>(s: Span<'a>) -> IResult<Span<'a>, &'a str, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (rest, string) = s.split_at_position1_complete(
        |c| c == ' ' || c == '\t' || c == '\r' || c == '\n',
        ErrorKind::AlphaNumeric,
    )?;

    Ok((rest, string.fragment()))
}

fn clean_text<'a, E>(s: Span<'a>) -> IResult<Span<'a>, String, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (span, res) = fold_many0(preceded(comment, get_text), String::new, |mut acc, item| {
        acc.push_str(item);
        acc
    })
    .parse(s)?;

    Ok((span, res))
}

fn get_step_offset(
    name: &str,
    offsets: &[(String, usize)],
) -> ((String, usize), Option<(String, usize)>) {
    for (index, (step_name, offset)) in offsets.iter().enumerate() {
        if step_name == name {
            return (
                (step_name.clone(), *offset),
                offsets.get(index + 1).cloned(),
            );
        }
    }

    unreachable!()
}

fn get_skip_offset(skip_offsets: &[usize], offset: usize) -> Option<usize> {
    skip_offsets.iter().copied().find(|&x| x > offset)
}

fn get_next_offset(
    offset: usize,
    next_step: Option<&(String, usize)>,
    skip_offsets: &[usize],
) -> Option<usize> {
    let Some((_, next_step_offset)) = next_step else {
        return get_skip_offset(skip_offsets, offset);
    };
    let next_step_offset = *next_step_offset;
    if let Some(skip_offset) = get_skip_offset(skip_offsets, offset)
        && skip_offset > next_step_offset
    {
        return Some(skip_offset);
    }
    Some(next_step_offset)
}

fn get_offsets(ast: &Flow) -> (Vec<(String, usize)>, Vec<usize>) {
    let mut offsets = vec![];
    let mut skip_offsets = vec![];

    for (instruction_type, block) in &ast.flow_instructions {
        match instruction_type {
            InstructionScope::StepScope(name) | InstructionScope::Constant(name) => {
                let interval = interval_from_expr(block);
                offsets.push((name.clone(), interval.offset));
            }
            InstructionScope::FunctionScope { .. }
            | InstructionScope::ImportScope(_)
            | InstructionScope::InsertStep(_) => {
                let interval = interval_from_expr(block);
                skip_offsets.push(interval.offset);
            }
            InstructionScope::DuplicateInstruction(..) => {}
        }
    }
    offsets.sort_by(|(_, a), (_, b)| a.cmp(b));
    skip_offsets.sort_unstable();

    (offsets, skip_offsets)
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[must_use]
pub fn get_step<'a>(step_name: &'a str, flow: &'a str, ast: &'a Flow) -> String {
    let (offsets, skip_offsets) = get_offsets(ast);
    let span = Span::new(flow);

    let ((_, offset), next_step) = get_step_offset(step_name, &offsets);
    let (new, _) = span.take_split(offset);
    match get_next_offset(offset, next_step.as_ref(), &skip_offsets) {
        Some(skip_offset) => {
            let (_, old) = new.take_split(skip_offset - offset);
            (*old.fragment()).to_string()
        }
        None => match clean_text::<CustomError<Span<'a>>>(new) {
            Ok((_s, string)) => string,
            Err(e) => match e {
                Err::Error(_) | Err::Failure(_) => unreachable!(),
                Err::Incomplete(_err) => unreachable!(),
            },
        },
    }
}
