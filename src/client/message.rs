use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterObject {
    reg_object: String,
}

impl RegisterObject {
    pub fn new(name: &str) -> Self {
        Self {
            reg_object: name.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Success {
    success: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    error: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallObjectRequest {
    pub object: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<HashMap<String, String>>,
}

impl CallObjectRequest {
    pub fn new(object: &str, method: &str) -> Self {
        Self {
            object: object.to_string(),
            method: method.to_string(),
            param: None,
        }
    }

    pub fn parameter(mut self, key: &str, value: &str) -> Self {
        if self.param.is_none() {
            self.param = Some(HashMap::new());
        }
        if let Some(param) = &mut self.param {
            param.insert(key.to_owned(), value.to_owned());
        }
        self
    }

    pub fn parameters(mut self, param: Option<HashMap<String, String>>) -> Self {
        self.param = param;
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
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum IncomingMessage {
    Register(Success),
    Error(Error),
    CallRequest(CallObjectRequest),
    CallResponse(CallObjectResponse),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum OutgoingMessage {
    Register(RegisterObject),
    Error(Error),
    CallRequest(CallObjectRequest),
    CallResponse(CallObjectResponse),
}
