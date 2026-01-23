pub mod ast;
pub mod context;
pub(crate) mod core;
pub mod csml_bot;
pub mod csml_flow;
pub mod csml_logs;
pub mod csml_result;
pub mod error_info;
pub mod event;
pub(crate) mod fn_args_type;
pub mod hold;
pub(crate) mod literal;
pub(crate) mod memories;
pub(crate) mod message;
pub mod message_data;
pub(crate) mod msg;
pub mod position;
pub mod primitive;
pub(crate) mod tokens;
pub mod warnings;

pub use ast::Interval;
pub use context::{ApiInfo, Context, PreviousBot};
pub use core::Data;
pub use csml_bot::{CsmlBot, Module, MultiBot};
pub use csml_flow::CsmlFlow;
pub use csml_model::Client;
pub use csml_result::CsmlResult;
pub use event::Event;
pub use fn_args_type::ArgsType;
pub use hold::{Hold, IndexInfo};
pub use literal::Literal;
pub use memories::{Memory, MemoryType};
pub use message::Message;
pub use message_data::MessageData;
pub use position::Position;

pub use msg::MSG;

// limit of steps in a single execution
pub(crate) const STEP_LIMIT: usize = 100;
