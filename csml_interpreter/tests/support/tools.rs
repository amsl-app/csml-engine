use csml_interpreter::data::Context;
use csml_interpreter::data::csml_bot::CsmlBot;
use csml_interpreter::data::csml_flow::CsmlFlow;
use csml_interpreter::data::event::Event;
use csml_interpreter::data::message_data::MessageData;
use csml_interpreter::{interpret, load_components};
use serde_json::{Value, json, map::Map};

use std::fs::File;
use std::io::prelude::*;

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn read_file(file_path: String) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    Ok(contents)
}

#[allow(dead_code)]
pub fn format_message(event: &Event, context: Context, filepath: &str) -> Box<MessageData> {
    let content = read_file(filepath.to_string()).unwrap();

    let flow = CsmlFlow::new("id", "flow", &content, Vec::default());
    let native_component = load_components().unwrap();

    let bot = CsmlBot::new(
        "id",
        "bot",
        None,
        vec![flow],
        Some(native_component),
        None,
        "flow",
        None,
        None,
        None,
        None,
        None,
    );

    interpret(&bot, context, event, None)
}

#[allow(dead_code)]
pub fn message_to_json_value(result: Box<MessageData>) -> Value {
    let mut message: Map<String, Value> = Map::new();
    let mut vec = vec![];
    let mut memories = vec![];

    for msg in &result.messages {
        vec.push(msg.message_to_json());
    }

    if let Some(mem) = result.memories {
        for elem in &mem {
            let mut map = Map::new();
            map.insert("key".to_owned(), json!(elem.key.clone()));
            map.insert("value".to_owned(), elem.value.clone());
            memories.push(json!(map));
        }
    }

    message.insert("memories".to_owned(), Value::Array(memories));
    message.insert("messages".to_owned(), Value::Array(vec));

    Value::Object(message)
}
