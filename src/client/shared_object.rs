use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use super::error::Error;
use super::message::{CallObjectResponse, IncomingMessage, RegisterObject};

#[async_trait]
pub trait SharedObject: Send + Sync + 'static {
    async fn remote_call(
        &self,
        method: &str,
        param: Option<HashMap<String, String>>,
    ) -> Result<CallObjectResponse, Error>;
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
            Err(Error::Io("remote connection error".to_string()))
        } else {
            let msg: IncomingMessage =
                serde_json::from_slice(&buf[0..n]).map_err(|e| Error::Serde(e.to_string()))?;

            match msg {
                IncomingMessage::Register(_) => Ok(()),
                IncomingMessage::Error(msg) => Err(Error::Other(msg.error)),
                IncomingMessage::CallRequest(_) => Ok(()),
                IncomingMessage::CallResponse(_) => Ok(()),
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
                    panic!("Server was closed");
                } else {
                    let msg: IncomingMessage = serde_json::from_slice(&buf[0..n]).unwrap();
                    match msg {
                        IncomingMessage::Register(_) => todo!(),
                        IncomingMessage::Error(_) => todo!(),
                        IncomingMessage::CallRequest(request) => {
                            let val = list.lock().await;
                            let call = val.get(&request.object).unwrap();
                            let result = call
                                .remote_call(&request.method, request.param)
                                .await
                                .unwrap();

                            socket
                                .write_all(result.serialize().unwrap().as_slice())
                                .await
                                .unwrap();
                        }
                        IncomingMessage::CallResponse(_) => todo!(),
                    }
                }
            }
        })
        .await;
    }
}
