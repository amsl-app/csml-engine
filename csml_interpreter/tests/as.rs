mod support;

use csml_interpreter::data::ast::Flow;
use csml_interpreter::error_format::ErrorInfo;
use csml_interpreter::parser::parse_flow;

use support::tools::read_file;

fn format_message(filepath: String) -> Result<Flow, ErrorInfo> {
    let text = read_file(filepath).unwrap();

    parse_flow(&text, "Test")
}

#[test]
fn as_0() {
    let result = format_message("CSML/basic_test/syntax/as/as_0.csml".to_owned()).is_ok();

    assert!(result);
}

#[test]
fn as_1() {
    let result = format_message("CSML/basic_test/syntax/as/as_1.csml".to_owned()).is_err();

    assert!(result);
}

#[test]
fn as_2() {
    let result = format_message("CSML/basic_test/syntax/as/as_2.csml".to_owned()).is_err();

    assert!(result);
}

#[test]
fn as_3() {
    let result = format_message("CSML/basic_test/syntax/as/as_3.csml".to_owned()).is_err();

    assert!(result);
}

#[test]
fn as_4() {
    let result = format_message("CSML/basic_test/syntax/as/as_4.csml".to_owned()).is_err();

    assert!(result);
}

#[test]
fn as_5() {
    let result = format_message("CSML/basic_test/syntax/as/as_5.csml".to_owned()).is_err();

    assert!(result);
}

#[test]
fn as_6() {
    let result = format_message("CSML/basic_test/syntax/as/as_6.csml".to_owned()).is_err();

    assert!(result);
}
