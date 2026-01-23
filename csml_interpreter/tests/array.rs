mod support;

use csml_interpreter::data::context::Context;
use csml_interpreter::data::event::Event;
use std::collections::HashMap;

use crate::support::tools::format_message;
use crate::support::tools::message_to_json_value;

use csml_interpreter::data::Interval;
use csml_interpreter::data::primitive::PrimitiveObject;
use csml_interpreter::interpreter::json_to_literal;
use serde_json::{Value, json};

#[test]
fn array_step_0() {
    let data =
        r#"{"memories":[{"key":"vec", "value":[]}, {"key":"vec", "value": [42]}], "messages":[]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_0",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_step_1() {
    let data = r#"
    {
        "memories":[{"key":"vec", "value": [42]}, {"key":"vec", "value": []}],
        "messages":[
            {"content":{"text": "42"}, "content_type":"text"},
            {"content":[], "content_type":"array"}
        ]
    }
    "#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_1",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_step_2() {
    let data = r#"
        {
            "memories":[{"key":"vec", "value": [42]}, {"key":"vec", "value": []}],
            "messages":[{"content":{"text": "false"},"content_type":"text"}, {"content":{"text": "true"}, "content_type":"text"}]
        }"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_2",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_step_3() {
    let data = r#"
        {
            "memories":[{"key":"vec", "value": [42]}, {"key":"vec", "value": [24, 42]}, {"key":"vec", "value": [42]}],
            "messages":[{"content":{"text": "2"}, "content_type":"text"}, {"content":{"text": "24"}, "content_type":"text"}, {"content":{"text": "42"}, "content_type":"text"}]
        }"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_3",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_step_4() {
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_4",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    assert_eq!(msg.messages[0].content_type, "error");
}

#[test]
fn array_step_5() {
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_5",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    eprintln!("=> {:?}", msg.messages);
    assert_eq!(msg.messages[0].content_type, "error");
}

#[test]
fn array_step_6() {
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_6",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    assert_eq!(msg.messages[0].content_type, "error");
}

#[test]
fn array_step_7() {
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_7",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    assert_eq!(msg.messages[0].content_type, "error");
}

#[test]
fn array_step_8() {
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_8",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    assert_eq!(msg.messages[0].content_type, "error");
}

#[test]
fn array_step_9() {
    let data = r#"{"memories":[], "messages":[{"content":{"text":""}, "content_type":"text"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_9",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_step_10() {
    let data = r#"{"memories":[], "messages":[{"content":{"text":"1"}, "content_type":"text"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_10",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_step_11() {
    let data = r#"{"memories":[], "messages":[{"content":{"text":"1,2"}, "content_type":"text"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "step_11",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_index_of_0() {
    let data = r#"{"memories":[], "messages":[{"content":{"text":"-1"}, "content_type":"text"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_index_of_0",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_index_of_1() {
    let data = r#"{"memories":[], "messages":[{"content":{"text":"1"}, "content_type":"text"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_index_of_1",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_index_of_2() {
    let data = r#"{"memories":[], "messages":[{"content":{"text":"2"}, "content_type":"text"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_index_of_2",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_find_0() {
    let data = r#"{"memories":[], "messages":[{"content":[], "content_type":"array"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_find_0",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_find_1() {
    let data = r#"{"memories":[], "messages":[{"content":[2, 2], "content_type":"array"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_find_1",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_find_2() {
    let data =
        r#"{"memories":[], "messages":[{"content":[{"obj":"42"}], "content_type":"array"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_find_2",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_map() {
    let data = r#"{"memories":[], "messages":[{"content":[2, 3], "content_type":"array"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_map",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_reduce() {
    let data = r#"{"memories":[], "messages":[{"content":{"text":"10"}, "content_type":"text"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_reduce",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_filter() {
    let data = r#"{"memories":[], "messages":[{"content":[1, 3, 5], "content_type":"array"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_filter",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_map_index() {
    let data =
        r#"{"memories":[], "messages":[{"content":[0, 1, 2, 3, 4], "content_type":"array"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_map_index",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_filter_index() {
    let data = r#"{"memories":[], "messages":[{"content":[34, 232], "content_type":"array"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_filter_index",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_reduce_index() {
    let data = r#"{"memories":[], "messages":[{"content":{"text":"6"}, "content_type":"text"}]}"#;
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_reduce_index",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn array_reverse() {
    let data = json!({"memories":[],
    "messages":[
        {"content":[1,2,3,4,5], "content_type":"array"},
        {"content":[5,4,3,2,1], "content_type":"array"}
    ]});
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_reverse",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);

    assert_eq!(v1, data);
}

#[test]
fn array_reverse_memories() {
    let data = json!({"memories":[
        {"key":"arr", "value":[1,2,3,4,5]},
        {"key":"arr_rev", "value":[5,4,3,2,1]}
    ],
    "messages":[
        {"content":[1,2,3,4,5], "content_type":"array"},
        {"content":[5,4,3,2,1], "content_type":"array"}
    ]});
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_reverse_memories",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);

    assert_eq!(v1, data);
}

#[test]
fn array_reverse_metadata() {
    let data = json!({"memories":[],
    "messages":[
        {"content":[1,2,3,4,5], "content_type":"array"},
        {"content":[5,4,3,2,1], "content_type":"array"}
    ]});
    let metadata = json!({
        "foo": "bar",
        "test": {
            "arr": [1,2,3,4,5]
        }
    });
    let lit = json_to_literal(&metadata, Interval::default(), "").unwrap();
    let metadata = lit
        .primitive
        .as_any()
        .downcast_ref::<PrimitiveObject>()
        .unwrap()
        .value
        .clone();
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            metadata,
            None,
            None,
            "array_reverse_metadata",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);

    assert_eq!(v1, data);
}

#[test]
fn array_reverse_metadata_string() {
    let data = json!({"memories":[],
    "messages":[
        {"content": {"text": "[1,2,3,4,5]"}, "content_type":"text"},
        {"content": {"text": "[5,4,3,2,1]"}, "content_type":"text"}
    ]});
    let metadata = json!({
        "foo": "bar",
        "test": {
            "arr": [1,2,3,4,5]
        }
    });
    let lit = json_to_literal(&metadata, Interval::default(), "").unwrap();
    let metadata = lit
        .primitive
        .as_any()
        .downcast_ref::<PrimitiveObject>()
        .unwrap()
        .value
        .clone();
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            metadata,
            None,
            None,
            "array_reverse_metadata_string",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);

    assert_eq!(v1, data);
}

#[test]
fn array_equals() {
    let data = json!({"memories":[],
    "messages":[
        {"content": {"text": "equal"}, "content_type":"text"},
    ]});
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_equals",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);

    assert_eq!(v1, data);
}

#[test]
fn array_not_equal() {
    let data = json!({"memories":[],
    "messages":[
        {"content": {"text": "not equal"}, "content_type":"text"},
    ]});
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "array_not_equal",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/array.csml",
    );

    let v1: Value = message_to_json_value(msg);

    assert_eq!(v1, data);
}
