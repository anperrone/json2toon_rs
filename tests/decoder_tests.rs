use json2toon_rs::{decode, DecoderOptions};
use serde_json::json;

fn default_opts() -> DecoderOptions {
    DecoderOptions {
        indent: 2,
        strict: true,
    }
}

#[test]
fn decode_single_primitive_root() {
    let json = decode("42", &default_opts()).unwrap();
    assert_eq!(json, json!(42));

    let json = decode("true", &default_opts()).unwrap();
    assert_eq!(json, json!(true));

    let json = decode("\"hello\"", &default_opts()).unwrap();
    assert_eq!(json, json!("hello"));
}

#[test]
fn decode_simple_object() {
    let input = "name: Alice\nage: 30";
    let value = decode(input, &default_opts()).unwrap();
    assert_eq!(value, json!({"name": "Alice", "age": 30}));
}

#[test]
fn decode_nested_object() {
    let input = "user:\n  name: Alice\n  age: 30\n  active: true";
    let value = decode(input, &default_opts()).unwrap();
    assert_eq!(
        value,
        json!({"user": {"name": "Alice", "age": 30, "active": true}})
    );
}

#[test]
fn decode_tabular_array() {
    let input = "users[2]{id,name,active}:\n  1,Alice,true\n  2,Bob,false";
    let value = decode(input, &default_opts()).unwrap();
    assert_eq!(
        value,
        json!({
            "users": [
                {"id": 1, "name": "Alice", "active": true},
                {"id": 2, "name": "Bob", "active": false}
            ]
        })
    );
}

#[test]
fn decode_inline_array_in_object() {
    let input = "tags[3]: one,two,three";
    let value = decode(input, &default_opts()).unwrap();
    assert_eq!(value, json!({"tags": ["one", "two", "three"]}));
}

#[test]
fn decode_list_array_multiline() {
    let input = "[3]:\n  - one\n  - two\n  - three";
    let value = decode(input, &default_opts()).unwrap();
    assert_eq!(value, json!(["one", "two", "three"]));
}

#[test]
fn decode_invalid_indentation_strict() {
    let input = "key:\n   bad_indent: 1"; // 3 spaces instead of multiple of 2
    let err = decode(input, &default_opts()).unwrap_err();
    match err {
        json2toon_rs::DecodeError::InvalidIndentation { line } => assert_eq!(line, 2),
        _ => panic!("expected InvalidIndentation error"),
    }
}

#[test]
fn decode_array_length_mismatch_inline() {
    let input = "tags[2]: one,two,three";
    let err = decode(input, &default_opts()).unwrap_err();
    match err {
        json2toon_rs::DecodeError::ArrayLengthMismatch { expected, found } => {
            assert_eq!(expected, 2);
            assert_eq!(found, 3);
        }
        _ => panic!("expected ArrayLengthMismatch error"),
    }
}

#[test]
fn decode_row_width_mismatch_tabular() {
    let input = "users[2]{id,name}:\n  1,Alice\n  2"; // second row has too few columns
    let err = decode(input, &default_opts()).unwrap_err();
    match err {
        json2toon_rs::DecodeError::RowWidthMismatch {
            line,
            expected,
            found,
        } => {
            assert_eq!(line, 3);
            assert_eq!(expected, 2);
            assert_eq!(found, 1);
        }
        _ => panic!("expected RowWidthMismatch error"),
    }
}

#[test]
fn decode_array_length_mismatch_tabular_rows() {
    let input = "users[3]{id,name}:\n  1,Alice\n  2,Bob"; // only 2 rows instead of 3
    let err = decode(input, &default_opts()).unwrap_err();
    match err {
        json2toon_rs::DecodeError::ArrayLengthMismatch { expected, found } => {
            assert_eq!(expected, 3);
            assert_eq!(found, 2);
        }
        _ => panic!("expected ArrayLengthMismatch error"),
    }
}
