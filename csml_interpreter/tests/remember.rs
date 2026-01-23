mod support;

use csml_interpreter::data::ast::Flow;
use csml_interpreter::error_format::ErrorInfo;
use csml_interpreter::parser::parse_flow;

use support::tools::read_file;

fn format_message(filepath: String) -> Result<Flow, ErrorInfo> {
    let text = read_file(filepath).unwrap();

    parse_flow(&text, "Test")
}

////////////////////////////////////////////////////////////////////////////////
/// REMEMBER VALID SYNTAX
////////////////////////////////////////////////////////////////////////////////

#[test]
fn remember_0() {
    let result =
        format_message("CSML/basic_test/syntax/remember/remember_0.csml".to_owned()).is_ok();

    assert!(result);
}

#[test]
fn remember_1() {
    let result =
        format_message("CSML/basic_test/syntax/remember/remember_1.csml".to_owned()).is_ok();

    assert!(result);
}

#[test]
fn remember_2() {
    let result =
        format_message("CSML/basic_test/syntax/remember/remember_2.csml".to_owned()).is_ok();

    assert!(result);
}

#[test]
fn remember_3() {
    let result =
        format_message("CSML/basic_test/syntax/remember/remember_3.csml".to_owned()).is_ok();

    assert!(result);
}

////////////////////////////////////////////////////////////////////////////////
/// USE INVALID SYNTAX
////////////////////////////////////////////////////////////////////////////////

#[test]
fn remember_4() {
    let result =
        format_message("CSML/basic_test/syntax/remember/remember_4.csml".to_owned()).is_err();

    assert!(result);
}

#[test]
fn remember_5() {
    let result =
        format_message("CSML/basic_test/syntax/remember/remember_5.csml".to_owned()).is_err();

    assert!(result);
}

#[test]
fn remember_6() {
    let result =
        format_message("CSML/basic_test/syntax/remember/remember_6.csml".to_owned()).is_err();

    assert!(result);
}
