use json_elem::jsonelem::JsonElem;
use serde_derive::{Deserialize, Serialize};

/// An object that is responsible to house error in JsonElem type
#[derive(Serialize, Deserialize)]
pub struct Error {
    error: JsonElem,
}

impl Error {
    /// Creates an Error object in JsonElem
    pub fn new(error: JsonElem) -> Self {
        Self { error }
    }
    /// Converts this object into JSON byte format.
    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}
