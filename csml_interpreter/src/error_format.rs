pub(crate) mod data;

use crate::data::tokens::Span;
use crate::data::{Interval, position::Position, warnings::Warnings};
use nom::{
    AsBytes, Err,
    error::{ContextError, ErrorKind, ParseError},
};

pub use crate::data::error_info::ErrorInfo;
pub(crate) use data::CustomError;
use std::fmt::Write;

// TODO: add link to docs

// Parsing Errors
pub(crate) const ERROR_NUMBER_AS_IDENT: &str = "Int/Float can't be used as identifier";
pub(crate) const ERROR_RESERVED: &str = "reserved keyword can't be used as identifier";
pub(crate) const ERROR_PARSING: &str =
    "Invalid argument. One of the action keywords [say, do, if, ...] is missing";
pub(crate) const ERROR_REMEMBER: &str =
    "'remember' must be assigning to a variable via '='. Example: 'remember key = value'";
pub(crate) const ERROR_USE: &str =
    "'use' must be assigning a variable with keyword 'as'. Example: 'use value as key'";
pub(crate) const ERROR_ACTION_ARGUMENT: &str =
    "expecting valid argument after action keywords. Example: say value";
pub(crate) const ERROR_IMPORT_ARGUMENT: &str =
    "'import' expecting valid function name. Example: 'import function from flow'";
pub(crate) const ERROR_INSERT_ARGUMENT: &str =
    "'insert' expecting valid step name. Example: 'insert step from flow'";
pub(crate) const ERROR_RETURN: &str = "return expects a value to return";
pub(crate) const ERROR_GOTO_STEP: &str = "missing step name after goto";
pub(crate) const ERROR_DOUBLE_QUOTE: &str = "expecting '\"' to end string";
pub(crate) const ERROR_DOUBLE_CLOSE_BRACE: &str = "expecting '}}' to end expandable string";
pub(crate) const ERROR_UNREACHABLE: &str = "unreachable";
pub(crate) const ERROR_WRONG_ARGUMENT_EXPANDABLE_STRING: &str =
    "wrong argument(s) given to expandable string";

// ### Validation
pub(crate) const ERROR_STEP_EXIST: &str = "step does not exist";
pub(crate) const ERROR_INVALID_FLOW: &str = "invalid flow: ";
pub(crate) const ERROR_START_INSTRUCTIONS: &str = "to start an action one of the following instructions is expected: [say, do, if, foreach, goto]";
pub(crate) const ERROR_FOREACH: &str = "foreach only accepts iterable elements like arrays and strings. Example: foreach(elem) in [1, 2, 3]";
pub(crate) const ERROR_FOREACH_INDEX_OVERFLOW: &str = "foreach index overflowed";
pub(crate) const ERROR_FIND_BY_INDEX: &str =
    "index must be of type int or string. Example var.[42] or var.[\"key\"]";
pub(crate) const ERROR_SIZE_IDENT: &str = "key can't be longer than 255 character";
pub(crate) const ERROR_EXPR_TO_LITERAL: &str = "expression can't be converted to Literal";
pub(crate) const ERROR_PAYLOAD_EXCEED_MAX_SIZE: &str = "payload exceeds max payload size (16kb)";

pub(crate) const ERROR_STEP_LIMIT: &str =
    "[Infinite loop] Step limit reached: 100 steps where executed in a single run";

// Event
pub(crate) const ERROR_EVENT_CONTENT_TYPE: &str = "event can only be of ContentType::Event";

// Goto
pub(crate) const ERROR_GOTO_VAR: &str = "variables in goto need to resolve as strings";

// Component
pub(crate) const ERROR_COMPONENT_NAMESPACE: &str = "component must have a function applied";
pub(crate) const ERROR_COMPONENT_UNKNOWN: &str = "function does not exist for component";

// Fn API
pub(crate) const ERROR_FN_ID: &str = "App name must be of type string";
pub(crate) const ERROR_FN_ENDPOINT: &str =
    "App can not be called because apps_endpoint is not set in bot";
pub(crate) const ERROR_FAIL_RESPONSE_JSON: &str = "failed to read response as JSON";

// ### Variables
pub(crate) const ERROR_GET_VAR_INFO: &str = "Expression must be a variable";
pub(crate) const ERROR_JSON_TO_LITERAL: &str = "Number is larger than a 64-bit integer";

