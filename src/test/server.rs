use std::collections::HashMap;

use serde_derive::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::unbounded_channel;

use crate::manager::TaskManager;
use crate::Server;

#[derive(Serialize, Deserialize, Debug)]
struct Success {
    success: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Error {
    error: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallObject {
    pub object: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallObjectResponse {
    pub response: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Message {
    Register(Success),
    Error(Error),
    Request(CallObject),
    Response(CallObjectResponse),
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

        let register = r#"{"reg_object":"my_object"}"#;
        stream.write_all(register.as_bytes()).await.unwrap();

        loop {
            let mut buf = [0u8; std::u16::MAX as usize];
            let n = stream.read(&mut buf).await.unwrap();

            if n == 0 {
                break;
            }
            let result: Message = serde_json::from_slice(&buf[0..n]).unwrap();

            match result {
                Message::Error(msg) => {
                    println!("(1)Remote: Error Registration: {:?}", msg);
                }
                Message::Register(msg) => {
                    println!("(1)Remote: Successful Registration: {:?}", msg);
                }
                Message::Request(msg) => {
                    println!("(1)Remote: Request: {:?}", msg);
                    let response = r#"{"response":"my reply"}"#;
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
                Message::Response(msg) => {
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
            let request = CallObject {
                object: String::from("my_object"),
                method: String::from("call"),
                param: Some(param),
            };

            println!("Process 2: Request: {:?}", request);
            stream
                .write_all(serde_json::to_vec(&request).unwrap().as_slice())
                .await
                .unwrap();

            let mut buf = [0u8; std::u16::MAX as usize];

            let n = stream.read(&mut buf).await.unwrap();

            let result: Message = serde_json::from_slice(&buf[0..n]).unwrap();
            match result {
                Message::Error(msg) => {
                    println!("(2)Remote: Error Registration: {:?}", msg);
                }
                Message::Register(msg) => {
                    println!("(2)Remote: Response: {:?}", msg);
                }
                Message::Request(msg) => {
                    println!("(2)Remote: Request: {:?}", msg);
                    let response = r#"{"success":"my reply"}"#;
                    stream.write_all(response.as_bytes()).await.unwrap();
                }
                Message::Response(msg) => {
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
