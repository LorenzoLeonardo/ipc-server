use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc::UnboundedReceiver, oneshot::Sender, Mutex},
};

use ipc_client::client::message::StaticReplies;

use crate::{
    error::Error,
    message::{CallObject, IpcMessage, ListObjects, Message, Success},
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
                                        let ipaddress = session.socket_holder.name.clone();
                                        log::trace!("[{}]: {:?}", ipaddress, data);
                                        list_session.insert(data.reg_object, session.socket_holder);
                                        log::trace!("[{}]: Shared objects: {:?}", ipaddress, list_session);

                                        tx.send(Success::new(StaticReplies::Ok.as_ref()).serialize().unwrap())
                                            .unwrap_or_else(|e| {
                                                log::error!("{:?}", e);
                                            });
                                    }
                                    IpcMessage::Call(request) => {
                                        log::trace!("[{}]: {:?}",session.socket_holder.name, request);
                                        if let Some(s) = list_session.get(request.object.as_str()) {
                                            TaskManager::handle_call_request(s.socket.clone(), request, tx).await;
                                        } else {
                                            tx.send(Error::new(StaticReplies::ObjectNotFound.as_ref()).serialize().unwrap())
                                                .unwrap_or_else(|e| {
                                                    log::error!("{:?}", e);
                                                });
                                        }
                                    }
                                    IpcMessage::WaitForObjects(request) => {
                                        let mut found = true;
                                        for item in request.list.clone() {
                                            if list_session.get(&item).is_none() {
                                                found = false;
                                                break;
                                            }
                                        }

                                        let response = if found {
                                            log::trace!("[{}]: {:?} object are available.", session.socket_holder.name, request);
                                            serde_json::to_vec(&request).unwrap()
                                        } else {
                                            log::trace!("[{}]: {:?} object not yet available.", session.socket_holder.name, request);
                                            let list = ListObjects::new(Vec::new());
                                            serde_json::to_vec(&list).unwrap()
                                        };

                                        tx.send(response).unwrap_or_else(|e| {
                                            log::error!("{:?}", e);
                                        });
                                    }
                                    _ => {}
                                }
                            },
                            Message::RemoveRegistered(session) => {
                                list_session.retain(|_, v| v.name != session.socket_holder.name);
                                log::trace!("[{}]: Shared objects: {:?}", session.socket_holder.name, list_session);
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

        let mut buffer = Vec::new();
        // Read the response from the destination process
        if let Ok(bytes_read) = socket.read_buf(&mut buffer).await {
            if bytes_read == 0 {
                tx.send(
                    Error::new(StaticReplies::ClientConnectionError.as_ref())
                        .serialize()
                        .unwrap(),
                )
                .unwrap_or_else(|e| {
                    log::error!("{:?}", e);
                });
            } else {
                // Forward the response of the call object back to the calling process
                tx.send(buffer).unwrap_or_else(|e| {
                    log::error!("{:?}", e);
                });
            }
        } else {
            tx.send(
                Error::new(StaticReplies::ClientConnectionError.as_ref())
                    .serialize()
                    .unwrap(),
            )
            .unwrap_or_else(|e| {
                log::error!("{:?}", e);
            });
        }
    }
}
