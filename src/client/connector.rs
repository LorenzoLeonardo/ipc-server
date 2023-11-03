use std::future::Future;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use super::error::Error;
use super::message::{
    CallObjectRequest, Event, IncomingMessage, JsonValue, StaticReplies, SubscribeToEvent,
};

use crate::SERVER_ADDRESS;

#[derive(Clone, Debug)]
pub struct Connector {
    socket: Arc<Mutex<TcpStream>>,
}

impl Connector {
    pub async fn connect() -> Result<Self, Error> {
        let stream = TcpStream::connect(SERVER_ADDRESS)
            .await
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;

        Ok(Self {
            socket: Arc::new(Mutex::new(stream)),
        })
    }

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

        let mut buf = [0u8; u16::MAX as usize];
        let n = socket
            .read(&mut buf)
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

    /// Send the event to the ipc-server and let ipc-server boadcast the message
    /// to all subscribed processes.
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
            let mut socket = socket.lock().await;

            let mut buf = [0u8; u16::MAX as usize];
            let n = socket
                .read(&mut buf)
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
            callback(value).await.unwrap_or_else(|e| {
                log::error!("{:?}", e);
            });
        });
        Ok(())
    }
}
