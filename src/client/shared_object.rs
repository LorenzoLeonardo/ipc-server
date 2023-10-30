use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use super::error::Error;
use super::message::{self, IncomingMessage, OutgoingMessage, RegisterObject};

#[async_trait]
pub trait SharedObject: Send + Sync + 'static {
    async fn remote_call(
        &self,
        method: &str,
        param: Option<HashMap<String, String>>,
    ) -> OutgoingMessage;
}

pub struct ObjectDispatcher {
    socket: Arc<Mutex<TcpStream>>,
    list: Arc<Mutex<HashMap<String, Box<dyn SharedObject>>>>,
}
impl ObjectDispatcher {
    pub async fn new() -> Result<Self, Error> {
        let stream = TcpStream::connect("127.0.0.1:1986")
            .await
            .map_err(|e| Error::Socket(e.to_string()))?;

        Ok(Self {
            socket: Arc::new(Mutex::new(stream)),
            list: Arc::new(Mutex::new(HashMap::new())),
        })
    }

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
            .map_err(|e| Error::Io(e.to_string()))?;

        let mut buf = [0u8; u16::MAX as usize];
        let n = socket
            .read(&mut buf)
            .await
            .map_err(|e| Error::Io(e.to_string()))?;

        if n == 0 {
            Err(Error::Io("server connection error".to_string()))
        } else {
            let msg: IncomingMessage =
                serde_json::from_slice(&buf[0..n]).map_err(|e| Error::Serde(e.to_string()))?;

            match msg {
                IncomingMessage::Register(msg) => {
                    println!("Success: {:?}", msg);
                    Ok(())
                }
                IncomingMessage::Error(msg) => Err(Error::Other(msg.error)),
                _ => {
                    println!("Unhandled Message: {:?}", msg);
                    Ok(())
                }
            }
        }
    }

    pub async fn spawn(&mut self) {
        let socket = self.socket.clone();
        let list = self.list.clone();
        let _ = tokio::spawn(async move {
            loop {
                let mut socket = socket.lock().await;
                let mut buf = [0u8; u16::MAX as usize];
                let n = socket.read(&mut buf).await.unwrap();

                if n == 0 {
                    eprintln!("Error: server connection error");
                    break;
                } else if let Ok(msg) = serde_json::from_slice(&buf[0..n]) {
                    match msg {
                        IncomingMessage::CallRequest(request) => {
                            let val = list.lock().await;
                            let response = if let Some(call) = val.get(&request.object) {
                                call.remote_call(&request.method, request.param).await
                            } else {
                                OutgoingMessage::Error(message::Error::new(
                                    "object not found".to_string(),
                                ))
                            };
                            socket
                                .write_all(response.serialize().unwrap().as_slice())
                                .await
                                .unwrap_or_else(|e| eprintln!("{:?}", e));
                        }
                        _ => {
                            println!("Unhandled Message: {:?}", msg);
                        }
                    }
                } else {
                    let response =
                        OutgoingMessage::Error(message::Error::new("Serde error".to_string()));
                    socket
                        .write_all(response.serialize().unwrap().as_slice())
                        .await
                        .unwrap_or_else(|e| eprintln!("{:?}", e));
                }
            }
        })
        .await;
    }
}