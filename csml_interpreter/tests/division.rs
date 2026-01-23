mod support;

use csml_interpreter::data::context::Context;
use csml_interpreter::data::event::Event;
use std::collections::HashMap;

use crate::support::tools::format_message;
use crate::support::tools::message_to_json_value;

use csml_interpreter::error_format::{ERROR_OPS_DIV_INT, OVERFLOWING_OPERATION};
use serde_json::Value;

#[test]
fn ok_division() {
    let data = r#"{"messages":[ {"content":{"text":"2"},"content_type":"text"}],"memories":[]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "start",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn ok_division_2() {
    let data = r#"{"messages":[ {"content":{"text":"21.333333333333332"},"content_type":"text"}],"memories":[]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "div2",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

// fn check_error_component(vec: &[Message]) -> bool {
//     let comp = &vec[0];

//     return comp.content.is_object();
// }

#[test]
fn ok_division_3() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "div3",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    println!("{msg:#?}");
    assert!(
        msg.messages
            .first()
            .unwrap()
            .content
            .get("error")
            .unwrap()
            .as_str()
            .unwrap()
            .contains(ERROR_OPS_DIV_INT)
    );
}

#[test]
fn ok_division_overflow() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "div_overflow",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    println!("{msg:#?}");
    assert!(
        msg.messages
            .first()
            .unwrap()
            .content
            .get("error")
            .unwrap()
            .as_str()
            .unwrap()
            .contains(OVERFLOWING_OPERATION)
    );
}

////////////////////////////////////////////////////////////////////////////////
/// ARRAY
////////////////////////////////////////////////////////////////////////////////

#[test]
fn division_array_step_0() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "simple_0",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_array_step_1() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_array_step_1",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_array_step_2() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_array_step_2",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_array_step_3() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_array_step_3",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_array_step_4() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_array_step_4",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_array_step_5() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_array_step_5",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_array_step_6() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_array_step_6",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

////////////////////////////////////////////////////////////////////////////////
/// BOOLEAN
////////////////////////////////////////////////////////////////////////////////

#[test]
fn division_boolean_step_0() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "simple_0",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_boolean_step_1() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_boolean_step_1",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_boolean_step_2() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_boolean_step_2",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_boolean_step_3() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_boolean_step_3",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_boolean_step_4() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_boolean_step_4",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_boolean_step_5() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_boolean_step_5",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_boolean_step_6() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_boolean_step_6",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

////////////////////////////////////////////////////////////////////////////////
/// FLOAT
////////////////////////////////////////////////////////////////////////////////

#[test]
fn division_float_step_0() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "simple_0",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_float_step_1() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_float_step_1",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_float_step_2() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "1"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_float_step_2",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn division_float_step_3() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "1"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_float_step_3",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn division_float_step_4() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_float_step_4",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_float_step_5() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_float_step_5",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_float_step_6() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "1"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_float_step_6",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

/////////////////////////////////////////////////////////////////////////////////
/// INT
////////////////////////////////////////////////////////////////////////////////

#[test]
fn division_int_step_0() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "simple_0",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_int_step_1() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_int_step_1",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_int_step_2() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "1"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_int_step_2",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn division_int_step_3() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "1"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_int_step_3",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn division_int_step_4() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_int_step_4",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_int_step_5() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_int_step_5",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_int_step_6() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "1"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_int_step_6",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

////////////////////////////////////////////////////////////////////////////////
/// NULL
////////////////////////////////////////////////////////////////////////////////

#[test]
fn division_null_step_0() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "simple_0",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_null_step_1() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_null_step_1",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_null_step_2() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_null_step_2",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_null_step_3() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_null_step_3",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_null_step_4() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_null_step_4",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_null_step_5() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_null_step_5",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_null_step_6() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_null_step_6",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

////////////////////////////////////////////////////////////////////////////////
/// OBJECT
////////////////////////////////////////////////////////////////////////////////

#[test]
fn division_object_step_0() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "simple_0",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_object_step_1() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_object_step_1",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_object_step_2() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_object_step_2",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_object_step_3() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_object_step_3",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_object_step_4() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_object_step_4",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_object_step_5() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_object_step_5",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_object_step_6() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_object_step_6",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

/////////////////////////////////////////////////////////////////////////////////
/// STRING
////////////////////////////////////////////////////////////////////////////////

#[test]
fn division_string_step_0() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "simple_0",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_string_step_1() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_string_step_1",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_string_step_2() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "1"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_string_step_2",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn division_string_step_3() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "1"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_string_step_3",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}

#[test]
fn division_string_step_4() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_string_step_4",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_string_step_5() {
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_string_step_5",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let value: Value = message_to_json_value(msg.clone());

    if let Some(value) = value.get("messages") {
        if let Some(value) = value.get(0) {
            if let Some(value) = value.get("content_type") {
                if value == "error" {
                    return;
                }
            }
        }
    }

    println!("{value:#?}");

    panic!("unexpected result")
}

#[test]
fn division_string_step_6() {
    let data = r#"{
        "memories":[
        ],
        "messages":[
            {"content":{"text": "1"}, "content_type":"text"}
        ]}"#;
    let msg = format_message(
        &Event::new("payload", "", serde_json::json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "division_string_step_6",
            "flow",
            None,
        ),
        "CSML/basic_test/numerical_operation/division.csml",
    );

    let v1: Value = message_to_json_value(msg);
    let v2: Value = serde_json::from_str(data).unwrap();

    assert_eq!(v1, v2);
}
