use std::fmt::Display;

use json_elem::jsonelem::JsonElem;
use serde_derive::{Deserialize, Serialize};

/// An object that is responsible to house error in JsonElem type
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Error {
    error: JsonElem,
}

impl Error {
    /// Creates an Error object in JsonElem
    pub fn new(error: JsonElem) -> Self {
        Self { error }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for Error {}
