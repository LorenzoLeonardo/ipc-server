use std::collections::HashMap;

use ipc_client::client::connector::{
    CallObjectRequest, CallObjectResponse, IncomingMessage, OutgoingMessage, RegisterObject,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::unbounded_channel;

use crate::manager::TaskManager;
use crate::Server;

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
