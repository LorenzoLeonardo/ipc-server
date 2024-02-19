use json_elem::jsonelem::JsonElem;
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
        serde_json::to_vec(&self).map_err(|e| Error::new(JsonElem::String(e.to_string())))
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

/// An object that is responsible in building a remote call method protocol in JSON stream.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallObjectRequest {
    pub object: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<JsonElem>,
}

impl CallObjectRequest {
    /// Creates a CallObjectRequest object.
    pub fn new(object: &str, method: &str, param: Option<JsonElem>) -> Self {
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
    pub response: JsonElem,
}

impl CallObjectResponse {
    /// Creates a new CallObjectResponse object.
    pub fn new(response: JsonElem) -> Self {
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
    pub result: JsonElem,
}

impl Event {
    /// Create a new Event object.
    pub fn new(event: &str, result: JsonElem) -> Self {
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
