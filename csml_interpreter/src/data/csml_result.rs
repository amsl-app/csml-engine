use crate::data::ast::Flow;
use crate::data::error_info::ErrorInfo;
use crate::data::warnings::Warnings;

use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CsmlResult {
    pub flows: HashMap<String, Flow>,
    pub extern_flows: HashMap<String, Flow>,
    pub warnings: Vec<Warnings>,
    pub errors: Vec<ErrorInfo>,
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTION
////////////////////////////////////////////////////////////////////////////////

impl CsmlResult {
    #[must_use]
    pub fn new(
        flows: HashMap<String, Flow>,
        extern_flows: HashMap<String, Flow>,
        warnings: Vec<Warnings>,
        errors: Vec<ErrorInfo>,
    ) -> Self {
        Self {
            flows,
            extern_flows,
            warnings,
            errors,
        }
    }
}
