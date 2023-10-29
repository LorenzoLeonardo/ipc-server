use std::collections::HashMap;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::UnboundedReceiver,
};

use crate::{
    error::Error,
    message::{IpcMessage, Message},
};

pub struct TaskManager;

impl TaskManager {
    pub async fn spawn(mut rx: UnboundedReceiver<Message>) {
        tokio::spawn(async move {
            let mut list_session = HashMap::new();
            loop {
                let msg = rx.recv().await.unwrap();
                match msg {
                    Message::ProcessInput(session, tx) => match session.msg {
                        IpcMessage::Register(data) => {
                            println!("Register");
                            list_session.insert(data.reg_object, session.socket_holder);
                            println!("Number of list: {}", list_session.len());
                            let reg_ok = r#"{"success":"ok"}"#;
                            tx.send(reg_ok.as_bytes().to_vec()).unwrap_or_else(|e| {
                                eprintln!("{:?}", e);
                            });
                        }
                        IpcMessage::Call(data) => {
                            println!("Call");
                            if let Some(s) = list_session.get(data.object.as_str()) {
                                let mut socket = s.socket.lock().await;
                                let reply = serde_json::to_string(&data).unwrap();
                                socket.write_all(reply.as_bytes()).await.unwrap();

                                let mut buffer = [0u8; u16::MAX as usize];
                                if let Ok(bytes_read) = socket.read(&mut buffer).await {
                                    if bytes_read == 0 {
                                        tx.send(
                                            Error::new("client has lost connection").serialize(),
                                        )
                                        .unwrap_or_else(
                                            |e| {
                                                eprintln!("{:?}", e);
                                            },
                                        );
                                    } else {
                                        tx.send(buffer[..bytes_read].to_vec()).unwrap_or_else(
                                            |e| {
                                                eprintln!("{:?}", e);
                                            },
                                        );
                                    }
                                } else {
                                    tx.send(Error::new("read error").serialize())
                                        .unwrap_or_else(|e| {
                                            eprintln!("{:?}", e);
                                        });
                                }
                            } else {
                                tx.send(Error::new("object not found").serialize())
                                    .unwrap_or_else(|e| {
                                        eprintln!("{:?}", e);
                                    });
                            }
                        }
                        _ => {}
                    },
                    Message::RemoveRegistered(session) => {
                        list_session.retain(|_, v| v.name != session.socket_holder.name);
                        println!("Number of list: {}", list_session.len());
                    }
                }
            }
        });
    }
}
