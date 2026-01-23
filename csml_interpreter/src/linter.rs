pub(crate) mod core;
pub(crate) mod data;

use crate::data::ast::Flow;
pub(crate) use data::{
    ConstantInfo, FlowConstantUse, FunctionCallInfo, FunctionInfo, ImportInfo, InsertInfo,
    LinterInfo, ScopeType, State, StepBreakers, StepInfo,
};
use std::collections::HashMap;

pub(crate) struct FlowToValidate<'a> {
    pub flow_name: String,
    pub ast: Flow,
    pub raw_flow: &'a str,
}

impl FlowToValidate<'_> {
    #[must_use]
    pub fn get_flows(flows: Vec<Self>) -> HashMap<String, Flow> {
        flows
            .into_iter()
            .map(|flow| (flow.flow_name, flow.ast))
            .collect::<HashMap<String, Flow>>()
    }
}
