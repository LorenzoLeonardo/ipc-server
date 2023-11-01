use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use super::error::Error;
use super::message::{CallObjectRequest, JsonValue};

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
        param: HashMap<String, JsonValue>,
    ) -> Result<Vec<u8>, Error> {
        let mut request = CallObjectRequest::new(object, method);
        for (key, value) in param.iter() {
            request = request.parameter(key, value.clone());
        }

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

        let mut buf = [0u8; u16::MAX as usize];
        let n = socket
            .read(&mut buf)
            .await
            .map_err(|e| Error::Io(e.to_string()))?;

        if n == 0 {
            Err(Error::Io("remote connection error".to_string()))
        } else {
            Ok(buf[0..n].to_vec())
        }
    }
}