// ### Memory
pub(crate) const ERROR_STEP_MEMORY: &str = "Variable does not exist in step's memory";
pub(crate) const ERROR_FIND_MEMORY: &str = "is used before it was saved in memory";

// ### Functions
pub(crate) const ERROR_FN_ARGS: &str = "function arguments are not valid";
pub(crate) const ERROR_FN_COLON: &str =
    "Expecting ':' at the end of function prototype. Example: 'fn name():' ";

// ### Built-in
pub(crate) const ERROR_ONE_OF: &str =
    "OneOf builtin expects one value of type Array. Example: OneOf( [1, 2, 3] )";
pub(crate) const ERROR_VAR_EXISTS: &str =
    "Exists builtin expects one value of type String. Example: Exists( \"var_name\" )";
pub(crate) const ERROR_SHUFFLE: &str =
    "Shuffle builtin expects one value of type Array. Example: Shuffle( [1, 2, 3] )";
pub(crate) const ERROR_LENGTH: &str =
    "Length builtin expects one value of type Array or String. Example: Length( value )";
pub(crate) const ERROR_LENGTH_OVERFLOW: &str = "Length to large to represent as a number.";
pub(crate) const ERROR_FIND: &str = "Find builtin expects 'in' param to be of type String. Example: Find(value, in = \"hola\", case_sensitive = true)";
pub(crate) const ERROR_FLOOR: &str =
    "Floor builtin expects one argument of type float. Example: Floor(4.2)";
pub(crate) const ERROR_UUID: &str = "UUID builtin expects one optional argument of type String. Example: UUID(\"v4\") or UUID(\"v1\")";
pub(crate) const ERROR_HTTP_GET_VALUE: &str = "not found in HTTP object. Use the HTTP() builtin to construct the correct object to make HTTP calls";
pub(crate) const ERROR_HTTP: &str =
    "HTTP builtin expects one url of type string. Example: HTTP(\"https://clevy.io\")";
pub(crate) const ERROR_BASE64: &str =
    "Base64 builtin expects one string. Example: Base64(\"hello world\")";
pub(crate) const ERROR_HEX: &str = "Hex builtin expects one string. Example: Hex(\"hello world\")";
pub(crate) const ERROR_JWT: &str = "JWT builtin expects payload as argument. Example: JWT({
        \"user\": \"name\",
        \"some_key\": {
          \"some_value\": 42
        },
        \"exp\": 1618064023,
        \"iss\": \"CSML STUDIO\"
      })";
pub(crate) const ERROR_SMTP: &str =
    "SMTP builtin expects SMTP Server Address. Example: SMTP(\"smtp.gmail.com\")";
pub(crate) const ERROR_CRYPTO: &str =
    "CRYPTO builtin expects one argument of type string. Example: CRYPTO(\"text\")";
pub(crate) const ERROR_BUILTIN_UNKNOWN: &str = "Unknown builtin";

// ### native Components
pub(crate) const ERROR_HTTP_NOT_DATA: &str = "bad format: no 'data' in HTTP response";
pub(crate) const ERROR_NATIVE_COMPONENT: &str = "native component does not exist";

// ### Constants
pub(crate) const ERROR_CONSTANT_MUTABLE_FUNCTION: &str =
    "Invalid operation constants can not execute self mutable functions";
pub(crate) const ERROR_INVALID_CONSTANT_EXPR: &str =
    "Constant invalid expression type: constants can not be assign this type of expression";

// ### Primitives
// #### Indexing
pub(crate) const ERROR_INDEXING: &str =
    "indexing can only be done in ARRAY, OBJECT or STRING primitive types";

// #### Closure
pub(crate) const ERROR_CLOSURE_UNKNOWN_METHOD: &str = "Closure don't have methods";

// #### Boolean
pub(crate) const ERROR_BOOLEAN_UNKNOWN_METHOD: &str = "is not a method of Boolean";

// #### NUMBER
pub(crate) const ERROR_NUMBER_POW: &str =
    "[pow] takes one parameter of type int or float usage: number.pow(42)";

// #### Float
pub(crate) const ERROR_FLOAT_UNKNOWN_METHOD: &str = "is not a method of Float";

// #### Int
pub(crate) const ERROR_INT_UNKNOWN_METHOD: &str = "is not a method of Int";

// #### Null
pub(crate) const ERROR_NULL_UNKNOWN_METHOD: &str = "is not a method of Null";

