use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::unbounded_channel;

use crate::manager::TaskManager;
use crate::Server;

#[derive(Serialize, Deserialize, Debug)]
struct RegisterObject {
    reg_object: String,
}

impl RegisterObject {
    fn new(name: &str) -> Self {
        Self {
            reg_object: name.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Success {
    success: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Error {
    error: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallObjectRequest {
    pub object: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<HashMap<String, String>>,
}

impl CallObjectRequest {
    pub fn new(object: &str, method: &str) -> Self {
        Self {
            object: object.to_string(),
            method: method.to_string(),
            param: None,
        }
    }

    pub fn parameter(mut self, key: &str, value: &str) -> Self {
        if self.param.is_none() {
            self.param = Some(HashMap::new());
        }

        if let Some(param) = &mut self.param {
            param.insert(key.to_owned(), value.to_owned());
        }

        self
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallObjectResponse {
    pub response: String,
}

impl CallObjectResponse {
    pub fn new(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum IncomingMessage {
    Register(Success),
    Error(Error),
    CallRequest(CallObjectRequest),
    CallResponse(CallObjectResponse),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum OutgoingMessage {
    Register(RegisterObject),
    Error(Error),
    CallRequest(CallObjectRequest),
    CallResponse(CallObjectResponse),
}

#[tokio::test]
async fn test_server() {
    let (tx, rx) = unbounded_channel();

    let server = tokio::spawn(async move {
        TaskManager::spawn(rx).await;
        Server::spawn(tx).await;
    });

    let process1 = tokio::spawn(async move {
        let mut stream = TcpStream::connect("127.0.0.1:1986").await.unwrap();

        let register = OutgoingMessage::Register(RegisterObject::new("applications.oauth2"));

        stream
            .write_all(serde_json::to_vec(&register).unwrap().as_slice())
            .await
            .unwrap();

        loop {
            let mut buf = [0u8; std::u16::MAX as usize];
            let n = stream.read(&mut buf).await.unwrap();

            if n == 0 {
                break;
            }
            let result: IncomingMessage = serde_json::from_slice(&buf[0..n]).unwrap();

            match result {
                IncomingMessage::Error(msg) => {
                    println!("(1)Remote: Error: {:?}", msg);
                }
                IncomingMessage::Register(msg) => {
                    println!("(1)Remote: Successful Registration: {:?}", msg);
                }
                IncomingMessage::CallRequest(msg) => {
                    println!("(1)Remote: Request: {:?}", msg);

                    let response = OutgoingMessage::CallResponse(CallObjectResponse::new(
                        r#"{"result":"ok"}"#,
                    ));
                    stream
                        .write_all(serde_json::to_vec(&response).unwrap().as_slice())
                        .await
                        .unwrap();
                }
                IncomingMessage::CallResponse(msg) => {
                    println!("(1)Remote: Response: {:?}", msg);
                }
            }
        }
    });

    let process2 = tokio::spawn(async move {
        let mut stream = TcpStream::connect("127.0.0.1:1986").await.unwrap();
        loop {
            let mut param = HashMap::new();
            param.insert("key".to_string(), "value".to_string());
            let request = CallObjectRequest::new("applications.oauth2", "login")
                .parameter("key", "value")
                .parameter("key1", "value1")
                .parameter("key2", "value2");

            println!("Process 2: Request: {:?}", request);
            stream
                .write_all(serde_json::to_vec(&request).unwrap().as_slice())
                .await
                .unwrap();

            let mut buf = [0u8; std::u16::MAX as usize];

            let n = stream.read(&mut buf).await.unwrap();

            let result: IncomingMessage = serde_json::from_slice(&buf[0..n]).unwrap();
            match result {
                IncomingMessage::Error(msg) => {
                    println!("(2)Remote: Error: {:?}", msg);
                }
                IncomingMessage::Register(msg) => {
                    println!("(2)Remote: Response: {:?}", msg);
                }
                IncomingMessage::CallRequest(msg) => {
                    println!("(2)Remote: Request: {:?}", msg);

                    let response = OutgoingMessage::CallResponse(CallObjectResponse::new(
                        r#"{"result":"ok"}"#,
                    ));
                    stream
                        .write_all(serde_json::to_vec(&response).unwrap().as_slice())
                        .await
                        .unwrap();
                }
                IncomingMessage::CallResponse(msg) => {
                    println!("(2)Remote: Response: {:?}", msg);
                    break;
                }
            }
        }
    });

    process2.await.unwrap();
    process1.abort();
    server.abort();
}
