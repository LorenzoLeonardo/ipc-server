use serde_derive::{Deserialize, Serialize};

/// An object that is responsible to house error in JsonValue type
#[derive(Serialize, Deserialize)]
pub struct Error {
    error: String,
}

impl Error {
    /// Creates an Error object in JsonValue
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }
    /// Converts this object into JSON byte format.
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}
