use crate::data::error_info::ErrorInfo;
use crate::data::{Hold, Literal, MSG, Memory, Message};
use crate::parser::ExitCondition;

use core::ops::Add;
use std::ops::ControlFlow;
use std::sync::mpsc;

////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURE
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default)]
pub struct MessageData {
    pub memories: Option<Vec<Memory>>,
    pub messages: Vec<Message>,
    pub hold: Option<Hold>,
    pub exit_condition: Option<ExitCondition>,
}

////////////////////////////////////////////////////////////////////////////////
// TRAIT FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl Add<Self> for MessageData {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            memories: match (self.memories, other.memories) {
                (Some(memory), None) => Some(memory),
                (None, Some(new_memory)) => Some(new_memory),
                (Some(memory), Some(new_memory)) => Some([&memory[..], &new_memory[..]].concat()),
                _ => None,
            },
            messages: [&self.messages[..], &other.messages[..]].concat(),
            hold: self.hold,
            exit_condition: match (&self.exit_condition, &other.exit_condition) {
                (Some(exit_condition), _) | (_, Some(exit_condition)) => {
                    Some(exit_condition.clone())
                }
                _ => None,
            },
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// STATIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl MessageData {
    #[must_use]
    pub fn error_to_message(
        result: Result<Self, ErrorInfo>,
        sender: Option<&mpsc::Sender<MSG>>,
    ) -> Self {
        match result {
            Ok(message_data) => message_data,
            Err(err) => {
                let json_msg = serde_json::json!({"error": err.format_error()});

                MSG::send(
                    sender,
                    MSG::Error(Message {
                        content_type: "error".to_owned(),
                        content: json_msg.clone(),
                    }),
                );

                Self {
                    memories: None,
                    messages: vec![Message {
                        content_type: "error".to_owned(),
                        content: json_msg,
                    }],
                    hold: None,
                    exit_condition: Some(ExitCondition::Error),
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// METHOD FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl MessageData {
    #[must_use]
    pub fn add_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    pub fn add_to_memory(&mut self, key: &str, value: &Literal) {
        let content_type = &value.content_type;

        let memories = self.memories.get_or_insert_with(Vec::new);
        memories.push(Memory {
            key: key.to_owned(),
            value: value.primitive.format_mem(content_type, true),
        });
    }

    pub fn branch(&mut self) -> ControlFlow<()> {
        match self.exit_condition {
            Some(ExitCondition::Break) => {
                self.exit_condition = None;
                ControlFlow::Break(())
            }
            Some(ExitCondition::Continue) => {
                self.exit_condition = None;
                ControlFlow::Continue(())
            }
            Some(_) => ControlFlow::Break(()),
            None => ControlFlow::Continue(()),
        }
    }
}
