use crate::data::{
    Hold, Interval, Literal,
    primitive::{PrimitiveObject, PrimitiveType},
};

use crate::interpreter::{json_to_literal, memory_to_literal};

use crate::error_format::ErrorInfo;
use csml_model::Client;
use nom::lib::std::collections::HashMap;
use serde::{Deserialize, Serialize};
////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURE
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ApiInfo {
    pub client: Client,
    pub apps_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextStepInfo {
    Normal(String),
    UnknownFlow(String),
    InsertedStep { step: String, flow: String },
}

impl ContextStepInfo {
    #[must_use]
    pub fn get_step(&self) -> &str {
        match self {
            Self::Normal(step) | Self::UnknownFlow(step) | Self::InsertedStep { step, flow: _ } => {
                step
            }
        }
    }

    #[must_use]
    fn is_step(&self, other: &str) -> bool {
        self.get_step() == other
    }

    #[must_use]
    pub fn is_start(&self) -> bool {
        self.is_step("start")
    }

    #[must_use]
    pub fn is_end(&self) -> bool {
        self.is_step("end")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PreviousBot {
    pub bot: String,
    pub flow: String,
    pub step: String,
}

#[derive(Debug, Clone)]
pub struct Context {
    pub current: HashMap<String, Literal>,
    pub metadata: HashMap<String, Literal>,
    pub api_info: Option<ApiInfo>,
    pub hold: Option<Hold>,
    pub step: ContextStepInfo,
    pub flow: String,
    pub previous_bot: Option<PreviousBot>,
}

////////////////////////////////////////////////////////////////////////////////
// STATIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

#[must_use]
fn to_literal<T, F: Fn(T, Interval, &str) -> Result<Literal, ErrorInfo>>(
    f: F,
    lit: T,
    flow_name: &str,
) -> HashMap<String, Literal> {
    if let Ok(vars) = f(
        lit,
        Interval {
            start_line: 0,
            start_column: 0,
            end_line: None,
            end_column: None,
            offset: 0,
        },
        flow_name,
    ) && vars.primitive.get_type() == PrimitiveType::PrimitiveObject
        && let Some(map) = vars.primitive.as_any().downcast_ref::<PrimitiveObject>()
    {
        return map.value.clone();
    }
    HashMap::new()
}

#[must_use]
pub fn get_hashmap_from_mem(lit: &serde_json::Value, flow_name: &str) -> HashMap<String, Literal> {
    to_literal(memory_to_literal, lit, flow_name)
}

#[must_use]
pub fn get_hashmap_from_json(lit: &serde_json::Value, flow_name: &str) -> HashMap<String, Literal> {
    to_literal(json_to_literal, lit, flow_name)
}

impl Context {
    #[must_use]
    pub fn new(
        current: HashMap<String, Literal>,
        metadata: HashMap<String, Literal>,
        api_info: Option<ApiInfo>,
        hold: Option<Hold>,
        step: &str,
        flow: &str,
        previous_bot: Option<PreviousBot>,
    ) -> Self {
        Self {
            current,
            metadata,
            api_info,
            hold,
            step: ContextStepInfo::Normal(step.to_owned()),
            flow: flow.to_owned(),
            previous_bot,
        }
    }
}
