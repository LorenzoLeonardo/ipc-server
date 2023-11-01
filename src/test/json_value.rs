use std::collections::HashMap;

use ipc_client::client::message::JsonValue;

use serde_derive::{Deserialize, Serialize};

#[test]
fn test_json_value() {
    let val = JsonValue::Int32(-1234567890);
    assert_eq!(serde_json::to_string(&val).unwrap().as_str(), "-1234567890");

    let val = JsonValue::Uint32(1234567890);
    assert_eq!(serde_json::to_string(&val).unwrap().as_str(), "1234567890");

    let val = JsonValue::Int64(-1234567890);
    assert_eq!(serde_json::to_string(&val).unwrap().as_str(), "-1234567890");

    let val = JsonValue::Uint64(1234567890);
    assert_eq!(serde_json::to_string(&val).unwrap().as_str(), "1234567890");

    let val = JsonValue::Float(123456.7890);
    assert_eq!(serde_json::to_string(&val).unwrap().as_str(), "123456.789");

    let val = JsonValue::Bool(true);
    assert_eq!(serde_json::to_string(&val).unwrap().as_str(), "true");

    let val = JsonValue::String("the quick brown fox jumps over the lazy dog.".into());
    assert_eq!(
        serde_json::to_string(&val).unwrap().as_str(),
        r#""the quick brown fox jumps over the lazy dog.""#
    );

    let val = JsonValue::Vec(vec![
        JsonValue::String("54".into()),
        JsonValue::String("true".into()),
        JsonValue::String("always".into()),
    ]);
    println!("{}", serde_json::to_string(&val).unwrap().as_str());
    assert_eq!(
        serde_json::to_string(&val).unwrap().as_str(),
        r#"["54","true","always"]"#
    );

    let mut hash = HashMap::new();
    hash.insert("test key".into(), JsonValue::String("test val".to_string()));
    let val = JsonValue::HashMap(hash);
    println!("{}", serde_json::to_string(&val).unwrap().as_str());
    assert_eq!(
        serde_json::to_string(&val).unwrap().as_str(),
        r#"{"test key":"test val"}"#
    );
}

#[test]
fn test_combo() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Error(JsonValue);

    let val = Error(JsonValue::Bool(true));

    println!("{}", serde_json::to_string(&val).unwrap().as_str());
    assert_eq!(serde_json::to_string(&val).unwrap().as_str(), "true");

    #[derive(Serialize, Deserialize, Debug)]
    struct Error2 {
        error: JsonValue,
    }

    let val = Error2 {
        error: JsonValue::Bool(true),
    };

    println!("{}", serde_json::to_string(&val).unwrap().as_str());
    assert_eq!(
        serde_json::to_string(&val).unwrap().as_str(),
        r#"{"error":true}"#
    );
}
