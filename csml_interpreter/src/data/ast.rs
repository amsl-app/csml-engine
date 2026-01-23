use crate::data::csml_logs::LogLvl;
use crate::data::tokens::Span;
use crate::data::{ArgsType, Literal};

use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Flow {
    pub flow_instructions: HashMap<InstructionScope, Expr>,
    pub flow_type: FlowType,
    pub constants: HashMap<String, Literal>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum FlowType {
    Normal,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum FromFlow {
    Normal(String),
    Extern(String),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct InsertStep {
    pub name: String,
    pub original_name: Option<String>,
    pub from_flow: String,
    pub interval: Interval,
}

impl Hash for InsertStep {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for InsertStep {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for InsertStep {}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct ImportScope {
    pub name: String,
    pub original_name: Option<String>,
    pub from_flow: FromFlow,
    pub interval: Interval,
}

impl Hash for ImportScope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for ImportScope {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for ImportScope {}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum InstructionScope {
    StepScope(String),
    FunctionScope { name: String, args: Vec<String> },
    ImportScope(ImportScope),
    InsertStep(InsertStep),
    Constant(String),

    // this Variant is used to store all duplicated instructions during parsing
    // and use by the linter to display them all as errors
    DuplicateInstruction(Interval, String),
}

impl Hash for InstructionScope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::StepScope(name) | Self::FunctionScope { name, .. } | Self::Constant(name) => {
                name.hash(state)
            }
            Self::ImportScope(import_scope) => import_scope.hash(state),
            Self::InsertStep(insert_step) => insert_step.hash(state),
            Self::DuplicateInstruction(interval, ..) => interval.hash(state),
        }
    }
}

impl PartialEq for InstructionScope {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::StepScope(name1, ..), Self::StepScope(name2, ..))
            | (
                Self::ImportScope(ImportScope { name: name1, .. }),
                Self::ImportScope(ImportScope { name: name2, .. }),
            )
            | (Self::FunctionScope { name: name1, .. }, Self::FunctionScope { name: name2, .. })
            | (
                Self::InsertStep(InsertStep { name: name1, .. }),
                Self::InsertStep(InsertStep { name: name2, .. }),
            )
            | (Self::Constant(name1), Self::Constant(name2)) => name1 == name2,
            (
                Self::DuplicateInstruction(interval1, ..),
                Self::DuplicateInstruction(interval2, ..),
            ) => interval1 == interval2,

            _ => false,
        }
    }
}

impl Eq for InstructionScope {}

impl Display for InstructionScope {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::StepScope(idents, ..) => write!(f, "step {idents}"),
            Self::FunctionScope { name, .. } => write!(f, "function {name}"),
            Self::ImportScope(ImportScope {
                name,
                original_name: _,
                from_flow,
                ..
            }) => write!(f, "import {name} from {from_flow:?} "),
            Self::InsertStep(InsertStep {
                name,
                original_name: _,
                from_flow,
                ..
            }) => write!(f, "insert {name} from {from_flow:?} "),
            Self::Constant(name) => write!(f, "constant {name}"),
            Self::DuplicateInstruction(index, ..) => {
                write!(f, "duplicate instruction at line {}", index.start_line)
            }
        }
    }
}

