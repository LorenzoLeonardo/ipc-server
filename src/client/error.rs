use std::fmt::Display;

use serde_derive::{Deserialize, Serialize};

use super::message::JsonValue;

/// An object that is responsible to house error in JsonValue type
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Error {
    error: JsonValue,
}

impl Error {
    /// Creates an Error object in JsonValue
    pub fn new(error: JsonValue) -> Self {
        Self { error }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}
