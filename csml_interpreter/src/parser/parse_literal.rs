use crate::data::{
    Literal,
    ast::Expr,
    tokens::{FALSE, NULL, Span, TRUE},
};
use crate::parser::tools::get_string;
use crate::parser::tools::get_tag;
use crate::parser::{parse_comments::comment, tools::get_interval};

use crate::data::primitive::{
    boolean::PrimitiveBoolean, float::PrimitiveFloat, int::PrimitiveInt, null::PrimitiveNull,
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{opt, recognize},
    error::{ContextError, ParseError},
    multi::{many0, many1},
    sequence::{preceded, terminated},
};

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn signed_digits<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    recognize((opt(one_of("+-")), decimal)).parse(s)
}

fn decimal<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))).parse(s)
}

fn parse_integer<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, interval) = get_interval(s)?;
    let (s, int) = get_int(s)?;

    let expression = Expr::LitExpr {
        literal: PrimitiveInt::get_literal(int, interval),
        in_in_substring: false,
    };
    Ok((s, expression))
}

fn floating_point<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    alt((
        // Case one: .42
        recognize((char('.'), decimal)), // Case two: 42.42
        recognize((opt(one_of("+-")), decimal, preceded(char('.'), decimal))),
    ))
    .parse(s)
}

fn parse_float<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, interval) = get_interval(s)?;
    let (s, float_raw) = floating_point(s)?;
    let float = float_raw.fragment().parse::<f64>().unwrap_or(0.0);

    let expression = Expr::LitExpr {
        literal: PrimitiveFloat::get_literal(float, interval),
        in_in_substring: false,
    };

    Ok((s, expression))
}

fn parse_number<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    alt((parse_float, parse_integer)).parse(s)
}

fn parse_true<'a, E>(s: Span<'a>) -> IResult<Span<'a>, PrimitiveBoolean, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = tag(TRUE)(s)?;

    Ok((s, PrimitiveBoolean::new(true)))
}

fn parse_false<'a, E>(s: Span<'a>) -> IResult<Span<'a>, PrimitiveBoolean, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, _) = tag(FALSE)(s)?;

    Ok((s, PrimitiveBoolean::new(false)))
}

fn parse_boolean<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, interval) = get_interval(s)?;
    let (s, boolean) = alt((parse_true, parse_false)).parse(s)?;

    let primitive = Box::new(boolean);
    let expression = Expr::LitExpr {
        literal: Literal {
            content_type: "boolean".to_owned(),
            primitive,
            additional_info: None,
            secure_variable: false,
            interval,
        },
        in_in_substring: false,
    };

    Ok((s, expression))
}

fn parse_null<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    let (s, interval) = get_interval(s)?;
    let (s, name) = preceded(comment, get_string).parse(s)?;
    let (s, ()) = get_tag(name.to_ascii_lowercase(), NULL)(s)?;

    let expression = Expr::LitExpr {
        literal: PrimitiveNull::get_literal(interval),
        in_in_substring: false,
    };

    Ok((s, expression))
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTION
////////////////////////////////////////////////////////////////////////////////

pub fn get_int<'a, E>(s: Span<'a>) -> IResult<Span<'a>, i64, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    // map_res(signed_digits, |s: Span| s.fragment().parse::<i64>())(s)
    let (s, raw_digits) = signed_digits(s)?;
    let int = raw_digits.fragment().parse::<i64>().unwrap_or(0);

    Ok((s, int))
}

pub fn parse_literal_expr<'a, E>(s: Span<'a>) -> IResult<Span<'a>, Expr, E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    // TODO: span: preceded( comment ,  position!() ?
    preceded(comment, alt((parse_number, parse_boolean, parse_null))).parse(s)
}

////////////////////////////////////////////////////////////////////////////////
// TEST FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{error::ErrorKind, *};

    pub fn test_literal(s: Span) -> IResult<Span, Expr> {
        let var = parse_literal_expr(s);
        var.and_then(|(s, v)| {
            if s.fragment().is_empty() {
                Ok((s, v))
            } else {
                Err(Err::Error(error::Error::new(s, ErrorKind::Tag)))
            }
        })
    }

    #[test]
    fn ok_int() {
        let string = Span::new(" +42");
        test_literal(string).unwrap();
    }

    #[test]
    fn ok_float() {
        let string = Span::new(" -42.42");
        test_literal(string).unwrap();
    }

    #[test]
    fn ok_bool() {
        let string = Span::new(" true");
        test_literal(string).unwrap();
    }

    #[test]
    fn err_sign() {
        let string = Span::new(" +++++4");
        test_literal(string).expect_err("need to fail");
    }

    #[test]
    fn err_float1() {
        let string = Span::new(" 2.2.2");
        test_literal(string).expect_err("need to fail");
    }

    #[test]
    fn err_float2() {
        let string = Span::new(" 3,2 ");
        test_literal(string).expect_err("need to fail");
    }
}
