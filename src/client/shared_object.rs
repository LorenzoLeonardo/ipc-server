use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::client::message::CallObjectResponse;
use crate::SERVER_ADDRESS;

use super::error::Error;
use super::message::{IncomingMessage, JsonValue, OutgoingMessage, RegisterObject, StaticReplies};

/// A trait to be implemented by an application that wants to share the object
/// to the IPC server for remote call method calls from other processes.
#[async_trait]
pub trait SharedObject: Send + Sync + 'static {
    async fn remote_call(&self, method: &str, param: Option<JsonValue>)
        -> Result<JsonValue, Error>;
}

/// An object that is responsible in registering the object to the IPC server,
/// and spawning a tokio task to handling incoming remote method calls from
/// other processes.
pub struct ObjectDispatcher {
    socket: Arc<Mutex<TcpStream>>,
    list: Arc<Mutex<HashMap<String, Box<dyn SharedObject>>>>,
}

impl ObjectDispatcher {
    /// Create a new ObjectDispatcher object and connects to the IPC server.
    pub async fn new() -> Result<Self, Error> {
        let stream = TcpStream::connect(SERVER_ADDRESS)
            .await
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;

        Ok(Self {
            socket: Arc::new(Mutex::new(stream)),
            list: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    /// This registers the Shared Object into the IPC server.
    pub async fn register_object(
        &mut self,
        object: &str,
        shared_object: Box<dyn SharedObject>,
    ) -> Result<(), Error> {
        let mut list = self.list.lock().await;

        list.insert(object.to_string(), shared_object);

        let mut socket = self.socket.lock().await;
        let object = RegisterObject::new(object).serialize()?;

        socket
            .write_all(object.as_slice())
            .await
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;

        let mut buf = [0u8; u16::MAX as usize];
        let n = socket
            .read(&mut buf)
            .await
            .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;
        if n == 0 {
            Err(Error::new(JsonValue::String(
                StaticReplies::ServerConnectionError.to_string(),
            )))
        } else {
            let msg: IncomingMessage = serde_json::from_slice(&buf[0..n])
                .map_err(|e| Error::new(JsonValue::String(e.to_string())))?;

            match msg {
                IncomingMessage::Register(msg) => {
                    log::trace!("Register Object: {:?}", msg);
                    Ok(())
                }
                IncomingMessage::Error(msg) => Err(Error::new(JsonValue::String(msg.to_string()))),
                _ => {
                    log::trace!("Unhandled Message: {:?}", msg);
                    Ok(())
                }
            }
        }
    }

    /// This handles remote object method call from other processess.
    /// It spawns a tokio task to handle the calls asynchronously and sends
    /// back the response back to the remote process.
    pub async fn spawn(&mut self) -> JoinHandle<()> {
        let socket = self.socket.clone();
        let list = self.list.clone();
        tokio::spawn(async move {
            loop {
                let mut socket = socket.lock().await;
                let mut buf = [0u8; u16::MAX as usize];
                let n = socket.read(&mut buf).await.map_or_else(
                    |e| {
                        log::error!("{:?}", e);
                        0
                    },
                    |size: usize| {
                        log::trace!("Read size: {}", size);
                        size
                    },
                );

                if n == 0 {
                    log::error!("Error: server connection error");
                    break;
                } else if let Ok(msg) = serde_json::from_slice(&buf[0..n]) {
                    match msg {
                        IncomingMessage::CallRequest(request) => {
                            log::trace!("CallObjectRequest: {:?}", &request);
                            let val = list.lock().await;
                            let response = if let Some(call) = val.get(&request.object) {
                                match call.remote_call(&request.method, request.param).await {
                                    Ok(response) => OutgoingMessage::CallResponse(
                                        CallObjectResponse::new(response),
                                    ),
                                    Err(err) => OutgoingMessage::Error(err),
                                }
                            } else {
                                OutgoingMessage::Error(Error::new(JsonValue::String(
                                    StaticReplies::ObjectNotFound.to_string(),
                                )))
                            };
                            socket
                                .write_all(response.serialize().unwrap().as_slice())
                                .await
                                .unwrap_or_else(|e| log::error!("{:?}", e));
                        }
                        _ => {
                            log::trace!("Unhandled Message: {:?}", msg);
                        }
                    }
                } else {
                    let response = OutgoingMessage::Error(Error::new(JsonValue::String(
                        StaticReplies::SerdeParseError.to_string(),
                    )));
                    socket
                        .write_all(response.serialize().unwrap().as_slice())
                        .await
                        .unwrap_or_else(|e| log::error!("{:?}", e));
                }
            }
        })
    }
}
