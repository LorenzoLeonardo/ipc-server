use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc::UnboundedReceiver, oneshot::Sender, Mutex},
};

use crate::{
    error::Error,
    message::{CallObject, IpcMessage, Message, Success},
};

pub struct TaskManager;

impl TaskManager {
    pub async fn spawn(mut rx: UnboundedReceiver<Message>) {
        tokio::spawn(async move {
            let mut list_session = HashMap::new();
            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        match msg {
                            Message::ProcessInput(session, tx) => {
                                match session.msg {
                                    IpcMessage::Register(data) => {
                                        log::trace!("{:?}", data);
                                        list_session.insert(data.reg_object, session.socket_holder);
                                        log::trace!("No. of shared objects: {}", list_session.len());

                                        tx.send(Success::new("ok").serialize().unwrap())
                                            .unwrap_or_else(|e| {
                                                log::error!("{:?}", e);
                                            });
                                    }
                                    IpcMessage::Call(request) => {
                                        log::trace!("{:?}", request);
                                        if let Some(s) = list_session.get(request.object.as_str()) {
                                            TaskManager::handle_call_request(s.socket.clone(), request, tx).await;
                                        } else {
                                            tx.send(Error::new("object not found").serialize().unwrap())
                                                .unwrap_or_else(|e| {
                                                    log::error!("{:?}", e);
                                                });
                                        }
                                    }
                                    _ => {}
                                }
                            },
                            Message::RemoveRegistered(session) => {
                                log::trace!("{:?}", session);
                                list_session.retain(|_, v| v.name != session.socket_holder.name);
                                log::trace!("Number of list: {}", list_session.len());
                            }
                        }
                    },
                }
            }
        });
    }

    async fn handle_call_request(
        socket: Arc<Mutex<TcpStream>>,
        request: CallObject,
        tx: Sender<Vec<u8>>,
    ) {
        let mut socket = socket.lock().await;

        match request.serialize() {
            Ok(request) => {
                // Forward this call request to the destination process
                socket.write_all(&request).await.unwrap_or_else(|e| {
                    log::error!("{:?}", e);
                });
            }
            Err(e) => {
                tx.send(Error::new(e.to_string().as_str()).serialize().unwrap())
                    .unwrap_or_else(|e| {
                        log::error!("{:?}", e);
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
                    log::error!("{:?}", e);
                });
            } else {
                // Forward the response of the call object back to the calling process
                tx.send(buffer[..bytes_read].to_vec()).unwrap_or_else(|e| {
                    log::error!("{:?}", e);
                });
            }
        } else {
            tx.send(Error::new("read error").serialize().unwrap())
                .unwrap_or_else(|e| {
                    log::error!("{:?}", e);
                });
        }
    }
}
