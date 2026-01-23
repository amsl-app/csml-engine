mod support;

use csml_interpreter::data::context::Context;
use csml_interpreter::data::event::Event;

use crate::support::tools::format_message;
use crate::support::tools::message_to_json_value;

use serde_json::Value;
use std::collections::HashMap;

#[test]
fn event_step_0() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "content"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new(
            "content_type",
            "content",
            Value::Object(serde_json::Map::new()),
        ),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_0",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_step_1() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"yolo": "my name is yolo"}, "content_type":"object"},
            {"content":{"text": "my name is yolo"}, "content_type":"text"}

        ]}"#;

    let mut map = serde_json::Map::new();
    let mut other_map = serde_json::Map::new();

    other_map.insert(
        "yolo".to_owned(),
        Value::String("my name is yolo".to_owned()),
    );
    map.insert("toto".to_owned(), Value::Object(other_map));

    let msg = format_message(
        &Event::new("content_type", "content", Value::Object(map)),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_1",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_step_2() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "content_type"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new(
            "content_type",
            "content",
            Value::Object(serde_json::Map::new()),
        ),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_2",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_step_3() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{}, "content_type":"content_type"}
        ]}"#;
    let msg = format_message(
        &Event::new(
            "content_type",
            "content",
            Value::Object(serde_json::Map::new()),
        ),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_3",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_step_4() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text":"true"}, "content_type":"text"}
        ]}"#;

    let mut map = serde_json::Map::new();

    map.insert("text".to_owned(), Value::String("42".to_owned()));

    let msg = format_message(
        &Event::new("content_type", "content", Value::Object(map)),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_4",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_step_5() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text":"true"}, "content_type":"text"}
        ]}"#;

    let mut map = serde_json::Map::new();

    map.insert("text".to_owned(), Value::String("hola@toto.com".to_owned()));

    let msg = format_message(
        &Event::new("content_type", "content", Value::Object(map)),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_5",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_step_6() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text":"true"}, "content_type":"text"}
        ]}"#;

    let mut map = serde_json::Map::new();

    map.insert("text".to_owned(), Value::String("a".to_owned()));

    let msg = format_message(
        &Event::new("content_type", "content", Value::Object(map)),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_6",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_step_7() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text":"true"}, "content_type":"text"}
        ]}"#;

    let mut map = serde_json::Map::new();

    map.insert("payload".to_owned(), Value::String("a".to_owned()));

    let msg = format_message(
        &Event::new("content_type", "text", Value::Object(map)),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_7",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_step_8() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text":"false"}, "content_type":"text"}
        ]}"#;

    let mut map = serde_json::Map::new();

    map.insert("text".to_owned(), Value::String("b".to_owned()));

    let msg = format_message(
        &Event::new("content_type", "content", Value::Object(map)),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_8",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_step_9() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text":"true"}, "content_type":"text"}
        ]}"#;

    let mut map = serde_json::Map::new();

    map.insert("text".to_owned(), Value::String("a".to_owned()));

    let msg = format_message(
        &Event::new("content_type", "content", Value::Object(map)),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_9",
            "flow",
            None,
        ),
        "CSML/basic_test/event.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn event_types() {
    let context = Context::new(
        HashMap::new(),
        HashMap::new(),
        None,
        None,
        "event_types",
        "flow",
        None,
    );

    let ok_types = ["text", "payload"];
    let err_types = ["file", "audio", "video", "image", "url", "flow_trigger"];

    for ok_type in &ok_types {
        let mut map = serde_json::Map::new();
        map.insert((*ok_type).to_string(), Value::String("42".to_owned()));
        let msg = format_message(
            &Event::new("content_type", "content", Value::Object(map)),
            context.clone(),
            "CSML/basic_test/event.csml",
        );
        let messages: Value = message_to_json_value(msg);
        assert_eq!(
            Some("true"),
            messages["messages"][0]["content"]["text"].as_str(),
            "{ok_type} event type can't be use as string"
        );
    }
    // We should get 2 messages warning and null because we can't use string methods with this types
    for err_type in &err_types {
        let mut map = serde_json::Map::new();
        map.insert((*err_type).to_string(), Value::String("42".to_owned()));
        let msg = format_message(
            &Event::new("content_type", "content", Value::Object(map)),
            context.clone(),
            "CSML/basic_test/event.csml",
        );
        let messages: Value = message_to_json_value(msg);

        assert_eq!(
            "null",
            &messages["messages"][1]["content"]["text"].to_string(),
            "{err_type} event type is use as string"
        );
    }
}
