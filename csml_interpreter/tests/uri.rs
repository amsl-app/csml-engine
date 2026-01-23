mod support;

use csml_interpreter::data::context::Context;
use csml_interpreter::data::event::Event;
use std::collections::HashMap;

use crate::support::tools::format_message;
use crate::support::tools::message_to_json_value;

use serde_json::{Value, json};

#[test]
fn ok_encode_uri() {
    let data = json! (
        {
            "messages": [
                {
                    "content": {
                        "text": "file:///a?foo%20bar",
                    },
                    "content_type": "text"
                },
                {
                    "content": {
                        "text": "file:///a?foo%20bar=x%20y",
                    },
                    "content_type": "text"
                },
                {
                    "content": {
                        "text": "file:///a?foo%20bar=x%20y&v=%25&i=%20%20&j=%20,%20n",
                    },
                    "content_type": "text"
                }
            ],
            "memories": []
        }
    );
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "encode_uri",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/uri.csml",
    );

    let v1: Value = message_to_json_value(msg);

    assert_eq!(v1, data);
}

#[test]
fn ok_decode_uri() {
    let data = json! (
        {
            "messages": [
                {
                    "content": {
                        "text": "file:///a?foo bar",
                    },
                    "content_type": "text"
                },
                {
                    "content": {
                        "text": "file:///a?foo bar=x y",
                    },
                    "content_type": "text"
                },
                {
                    "content": {
                        "text": "file:///a?foo bar=x y&v=%&i=  &j= , n",
                    },
                    "content_type": "text"
                }
            ],
            "memories": []
        }
    );
    let msg = format_message(
        &Event::new("payload", "", json!({})),
        Context::new(
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            "decode_uri",
            "flow",
            None,
        ),
        "CSML/basic_test/stdlib/uri.csml",
    );

    let v1: Value = message_to_json_value(msg);

    assert_eq!(v1, data);
}
