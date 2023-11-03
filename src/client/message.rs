use std::{collections::HashMap, fmt::Display};

use serde_derive::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

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

    pub fn serialize(self) -> Result<Vec<u8>, Error> {
        serde_json::to_vec(&self).map_err(|e| Error::new(JsonValue::String(e.to_string())))
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Error {
    error: JsonValue,
}

impl Error {
    pub fn new(error: JsonValue) -> Self {
        Self { error }
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

impl Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonValue::Int32(val) => write!(f, "{}", val),
            JsonValue::Uint32(val) => write!(f, "{}", val),
            JsonValue::Int64(val) => write!(f, "{}", val),
            JsonValue::Uint64(val) => write!(f, "{}", val),
            JsonValue::Float(val) => write!(f, "{}", val),
            JsonValue::Bool(val) => write!(f, "{}", val),
            JsonValue::String(val) => write!(f, "{}", val),
            JsonValue::Vec(val) => write!(f, "{:?}", val),
            JsonValue::HashMap(val) => write!(f, "{:?}", val),
        }
    }
}

impl JsonValue {
    pub fn try_from<T: serde::Serialize>(value: T) -> Result<Self, Error> {
        let val = serde_json::to_string(&value)
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;
        let val: JsonValue =
            serde_json::from_str(&val).map_err(|e| Error::new(JsonValue::String(e.to_string())))?;
        Ok(val)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallObjectRequest {
    pub object: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<JsonValue>,
}

impl CallObjectRequest {
    pub fn new(object: &str, method: &str, param: Option<JsonValue>) -> Self {
        Self {
            object: object.to_string(),
            method: method.to_string(),
            param,
        }
    }

    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallObjectResponse {
    pub response: JsonValue,
}

impl CallObjectResponse {
    pub fn new(response: JsonValue) -> Self {
        Self { response }
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
    #[strum(serialize = "remote connection error")]
    RemoteConnectionError,
    #[strum(serialize = "invalid response data")]
    InvalidResponseData,
}
