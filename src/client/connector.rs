use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use super::error::Error;
use super::message::CallObjectRequest;

use crate::SERVER_ADDRESS;

#[derive(Clone, Debug)]
pub struct Connector {
    socket: Arc<Mutex<TcpStream>>,
}

impl Connector {
    pub async fn connect() -> Result<Self, Error> {
        let stream = TcpStream::connect(SERVER_ADDRESS)
            .await
            .map_err(|e| Error::Socket(e.to_string()))?;

        Ok(Self {
            socket: Arc::new(Mutex::new(stream)),
        })
    }

    pub async fn remote_call(
        &self,
        object: &str,
        method: &str,
        param: Option<HashMap<String, String>>,
    ) -> Result<Vec<u8>, Error> {
        let request = CallObjectRequest::new(object, method).parameters(param);

        let mut socket = self.socket.lock().await;

        socket
            .write_all(
                request
                    .serialize()
                    .map_err(|e| Error::Serde(e.to_string()))?
                    .as_slice(),
            )
            .await
            .map_err(|e| Error::Io(e.to_string()))?;

        let mut buf = Vec::new();
        let n = socket
            .read_buf(&mut buf)
            .await
            .map_err(|e| Error::Io(e.to_string()))?;

        if n == 0 {
            Err(Error::Io("remote connection error".to_string()))
        } else {
            Ok(buf)
        }
    }
}
