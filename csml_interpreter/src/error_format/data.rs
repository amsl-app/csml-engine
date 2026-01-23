use nom::error::{ContextError, ErrorKind, FromExternalError, ParseError};

#[derive(Clone, Debug, PartialEq)]
pub struct CustomError<I> {
    pub input: I,
    pub end: Option<I>,
    pub error: String,
}

impl<I: std::fmt::Display> ParseError<I> for CustomError<I> {
    //TODO: update this in nom 6
    fn from_error_kind(input: I, _kind: ErrorKind) -> Self {
        Self {
            input,
            end: None,
            error: String::new(),
        }
    }

    fn append(input: I, _kind: ErrorKind, other: Self) -> Self {
        Self {
            input,
            end: Some(other.input),
            error: other.error,
        }
    }
}

impl<I: std::fmt::Display> ContextError<I> for CustomError<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        if other.error.is_empty() {
            other.input = input;
            ctx.clone_into(&mut other.error);
        }
        other
    }
}

impl<I, E> FromExternalError<I, E> for CustomError<I> {
    /// Create a new error from an input position and an external error
    fn from_external_error(input: I, _kind: ErrorKind, _e: E) -> Self {
        Self {
            input,
            end: None,
            error: String::new(),
        }
    }
}