// #### String
pub(crate) const ERROR_STRING_DO_MATCH: &str =
    "[do_match] takes one parameter of type String. Usage: string.do_match(\"tag\")";
pub(crate) const ERROR_STRING_APPEND: &str =
    "[append] takes one parameter of type String. Usage: string.append(\"text to append\")";
pub(crate) const ERROR_STRING_REPLACE: &str = "[replace] takes tow parameter of type String. Usage: \"this is old\".replace(\"old\", \"new\")";
pub(crate) const ERROR_STRING_REPLACE_ALL: &str = "[replace_all] takes tow parameter of type String. Usage: \"old old old old\".replace_all(\"old\", \"new\")";
pub(crate) const ERROR_STRING_REPLACE_REGEX: &str = "[replace_regex] takes tow parameter of type String. Usage: \"hello world\".replace_regex(\"world\", \"Clevy\")";
pub(crate) const ERROR_STRING_CONTAINS_REGEX: &str =
    "[contains_regex] takes one parameter of type String. Usage: string.contains_regex(\"regex\")";
pub(crate) const ERROR_STRING_VALID_REGEX: &str = "parameter must be a valid regex expression"; // link to docs
pub(crate) const ERROR_STRING_START_WITH: &str =
    "[starts_with] takes one parameter of type String. Usage: string.starts_with(\"tag\")";
pub(crate) const ERROR_STRING_START_WITH_REGEX: &str = "[starts_with_regex] takes one parameter of type String. Usage: string.start_with_regex(\"regex\")";
pub(crate) const ERROR_STRING_END_WITH: &str =
    "[ends_with] takes one parameter of type String. Usage: string.ends_with(\"tag\")";
pub(crate) const ERROR_STRING_END_WITH_REGEX: &str = "[ends_with_regex] takes one parameter of type String. Usage: string.ends_with_regex(\"regex\")";
pub(crate) const ERROR_STRING_FROM_JSON: &str = "[from_json] [!] string to object failed]";
pub(crate) const ERROR_STRING_SPLIT: &str =
    "[split] takes one parameter of type String. Usage: string.split(\"separator\")";
pub(crate) const ERROR_STRING_MATCH_REGEX: &str =
    "[match_regex] takes one parameter of type String. Usage: string.match_regex(\"regex\")";
pub(crate) const ERROR_STRING_NUMERIC: &str = "the string must be of numeric type in order to use this method. Verify first with 'string.is_number() == true' to check it";
pub(crate) const ERROR_STRING_RHS: &str = "rhs must be of type string";

pub(crate) const ERROR_SLICE_ARG_INT: &str =
    ".slice(start, optional<end>) args need to be of type Integer";
pub(crate) const ERROR_SLICE_ARG_LEN: &str =
    ".slice(start, optional<end>) args need to be inferior to the string length";
pub(crate) const ERROR_SLICE_ARG2: &str =
    ".slice(start, optional<end>) end need to be superior to start in value ex: .slice(2, 5)";

pub(crate) const ERROR_STRING_UNKNOWN_METHOD: &str = "is not a method of String";

// #### Array
pub(crate) const ERROR_ARRAY_TYPE: &str = "value must be of type array";
pub(crate) const ERROR_ARRAY_INDEX_EXIST: &str = "index does not exist";
pub(crate) const ERROR_ARRAY_INDEX_TYPE: &str = "index must be of type int";
pub(crate) const ERROR_ARRAY_BOUNDS: &str =
    "index must be positive and below usize::MAX. Usage: array[1]";
pub(crate) const ERROR_ARRAY_INDEX: &str = "index must be lower than or equal to array.length()";
pub(crate) const ERROR_ARRAY_OVERFLOW: &str =
    "[push] Cannot push inside array, since array limit is ";
pub(crate) const ERROR_ARRAY_OUT_OF_RANGE: &str = "index must be between 0 and usize::MAX";
pub(crate) const ERROR_ARRAY_POP: &str = "[pop] Cannot pop if array is empty";
pub(crate) const ERROR_ARRAY_INIT: &str =
    "[init] parameter must be a positive int. Usage: [].init(size)";
pub(crate) const ERROR_ARRAY_INSERT_AT: &str =
    "[insert_at] takes two arguments. Usage: array.insert_at(1, elem)";
pub(crate) const ERROR_ARRAY_INSERT_AT_INT: &str =
    "[insert_at] first parameter must be of type int. Usage: array.insert_at(1, elem)";
