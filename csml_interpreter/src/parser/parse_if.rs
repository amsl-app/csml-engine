use crate::data::tokens::{LParen, RParen};
use crate::data::{
    ast::{Expr, IfStatement},
    tokens::{ELSE, IF, Span},
};
use crate::parser::operator::parse_operator::parse_operator;
use crate::parser::parse_braces::parse_brace;
use crate::parser::{
    parse_comments::comment,
    parse_scope::{parse_implicit_scope, parse_scope},
    tools::get_interval,
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    combinator::opt,
    error::{ContextError, ParseError},
    sequence::delimited,
    sequence::preceded,
};
////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn parse_strict_condition_group<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    delimited(
        preceded(comment, parse_brace(LParen)),
        parse_operator,
        preceded(comment, parse_brace(RParen)),
    )
    .parse(s)
}

fn parse_else_if<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Box<IfStatement>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = preceded(comment, tag(ELSE)).parse(s)?;
    let (s, _) = preceded(comment, tag(IF)).parse(s)?;

    let (s, condition) = parse_strict_condition_group(s)?;

    let (s, block) = alt((parse_scope, parse_implicit_scope)).parse(s)?;

    let (s, opt) = opt(alt((parse_else_if, parse_else))).parse(s)?;

    Ok((
        s,
        Box::new(IfStatement::IfStmt {
            cond: Box::new(condition),
            consequence: block,
            then_branch: opt,
            last_action_index: 0, // this wil be update in parse_root
        }),
    ))
}

fn parse_else<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Box<IfStatement>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = preceded(comment, tag(ELSE)).parse(s)?;
    let (s, mut interval) = get_interval(s)?;

    let (s, block) = alt((parse_scope, parse_implicit_scope)).parse(s)?;

    let (s, end) = get_interval(s)?;
    interval.add_end(end);

    Ok((s, Box::new(IfStatement::ElseStmt(block, interval))))
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTION
////////////////////////////////////////////////////////////////////////////////

pub fn parse_if<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = preceded(comment, tag(IF)).parse(s)?;
    let (s, condition) = parse_strict_condition_group(s)?;

    let (s, block) = alt((parse_scope, parse_implicit_scope)).parse(s)?;

    let (s, opt) = opt(alt((parse_else_if, parse_else))).parse(s)?;

    Ok((
        s,
        Expr::IfExpr(IfStatement::IfStmt {
            cond: Box::new(condition),
            consequence: block,
            then_branch: opt,
            last_action_index: 0, // this wil be update in parse_root
        }),
    ))
}

////////////////////////////////////////////////////////////////////////////////
// TEST FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    pub fn test_if(s: Span) -> IResult<Span, Expr> {
        preceded(comment, parse_if).parse(s)
    }

    #[test]
    fn ok_normal_if1() {
        let string = Span::new("if ( event ) { say \"hola\" }");
        let (_, r) = test_if(string).unwrap();
        assert!(matches!(r, Expr::IfExpr(IfStatement::IfStmt { .. })));
    }

    #[test]
    fn ok_normal_if2() {
        let string = Span::new("if ( event ) { say \"hola\"  say event }");
        test_if(string).unwrap();
    }

    #[test]
    fn ok_normal_else_if1() {
        let string =
            Span::new("if ( event ) { say \"hola\" } else if ( event ) { say \" hola 2 \" }");
        test_if(string).unwrap();
    }

    #[test]
    fn err_normal_if1() {
        let string = Span::new("if ");
        test_if(string).expect_err("need to fail");
    }

    #[test]
    fn err_normal_if2() {
        let string = Span::new("if ( event ) ");
        test_if(string).expect_err("need to fail");
    }

    #[test]
    fn err_normal_if3() {
        let string = Span::new("if ( event { say \"hola\"  say event }");
        test_if(string).expect_err("need to fail");
    }
}
