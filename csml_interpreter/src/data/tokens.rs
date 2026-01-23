use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

pub trait Token {
    const TOKEN: &'static str;
    const MISSING_ERROR: &'static str;
}

macro_rules! token {
    ($name:ident, $text:expr) => {
        pub struct $name;

        impl Token for $name {
            const TOKEN: &'static str = $text;
            const MISSING_ERROR: &'static str = concat!("expecting ", $text, "'");
        }
    };
}

pub trait Braces {
    type Left: Token;
    type Right: Token;
}

macro_rules! braces {
    ($name:ident, $left:ident, $right:ident) => {
        pub struct $name;

        impl Braces for $name {
            type Left = $left;
            type Right = $right;
        }
    };
}

pub const START_COMMENT: &str = "/*";
pub const END_COMMENT: &str = "*/";

pub const DOLLAR: &str = "$";

pub const ADDITION: &str = "+";
pub const SUBTRACTION: &str = "-";
pub const DIVIDE: &str = "/";
pub const MULTIPLY: &str = "*";
pub const REMAINDER: &str = "%";
pub const NOT: &str = "!";

pub const EQUAL: &str = "==";
pub const NOT_EQUAL: &str = "!=";
pub const ASSIGN: &str = "=";

pub const OR: &str = "||";
pub const AND: &str = "&&";

pub const SUBTRACTION_ASSIGNMENT: &str = "-=";
pub const ADDITION_ASSIGNMENT: &str = "+=";
pub const MULTIPLY_ASSIGNMENT: &str = "*=";
pub const DIVISION_ASSIGNMENT: &str = "/=";
pub const REMAINDER_ASSIGNMENT: &str = "%=";

pub const GREATER_THAN_EQUAL: &str = ">=";
pub const LESS_THAN_EQUAL: &str = "<=";
pub const GREATER_THAN: &str = ">";
pub const LESS_THAN: &str = "<";

pub const COMMA: &str = ",";
pub const DOT: &str = ".";
pub const COLON: &str = ":";
pub const DOUBLE_QUOTE: &str = "\"";
pub const BACKSLASH_DOUBLE_QUOTE: &str = "\\\"";

token!(LParen, "(");
token!(RParen, ")");
braces!(Paren, LParen, RParen);

token!(LBrace, "{");
token!(RBrace, "}");
braces!(Brace, LBrace, RBrace);

token!(LBracket, "[");
token!(RBracket, "]");
braces!(Bracket, LBracket, RBracket);

pub const FOREACH: &str = "foreach";
pub const WHILE: &str = "while";
pub const IF: &str = "if";
pub const ELSE: &str = "else";

pub const IMPORT: &str = "import";
pub const CONST: &str = "const";
pub const INSERT: &str = "insert";
pub const FROM: &str = "from";
pub const AS: &str = "as";
pub const IN: &str = "in";
pub const DO: &str = "do";
pub const EVENT: &str = "event";
pub const COMPONENT: &str = "Component";

pub const FLOW: &str = "flow";
pub const STEP: &str = "step";
pub const SAY: &str = "say";
pub const DEBUG_ACTION: &str = "debug";
pub const LOG_ACTION: &str = "log";
pub const USE: &str = "use";
token!(Hold, "hold");
token!(HoldSecure, "hold_secure");
pub const GOTO: &str = "goto";
pub const PREVIOUS: &str = "previous";
pub const MATCH: &str = "match";
pub const NOT_MATCH: &str = "!match";
pub const REMEMBER: &str = "remember";
pub const FORGET: &str = "forget";
pub const _METADATA: &str = "_metadata";
pub const _MEMORY: &str = "_memory";
pub const _ENV: &str = "_env";
token!(Break, "break");
token!(Continue, "continue");
pub const RETURN: &str = "return";

pub const TRUE: &str = "true";
pub const FALSE: &str = "false";
pub const NULL: &str = "null";

pub const OBJECT_TYPE: &str = "object";
pub const ARRAY: &str = "array";
pub const TEXT_TYPE: &str = "text";
pub const STRING: &str = "string";
pub const INT: &str = "int";
pub const FLOAT: &str = "float";
pub const BOOLEAN: &str = "boolean";
pub const CLOSURE: &str = "closure";

pub const TYPES: &[&str] = &[
    CLOSURE,
    OBJECT_TYPE,
    ARRAY,
    TEXT_TYPE,
    STRING,
    INT,
    FLOAT,
    BOOLEAN,
    NULL,
];

pub const UTILISATION_RESERVED: &[&str] = &[
    FOREACH,
    WHILE,
    IF,
    ELSE,
    IMPORT,
    CONST,
    INSERT,
    AS,
    DO,
    FLOW,
    STEP,
    SAY,
    USE,
    Hold::TOKEN,
    GOTO,
    MATCH,
    REMEMBER,
    FORGET,
    Break::TOKEN,
    COMPONENT,
];

pub const ASSIGNATION_RESERVED: &[&str] = &[
    FOREACH,
    WHILE,
    IF,
    ELSE,
    IMPORT,
    AS,
    DO,
    EVENT,
    FLOW,
    STEP,
    SAY,
    USE,
    Hold::TOKEN,
    GOTO,
    MATCH,
    REMEMBER,
    FORGET,
    _METADATA,
    _MEMORY,
    _ENV,
    TRUE,
    FALSE,
    NULL,
    Break::TOKEN,
    COMPONENT,
];

pub const ONE_OF: &str = "OneOf";
pub const SHUFFLE: &str = "Shuffle";
pub const LENGTH: &str = "Length";
pub const FIND: &str = "Find";
pub const RANDOM: &str = "Random";
pub const FLOOR: &str = "Floor";

pub const FN: &str = "Fn";
pub const APP: &str = "App";
pub const HTTP: &str = "HTTP";
pub const SMTP: &str = "SMTP";
pub const JWT: &str = "JWT";
pub const CRYPTO: &str = "Crypto";
pub const BASE64: &str = "Base64";
pub const HEX: &str = "Hex";
pub const DEBUG: &str = "Debug";
pub const UUID: &str = "UUID";
pub const TIME: &str = "Time";
pub const EXISTS: &str = "Exists";

pub const OBJECT: &str = "Object";

pub const BUILT_IN: &[&str] = &[
    ONE_OF, SHUFFLE, LENGTH, FIND, RANDOM, FLOOR, FN, APP, HTTP, OBJECT, DEBUG, UUID, BASE64, HEX,
    JWT, CRYPTO, TIME, SMTP, EXISTS,
];

pub const OR_BUILT_IN: &str = "Or";

pub const BUILT_IN_WITHOUT_WARNINGS: &[&str] = &[OR_BUILT_IN];
