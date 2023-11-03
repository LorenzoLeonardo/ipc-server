use std::{collections::HashMap, fmt::Display};

use serde_derive::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

use super::error::Error;

/// An object that is responsible in building an object registration
/// protocol stream in JSON format.
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterObject {
    pub reg_object: String,
}

impl RegisterObject {
    /// Creates a RegisterObject with the object name to be
    /// listed in the IPC server.
    pub fn new(name: &str) -> Self {
        Self {
            reg_object: name.to_string(),
        }
    }
    /// Converts this object into JSON bytes stream.
    pub fn serialize(self) -> Result<Vec<u8>, Error> {
        serde_json::to_vec(&self).map_err(|e| Error::new(JsonValue::String(e.to_string())))
    }
}

/// An object that is return after a successful registration,
/// successful subscription for listening events.
#[derive(Serialize, Deserialize, Debug)]
pub struct Success {
    pub success: String,
}

impl Success {
    /// Creates a Success object.
    pub fn new(success: &str) -> Self {
        Self {
            success: success.to_string(),
        }
    }
    /// Converts this object into JSON bytes stream.
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

/// A list of JsonValue type.
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
    /// Converts from any struct T that implements serde::Serialize trait into
    /// a JsonValue type.
    pub fn convert_from<T: serde::Serialize>(value: &T) -> Result<Self, Error> {
        let val = serde_json::to_string(&value)
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;
        let val: JsonValue =
            serde_json::from_str(&val).map_err(|e| Error::new(JsonValue::String(e.to_string())))?;
        Ok(val)
    }
    /// Converts from any JsonValue to any T that implements serde::Deserialize trait.
    pub fn convert_to<T: serde::de::DeserializeOwned>(value: &JsonValue) -> Result<T, Error> {
        let val = serde_json::to_string(value)
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;
        let val: T =
            serde_json::from_str(&val).map_err(|e| Error::new(JsonValue::String(e.to_string())))?;
        Ok(val)
    }
}

/// An object that is responsible in building a remote call method protocol in JSON stream.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallObjectRequest {
    pub object: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<JsonValue>,
}

impl CallObjectRequest {
    /// Creates a CallObjectRequest object.
    pub fn new(object: &str, method: &str, param: Option<JsonValue>) -> Self {
        Self {
            object: object.to_string(),
            method: method.to_string(),
            param,
        }
    }
    /// Converts this object into JSON bytes stream.
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

/// An object that is responsible in building a response protocol
/// of CallObjectRequest is JSON format.
#[derive(Serialize, Deserialize, Debug)]
pub struct CallObjectResponse {
    pub response: JsonValue,
}

impl CallObjectResponse {
    /// Creates a new CallObjectResponse object.
    pub fn new(response: JsonValue) -> Self {
        Self { response }
    }
    /// Converts this object into JSON bytes stream.
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

/// An object that is responsible in building a event result
/// in JSON format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub event: String,
    pub result: JsonValue,
}

impl Event {
    /// Create a new Event object.
    pub fn new(event: &str, result: JsonValue) -> Self {
        Self {
            event: event.to_string(),
            result,
        }
    }
    /// Converts this object into JSON bytes stream.
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

/// An object that is responsible in building a subscription protocol
/// to the IPC server to be able to listen future events in in JSON format.
#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeToEvent {
    pub event_name: String,
}

impl SubscribeToEvent {
    /// Creates a new SubscribeToEvent object.
    pub fn new(event_name: &str) -> Self {
        Self {
            event_name: event_name.to_string(),
        }
    }
    /// Converts this object into JSON bytes stream.
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

/// An object that is responsible in building a list of objects in
/// JSON format to determine if this objects are already available
/// for communication in the IPC server to other processes.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ListObjects {
    pub list: Vec<String>,
}

impl ListObjects {
    /// Creates a new ListObjects object.
    pub fn new(list: Vec<String>) -> Self {
        Self { list }
    }
}

/// A list of possible incoming messages to the client.
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IncomingMessage {
    Register(Success),
    Error(Error),
    CallRequest(CallObjectRequest),
    CallResponse(CallObjectResponse),
    WaitForObjects(ListObjects),
}

/// A list of possible outgoing messages from the client.
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
    /// Converts this object into JSON bytes stream.
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}

/// A list of common error string.
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
