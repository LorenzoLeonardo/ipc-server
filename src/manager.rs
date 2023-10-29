use std::collections::HashMap;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc::UnboundedReceiver,
};

use crate::{
    error::Error,
    message::{IpcMessage, Message, Success},
};

pub struct TaskManager;

impl TaskManager {
    pub async fn spawn(mut rx: UnboundedReceiver<Message>) {
        tokio::spawn(async move {
            let mut list_session = HashMap::new();
            loop {
                let msg = rx.recv().await.unwrap();
                match msg {
                    Message::ProcessInput(session, tx) => {
                        match session.msg {
                            IpcMessage::Register(data) => {
                                println!("Register");
                                list_session.insert(data.reg_object, session.socket_holder);
                                println!("Number of list: {}", list_session.len());

                                tx.send(Success::new("ok").serialize().unwrap())
                                    .unwrap_or_else(|e| {
                                        eprintln!("{:?}", e);
                                    });
                            }
                            IpcMessage::Call(request) => {
                                println!("Call");
                                if let Some(s) = list_session.get(request.object.as_str()) {
                                    let mut socket = s.socket.lock().await;

                                    match request.serialize() {
                                        Ok(request) => {
                                            // Forward this call request to the destination process
                                            socket.write_all(&request).await.unwrap_or_else(|e| {
                                                eprintln!("{:?}", e);
                                            });
                                        }
                                        Err(e) => {
                                            tx.send(
                                                Error::new(e.to_string().as_str())
                                                    .serialize()
                                                    .unwrap(),
                                            )
                                            .unwrap_or_else(|e| {
                                                eprintln!("{:?}", e);
                                            });
                                            return;
                                        }
                                    }

                                    let mut buffer = [0u8; u16::MAX as usize];
                                    // Read the response from the destination process
                                    if let Ok(bytes_read) = socket.read(&mut buffer).await {
                                        if bytes_read == 0 {
                                            tx.send(
                                                Error::new("client has lost connection")
                                                    .serialize()
                                                    .unwrap(),
                                            )
                                            .unwrap_or_else(|e| {
                                                eprintln!("{:?}", e);
                                            });
                                        } else {
                                            // Forward the response of the call object back to the calling process
                                            tx.send(buffer[..bytes_read].to_vec()).unwrap_or_else(
                                                |e| {
                                                    eprintln!("{:?}", e);
                                                },
                                            );
                                        }
                                    } else {
                                        tx.send(Error::new("read error").serialize().unwrap())
                                            .unwrap_or_else(|e| {
                                                eprintln!("{:?}", e);
                                            });
                                    }
                                } else {
                                    tx.send(Error::new("object not found").serialize().unwrap())
                                        .unwrap_or_else(|e| {
                                            eprintln!("{:?}", e);
                                        });
                                }
                            }
                            _ => {}
                        }
                    }
                    Message::RemoveRegistered(session) => {
                        list_session.retain(|_, v| v.name != session.socket_holder.name);
                        println!("Number of list: {}", list_session.len());
                    }
                }
            }
        });
    }
}
