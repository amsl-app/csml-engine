use crate::data::Literal;
use crate::data::primitive::PrimitiveObject;

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryType {
    Event(String),
    Metadata,
    Use,
    Remember,
    Constant,
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub key: String,
    pub value: serde_json::Value,
}

impl Memory {
    #[must_use]
    pub fn new(key: String, value: Literal) -> Self {
        let mut formatted_value = value.primitive.format_mem(&value.content_type, true);

        if let Some(obj) = value.additional_info {
            formatted_value = serde_json::json!({
                "_additional_info": PrimitiveObject::obj_literal_to_json(&obj),
                "value": formatted_value
            });
        }

        Self {
            key,
            value: formatted_value,
        }
    }
}
