use std::future::Future;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, MutexGuard};

use super::error::Error;
use super::message::{
    CallObjectRequest, Event, IncomingMessage, JsonValue, StaticReplies, SubscribeToEvent,
};

use crate::{CHUNK_SIZE, ENV_SERVER_ADDRESS, SERVER_ADDRESS};

/// An object that is responsible for remote object method calls,
/// sending events and listening for incoming events.
#[derive(Clone, Debug)]
pub struct Connector {
    socket: Arc<Mutex<TcpStream>>,
}

impl Connector {
    /// Connects to the IPC server.
    pub async fn connect() -> Result<Self, Error> {
        let server_address = std::env::var(ENV_SERVER_ADDRESS).unwrap_or(SERVER_ADDRESS.to_owned());
        let stream = TcpStream::connect(server_address)
            .await
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;

        Ok(Self {
            socket: Arc::new(Mutex::new(stream)),
        })
    }

    /// Calls shared object methods from other processes.
    /// It has an optional parameters, the value is in JsonValue type.
    pub async fn remote_call(
        &self,
        object: &str,
        method: &str,
        param: Option<JsonValue>,
    ) -> Result<JsonValue, Error> {
        let request = CallObjectRequest::new(object, method, param);

        let mut socket = self.socket.lock().await;

        socket
            .write_all(
                request
                    .serialize()
                    .map_err(|e| Error::new(JsonValue::String(e.to_string())))?
                    .as_slice(),
            )
            .await
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;

        let mut buf = Vec::new();
        let n = read(&mut socket, &mut buf)
            .await
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;

        if n == 0 {
            Err(Error::new(JsonValue::String(
                StaticReplies::RemoteConnectionError.to_string(),
            )))
        } else {
            let result: IncomingMessage = serde_json::from_slice(&buf[0..n])
                .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;
            if let IncomingMessage::CallResponse(response) = result {
                log::trace!("Response: {:?}", response);
                Ok(response.response)
            } else if let IncomingMessage::Error(err) = result {
                Err(err)
            } else {
                Err(Error::new(JsonValue::String(
                    StaticReplies::InvalidResponseData.to_string(),
                )))
            }
        }
    }

    /// Sends the event to the ipc-server and let the ipc-server
    /// boadcast the message to all subscribed processes.
    /// Parameters in JsonValue type.
    pub async fn send_event(&self, event: &str, result: JsonValue) -> Result<(), Error> {
        let request = Event::new(event, result);

        let mut socket = self.socket.lock().await;

        socket
            .write_all(
                request
                    .serialize()
                    .map_err(|e| Error::new(JsonValue::String(e.to_string())))?
                    .as_slice(),
            )
            .await
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;
        Ok(())
    }

    /// Subscribes and listens for incoming events from other processes.
    pub async fn listen_for_event<
        F: Future<Output = Result<(), RE>> + Send,
        RE: std::error::Error + 'static + Send,
        T: Fn(JsonValue) -> F + Send + Sync + 'static,
    >(
        &self,
        event_name: &str,
        callback: T,
    ) -> Result<(), Error> {
        let request = SubscribeToEvent::new(event_name);

        let mut socket = self.socket.lock().await;

        socket
            .write_all(
                request
                    .serialize()
                    .map_err(|e| Error::new(JsonValue::String(e.to_string())))?
                    .as_slice(),
            )
            .await
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;

        let socket = self.socket.clone();

        tokio::spawn(async move {
            loop {
                let mut socket = socket.lock().await;

                let mut buf = Vec::new();
                let n = read(&mut socket, &mut buf)
                    .await
                    .map_err(|e| Error::new(JsonValue::String(e.to_string())))
                    .unwrap_or_else(|e| {
                        log::error!("{:?}", e);
                        0
                    });

                let value: JsonValue = serde_json::from_slice(&buf[0..n])
                    .map_err(|e| Error::new(JsonValue::String(e.to_string())))
                    .unwrap_or_else(|e| {
                        log::error!("{:?}", e);
                        JsonValue::Bool(false)
                    });

                log::trace!("{:?}", &value);
                match callback(value).await {
                    Ok(_) => {
                        continue;
                    }
                    Err(err) => {
                        log::error!("{err:?}");
                        break;
                    }
                }
            }
        });
        Ok(())
    }
}

pub async fn read(
    socket: &mut MutexGuard<'_, TcpStream>,
    data: &mut Vec<u8>,
) -> std::io::Result<usize> {
    loop {
        let mut buffer = [0u8; CHUNK_SIZE];
        match socket.read(&mut buffer).await {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    return Ok(bytes_read);
                }
                data.extend_from_slice(&buffer[0..bytes_read]);

                if bytes_read < CHUNK_SIZE {
                    return Ok(data.len());
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
