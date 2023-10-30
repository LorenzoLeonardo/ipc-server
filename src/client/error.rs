#[derive(Debug)]
pub enum Error {
    Socket(String),
    Serde(String),
    Io(String),
    Other(String),
}
