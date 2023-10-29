use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Error {
    error: String,
}

impl Error {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
        }
    }

    pub fn serialize(self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(&self)
    }
}
