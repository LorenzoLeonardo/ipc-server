use std::{collections::HashMap, fmt::Display};

use serde_derive::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

use super::error;

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterObject {
    pub reg_object: String,
}

impl RegisterObject {
    pub fn new(name: &str) -> Self {
        Self {
            reg_object: name.to_string(),
        }
    }

    pub fn serialize(self) -> Result<Vec<u8>, error::Error> {
        serde_json::to_vec(&self).map_err(|e| error::Error::Serde(e.to_string()))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Success {
    pub success: String,
}

impl Success {
    pub fn new(success: &str) -> Self {
        Self {
            success: success.to_string(),
        }
    }

    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    error: String,
}

impl Error {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum JsonValue {
    Int32(i32),
    Uint32(u32),
    Int64(i64),
    Uint64(u64),
    Float(f64),
    Bool(bool),
    String(String),
    Vec(Vec<JsonValue>),
    HashMap(HashMap<String, JsonValue>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallObjectRequest {
    pub object: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<HashMap<String, JsonValue>>,
}

impl CallObjectRequest {
    pub fn new(object: &str, method: &str) -> Self {
        Self {
            object: object.to_string(),
            method: method.to_string(),
            param: None,
        }
    }

    pub fn parameter(mut self, key: &str, value: JsonValue) -> Self {
        if self.param.is_none() {
            self.param = Some(HashMap::new());
        }

        if let Some(param) = &mut self.param {
            param.insert(key.to_owned(), value);
        }

        self
    }

    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallObjectResponse {
    pub response: String,
}

impl CallObjectResponse {
    pub fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }

    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub event: String,
    pub result: JsonValue,
}

impl Event {
    pub fn new(event: &str, result: JsonValue) -> Self {
        Self {
            event: event.to_string(),
            result,
        }
    }

    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeToEvent {
    pub event_name: String,
}

impl SubscribeToEvent {
    pub fn new(event_name: &str) -> Self {
        Self {
            event_name: event_name.to_string(),
        }
    }

    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ListObjects {
    pub list: Vec<String>,
}

impl ListObjects {
    pub fn new(list: Vec<String>) -> Self {
        Self { list }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IncomingMessage {
    Register(Success),
    Error(Error),
    CallRequest(CallObjectRequest),
    CallResponse(CallObjectResponse),
    WaitForObjects(ListObjects),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum OutgoingMessage {
    Error(Error),
    CallResponse(CallObjectResponse),
    WaitForObjects(ListObjects),
    SendEvent(Event),
    SubscribeEvent(SubscribeToEvent),
}

impl OutgoingMessage {
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[derive(Debug, EnumString, Display, AsRefStr)]
pub enum StaticReplies {
    #[strum(serialize = "OK")]
    Ok,
    #[strum(serialize = "Object not found")]
    ObjectNotFound,
    #[strum(serialize = "client connection error")]
    ClientConnectionError,
    #[strum(serialize = "server connection error")]
    ServerConnectionError,
    #[strum(serialize = "serde parsing error")]
    SerdeParseError,
}
