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

#[test]
fn test_jsonvalue_to_any_type() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Provider {
        pub process: String,
        pub provider: String,
        pub authorization_endpoint: String,
        pub token_endpoint: String,
        pub device_auth_endpoint: String,
        pub scopes: Vec<String>,
        pub client_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub client_secret: Option<String>,
    }
    let mut hash = HashMap::new();

    hash.insert(
        "process".to_string(),
        JsonValue::String("process name".to_string()),
    );
    hash.insert(
        "provider".to_string(),
        JsonValue::String("provider name".to_string()),
    );
    hash.insert(
        "authorization_endpoint".to_string(),
        JsonValue::String(
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
        ),
    );
    hash.insert(
        "token_endpoint".to_string(),
        JsonValue::String("https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string()),
    );
    hash.insert(
        "device_auth_endpoint".to_string(),
        JsonValue::String(
            "https://login.microsoftonline.com/common/oauth2/v2.0/devicecode".to_string(),
        ),
    );
    hash.insert(
        "scopes".to_string(),
        JsonValue::Vec(vec![
            JsonValue::String("offline_access".to_string()),
            JsonValue::String("https://outlook.office.com/SMTP.Send".to_string()),
            JsonValue::String("https://outlook.office.com/User.Read".to_string()),
        ]),
    );
    hash.insert(
        "client_id".to_string(),
        JsonValue::String("client-id-12345".to_string()),
    );
    hash.insert(
        "client_secret".to_string(),
        JsonValue::String("secret-12345".to_string()),
    );

    let expected = JsonValue::HashMap(hash);
    let provider = JsonValue::convert_to::<Provider>(&expected).unwrap();
    let result = JsonValue::convert_from(&provider).unwrap();

    assert_eq!(expected, result);
}