pub(crate) const ERROR_ARRAY_REMOVE_AT: &str =
    "[remove_at] takes one parameter of type Int. Usage: array.remove_at(1) ";
pub(crate) const ERROR_ARRAY_JOIN: &str =
    "[join] takes one parameter of type String. Usage: array.join(\"elem\") ";
pub(crate) const ERROR_ARRAY_UNKNOWN_METHOD: &str = "is not a method of Array";

// #### CRYPTO OBJECT
// ## HMAC and HASH OBJECT
pub(crate) const ERROR_HASH: &str = "Crypto(string) command expect argument of type String";
pub(crate) const ERROR_HASH_ALGO: &str =
    "Invalid Algorithm, supported Algorithms are md5 sha1 sha256 sha384 sha512";
pub(crate) const ERROR_HMAC_KEY: &str = "HMAC key need to be of type string";

pub(crate) const ERROR_DIGEST: &str = "Invalid argument, '.digest' is use incorrectly";
pub(crate) const ERROR_DIGEST_ALGO: &str =
    "Invalid Digest Algorithm, supported Algorithms are hex, base64";

// #### JWT OBJECT
pub(crate) const ERROR_JWT_ALGO: &str =
    "Invalid Algorithm, supported Algorithms are HS256, HS384, HS512";
pub(crate) const ERROR_JWT_SECRET: &str = "secret must be of type String";

pub(crate) const ERROR_JWT_SIGN_CLAIMS: &str =
    "JWT(claims) command expect argument 'claims' of type Object";
pub(crate) const ERROR_JWT_SIGN_ALGO: &str =
    "JWT(claims).sign(algo, secret, Optional<Header>) expect first argument 'algo' of type String";
pub(crate) const ERROR_JWT_SIGN_SECRET: &str = "JWT(claims).sign(algo, secret, Optional<Header>) expect second argument 'claims' of type String";

pub(crate) const ERROR_JWT_TOKEN: &str = "JWT(jwt) command expect argument 'jwt' of type String";

pub(crate) const ERROR_JWT_DECODE_ALGO: &str =
    "JWT(jwt).decode(algo, secret) expect first argument 'algo' of type String";
pub(crate) const ERROR_JWT_DECODE_SECRET: &str =
    "JWT(jwt).decode(algo, secret) expect second argument 'claims' of type String";

pub(crate) const ERROR_JWT_VALIDATION_CLAIMS: &str =
    "JWT(jwt).verify(claims, algo, secret) expect first argument 'claims' of type Object";
pub(crate) const ERROR_JWT_VALIDATION_ALGO: &str =
    "JWT(jwt).verify(claims, algo, secret) expect second argument 'algo' of type String";
pub(crate) const ERROR_JWT_VALIDATION_SECRETE: &str =
    "JWT(jwt).verify(claims, algo, secret) expect third argument 'secrete' of type String";

// #### HTTP OBJECT
pub(crate) const ERROR_HTTP_SET: &str =
    "[set] takes one argument of type Object. Usage: HTTP(...).set( {\"key\": 42} )";
pub(crate) const ERROR_HTTP_QUERY: &str =
    "[query] takes one argument of type Object. Usage: HTTP(...).query( {\"key\": 42} )";

pub(crate) const ERROR_HTTP_SEND: &str =
    "[send] HTTP Object is bad formatted read doc for correct usage";
pub(crate) const ERROR_HTTP_UNKNOWN_METHOD: &str = "is not a method of HTTP";

// #### OBJECT
pub(crate) const ERROR_OBJECT_TYPE: &str = "value must be of type Object";
pub(crate) const ERROR_OBJECT_GET: &str = "key does not exist";
pub(crate) const ERROR_OBJECT_CONTAINS: &str =
    "[contains] takes one argument of type String. Usage: object.contains(\"key\")";
pub(crate) const ERROR_OBJECT_GET_GENERICS: &str =
    "[get_generics] takes one argument of type String. Usage: object.get_generics(\"key\")";
pub(crate) const ERROR_OBJECT_INSERT: &str =
    "[insert] take tow arguments. Usage: object.insert(string, any_type)";
pub(crate) const ERROR_OBJECT_ASSIGN: &str =
    "[assign] take one argument. Usage: object.assign({\"key\": \"value\"})";
pub(crate) const ERROR_OBJECT_REMOVE: &str =
    "[remove] takes one argument of type String. Usage: object.remove(\"key\")";
