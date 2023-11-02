#[derive(Debug)]
pub enum Error {
    Socket(String),
    Serde(String),
    Io(String),
    InvalidData(String),
    Other(String),
}