impl InstructionScope {
    #[must_use]
    pub fn get_info(&self) -> String {
        match self {
            Self::StepScope(name, ..) => format!("step {name}"),
            Self::FunctionScope { name, .. } => format!("function {name}"),
            Self::Constant(name) => format!("constant {name}"),
            Self::ImportScope(ImportScope { name, .. }) => format!("import {name}"),
            Self::InsertStep(InsertStep { name, .. }) => format!("insert {name}"),
            Self::DuplicateInstruction(_, info) => format!("duplicate {info}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub instruction_type: InstructionScope,
    pub actions: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum GotoValueType {
    Name(Identifier),
    Variable(Box<Expr>),
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum GotoType {
    Step(GotoValueType),
    Flow(GotoValueType),
    StepFlow {
        step: Option<GotoValueType>,
        flow: Option<GotoValueType>,
        bot: Option<GotoValueType>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum DoType {
    Update(AssignType, Box<Expr>, Box<Expr>),
    Exec(Box<Expr>),
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Function {
    pub name: String,
    pub interval: Interval,
    pub args: Box<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum ForgetMemory {
    ALL,
    SINGLE(Identifier),
    LIST(Vec<Identifier>),
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum PreviousType {
    Step(Interval),
    Flow(Interval),
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum AssignType {
    Assignment,
    AdditionAssignment,
    SubtractionAssignment,
    MultiplicationAssignment,
    DivisionAssignment,
    RemainderAssignment,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum ObjectType {
    Goto(GotoType, Interval),
    Previous(PreviousType, Interval),
    Hold(Interval),
    HoldSecure(Interval),
    Say(Box<Expr>),
    Debug(Box<Expr>, Interval),
    Log {
        expr: Box<Expr>,
        interval: Interval,
        log_lvl: LogLvl,
    },
    Return(Box<Expr>),
    Do(DoType),
    Use(Box<Expr>),

    Remember(Identifier, Box<Expr>),
    Assign(AssignType, Box<Expr>, Box<Expr>),
    Forget(ForgetMemory, Interval),

    As(Identifier, Box<Expr>),

    BuiltIn(Function),
    Break(Interval),
    Continue(Interval),
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct InstructionInfo {
    pub index: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, bincode::Encode, bincode::Decode)]
pub struct Block {
    pub commands: Vec<(Expr, InstructionInfo)>,
    pub commands_count: usize,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Hook {
    pub index: i64,
    pub name: String,
    pub step: String,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum BlockType {
    LoopBlock,
    Block,
    IfLoop,
    Step,
    Function,
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum IfStatement {
    IfStmt {
        cond: Box<Expr>,
        consequence: Block,
        then_branch: Option<Box<IfStatement>>,
        last_action_index: usize,
    },
    ElseStmt(Block, Interval),
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum Expr {
    Scope {
        block_type: BlockType,
        scope: Block,
        range: Interval,
    },
    ForEachExpr(Identifier, Option<Identifier>, Box<Expr>, Block, Interval),
    WhileExpr(Box<Expr>, Block, Interval),
    ComplexLiteral(Vec<Expr>, Interval),
    MapExpr {
        object: HashMap<String, Expr>,
        is_in_sub_string: bool, // this value is used to determine if this object was declared inside a string or not
        interval: Interval,
    },
    VecExpr(Vec<Expr>, Interval),
    InfixExpr(Infix, Box<Expr>, Box<Expr>),
    PostfixExpr(Vec<Prefix>, Box<Expr>),
    ObjectExpr(ObjectType),
    IfExpr(IfStatement),

    PathExpr {
        literal: Box<Expr>,
        path: Vec<(Interval, PathState)>,
    },
    IdentExpr(Identifier),

    LitExpr {
        literal: Literal,
        in_in_substring: bool, // this value is used to determine if this literal was declared inside a string or not
    },
}

impl Expr {
    #[must_use]
    pub fn new_idents(ident: String, interval: Interval) -> Identifier {
        Identifier { ident, interval }
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum Infix {
    Addition,
    Subtraction,
    Divide,
    Multiply,
    Remainder,

    Match,
    NotMatch,

    Equal,
    NotEqual,
    GreaterThanEqual,
    LessThanEqual,
    GreaterThan,
    LessThan,

    And,
    Or,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum Prefix {
    Not,
}

#[derive(
    PartialEq,
    Debug,
    Clone,
    Eq,
    Hash,
    Copy,
    Serialize,
    Deserialize,
    Default,
    bincode::Encode,
    bincode::Decode,
)]
pub struct Interval {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub offset: usize,
}

impl Interval {
    #[must_use]
    pub fn new_as_u32(
        start_line: u32,
        start_column: u32,
        offset: usize,
        end_line: Option<u32>,
        end_column: Option<u32>,
    ) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
            offset,
        }
    }

    #[must_use]
    pub fn new_as_span(span: Span) -> Self {
        Self {
            start_line: span.location_line(),
            // If this ever happens, the row is over 512 megabytes
            start_column: span.get_column().to_u32().expect("Column can't exceed u32"),
            end_line: None,
            end_column: None,
            offset: span.location_offset(),
        }
    }

    pub fn add_end(&mut self, end: Self) {
        self.end_line = Some(end.start_line);
        self.end_column = Some(end.start_column);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum PathState {
    ExprIndex(Expr),
    StringIndex(String),
    Func(Function),
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub enum PathLiteral {
    VecIndex(usize),
    MapIndex(String),
    Func {
        name: String,
        interval: Interval,
        args: ArgsType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct Identifier {
    pub ident: String,
    pub interval: Interval,
}

impl Identifier {
    #[must_use]
    pub fn new(ident: &str, interval: Interval) -> Self {
        Self {
            ident: ident.to_string(),
            interval,
        }
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ident.partial_cmp(&other.ident)
    }
}