pub(crate) const ERROR_OBJECT_UNKNOWN_METHOD: &str = "is not a method of Object";

// #### METHODS
pub(crate) const ERROR_METHOD_NAMED_ARGS: &str = "arguments in method are not named";

pub const ERROR_OPS_DIV_INT: &str = "[!] Int: Division by zero";
pub(crate) const ERROR_OPS_DIV_FLOAT: &str = "[!] Float: Division by zero";

pub(crate) const ERROR_ILLEGAL_OPERATION: &str = "illegal operation:";
pub const OVERFLOWING_OPERATION: &str = "overflowing operation:";

////////////////////////////////////////////////////////////////////////////////
// PRiVTE FUNCTION
////////////////////////////////////////////////////////////////////////////////

fn add_context_to_error_message(
    flow_slice: &Span<'_>,
    message: &str,
    line_number: u32,
    column: usize,
    offset: usize,
) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    let prefix = &flow_slice.fragment().as_bytes()[..offset];

    // Find the line that includes the subslice:
    // Find the *last* newline before the substring starts
    let line_begin = prefix
        .iter()
        .rev()
        .position(|&b| b == b'\n')
        .map_or(0, |pos| offset - pos);

    // Find the full line after that newline
    let line = flow_slice.fragment()[line_begin..]
        .lines()
        .next()
        .unwrap_or(&flow_slice.fragment()[line_begin..])
        .trim_end();

    write!(
        &mut result,
        "at line {line_number},\n\
            {line}\n\
            {caret:>column$}\n\
            {context}\n\n",
        line_number = line_number,
        context = message,
        line = line,
        caret = '^',
        column = column,
    )
    // Because `write!` to a `String` is infallible, this `unwrap` is fine.
    .unwrap();

    result
}

#[must_use]
pub(crate) fn gen_error_info(position: Position, message: String) -> ErrorInfo {
    ErrorInfo::new(position, message)
}

#[must_use]
pub(crate) fn gen_warning_info(position: Position, message: String) -> Warnings {
    Warnings { message, position }
}

#[must_use]
pub(crate) fn gen_nom_error<'a, E>(span: Span<'a>, error: &'static str) -> Err<E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    Err::Error(add_context(span, error))
}

#[must_use]
pub(crate) fn gen_nom_failure<'a, E>(span: Span<'a>, error: &'static str) -> Err<E>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    Err::Failure(add_context(span, error))
}

#[must_use]
pub(crate) fn add_context<'a, E>(span: Span<'a>, error: &'static str) -> E
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    E::add_context(span, error, E::from_error_kind(span, ErrorKind::Tag))
}

#[must_use]
pub(crate) fn escalate_nom_error<'a, E, O, F: Fn(E) -> O>(nom: Err<E>, f: F) -> Err<O>
where
    E: ParseError<Span<'a>> + ContextError<Span<'a>>,
    O: ParseError<Span<'a>> + ContextError<Span<'a>>,
{
    match nom {
        Err::Error(e) | Err::Failure(e) => Err::Failure(f(e)),
        Err::Incomplete(needed) => Err::Incomplete(needed),
    }
}

#[must_use]
pub(crate) fn convert_error_from_span<'a>(
    flow_slice: &Span<'a>,
    e: &CustomError<Span<'a>>,
) -> String {
    let message = e.error.as_str();
    let offset = e.input.location_offset();
    // Count the number of newlines in the first `offset` bytes of input
    let line_number = e.input.location_line();
    // The (1-indexed) column number is the offset of our substring into that line
    let column = e.input.get_column();

    add_context_to_error_message(flow_slice, message, line_number, column, offset)
}

#[must_use]
pub(crate) fn convert_error_from_interval(
    flow_slice: &Span<'_>,
    message: &str,
    interval: Interval,
) -> String {
    let offset = interval.offset;
    // Count the number of newlines in the first `offset` bytes of input
    let line_number = interval.start_line;
    // The (1-indexed) column number is the offset of our substring into that line
    let column = interval.start_column as usize;

    add_context_to_error_message(flow_slice, message, line_number, column, offset)
}

#[must_use]
pub(crate) fn gen_infinite_loop_error_msg(infinite_loop: &[(String, String)]) -> String {
    infinite_loop
        .iter()
        .fold(String::new(), |mut acc, (flow, step)| {
            writeln!(&mut acc, "[flow] {flow}, [step] {step}")
                .expect("writing to a string cannot fail");
            acc
        })
}
