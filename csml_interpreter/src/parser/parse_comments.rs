use crate::data::tokens::{END_COMMENT, START_COMMENT, Span};
use nom::{
    Err, IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till, take_until},
    character::complete::multispace0,
    error::ParseError,
    multi::many0,
    sequence::delimited,
};

fn comment_single_line<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    let (s, _) = tag("//")(s)?;

    take_till(|ch| ch == '\n').parse(s)
}

fn comment_delimited<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    let (s, _) = tag(START_COMMENT)(s)?;
    let val: IResult<Span<'a>, Span<'a>, E> = take_until(END_COMMENT).parse(s);
    match val {
        Ok((s, _)) => tag(END_COMMENT)(s),
        // Error in comment_delimited is if '*/' is not found so the rest of the file is commented
        Err(Err::Error(_e) | Err::Failure(_e)) => Ok((Span::new(""), Span::new(""))),
        Err(Err::Incomplete(_)) => Ok((Span::new(""), Span::new(""))),
    }
}

fn all_comments<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    alt((comment_single_line, comment_delimited)).parse(s)
}

pub fn comment<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Span<'a>, E> {
    let (s, _) = multispace0(s)?;

    let (s, _) = match many0(ws(all_comments)).parse(s) {
        Ok(val) => val,
        Err(Err::Error((s, _val)) | Err::Failure((s, _val))) => return Ok((s, s)),
        Err(Err::Incomplete(i)) => return Err(Err::Incomplete(i)),
    };

    Ok((s, s))
}

fn ws<'a, F, O, E>(inner: F) -> impl Parser<Span<'a>, Output = O, Error = E>
where
    F: 'a,
    F: Fn(Span<'a>) -> IResult<Span<'a>, O, E>,
    E: ParseError<Span<'a>>,
{
    delimited(multispace0, inner, multispace0)
}
