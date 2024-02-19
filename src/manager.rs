use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::{mpsc::UnboundedReceiver, oneshot::Sender, Mutex},
};

use ipc_client::client::{
    connector::read,
    message::{CallObjectRequest, ListObjects, StaticReplies, Success},
};

use json_elem::jsonelem::JsonElem;

use crate::{
    error::Error,
    message::{IpcMessage, Message, SocketHolder},
};

/// An object that is responsible in handling request from the server.
pub struct TaskManager;

impl TaskManager {
    /// Spawns the TaskManager in the background with tokio::select!() it handle
    /// asynchronous request from the server and pass into different handling stations
    /// depend on the type of Message that the server wants it to do.
    pub async fn spawn(mut rx: UnboundedReceiver<Message>) {
        tokio::spawn(async move {
            let mut list_session = HashMap::new();
            let mut list_subscriber_for_event = HashMap::new();
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
                                            TaskManager::handle_call_request(s.socket.clone(), request, tx, &mut list_session).await;
                                        } else {
                                            tx.send(Error::new(JsonElem::String(StaticReplies::ObjectNotFound.to_string())).serialize().unwrap())
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
                                    IpcMessage::AddToEventList(add_to_event) => {
                                        let ipaddress = session.socket_holder.name.clone();
                                        insert_or_update(&mut list_subscriber_for_event, add_to_event.event_name.as_str(), session.socket_holder.clone());

                                        log::trace!("{} has subscribe for events.", ipaddress);
                                        log::trace!("Subscriber List: {:?}", list_subscriber_for_event);
                                        tx.send(Success::new(StaticReplies::Ok.as_ref()).serialize().unwrap())
                                        .unwrap_or_else(|e| {
                                            log::error!("{:?}", e);
                                        });
                                    }

                                    IpcMessage::BroadCastEvent(event) => {
                                        if let Some(list_socket_holder) = list_subscriber_for_event.get(&event.event) {
                                            for holder in list_socket_holder {
                                                log::trace!("Broadcasting this event to -> {}", &holder.name);
                                                let mut socket = holder.socket.lock().await;
                                                socket.write_all(serde_json::to_string(&event.result).unwrap().as_bytes()).await.unwrap_or_else(|e|{
                                                    log::error!("{:?}", e);
                                                });
                                            }
                                        }
                                        tx.send(Success::new(StaticReplies::Ok.as_ref()).serialize().unwrap())
                                        .unwrap_or_else(|e| {
                                            log::error!("{:?}", e);
                                        });
                                    }
                                    _ => {
                                        log::error!("Unhandled Message: {:?}", session.msg);
                                    }
                                }
                            },
                            Message::RemoveRegistered(session) => {
                                log::trace!("{:?}", session);
                                let ip_address = session.socket_holder.name.clone();
                                list_session.retain(|_, v| v.name != ip_address);
                                log::trace!("[{}]: Shared objects: {:?}", session.socket_holder.name, list_session);

                                remove_socket(&mut list_subscriber_for_event, ip_address.as_str());
                                log::trace!("{} has unsubscribe from events.", ip_address);
                                log::trace!("Subscriber List: {:?}", list_subscriber_for_event);
                            }
                        }
                    },
                }
            }
        });
    }

    /// This handle remote object call method request from other process and return back to the server
    /// for proper sending of message to what client the response is needed to.
    async fn handle_call_request(
        socket: Arc<Mutex<TcpStream>>,
        request: CallObjectRequest,
        tx: Sender<Vec<u8>>,
        list_session: &mut HashMap<String, SocketHolder>,
    ) {
        let mut socket = socket.lock().await;
        let ip_address = socket.peer_addr().unwrap().to_string();
        match request.serialize() {
            Ok(request) => {
                // Forward this call request to the destination process
                if let Err(e) = socket.write_all(&request).await {
                    // If Destination process cannot be reached, better remove it from the list.
                    list_session.retain(|_, v| v.name != ip_address);
                    log::trace!("[{}]: Shared objects: {:?}", ip_address, list_session);

                    tx.send(
                        Error::new(JsonElem::String(e.to_string()))
                            .serialize()
                            .unwrap(),
                    )
                    .unwrap_or_else(|e| {
                        log::error!("{:?}", e);
                    });
                    return;
                }
            }
            Err(e) => {
                tx.send(
                    Error::new(JsonElem::String(e.to_string()))
                        .serialize()
                        .unwrap(),
                )
                .unwrap_or_else(|e| {
                    log::error!("{:?}", e);
                });
                return;
            }
        }

        let mut buffer = Vec::new();
        // Read the response from the destination process
        if let Ok(bytes_read) = read(&mut socket, &mut buffer).await {
            if bytes_read == 0 {
                tx.send(
                    Error::new(JsonElem::String(
                        StaticReplies::ClientConnectionError.to_string(),
                    ))
                    .serialize()
                    .unwrap(),
                )
                .unwrap_or_else(|e| {
                    log::error!("{:?}", e);
                });
            } else {
                // Forward the response of the call object back to the calling process
                tx.send(buffer[0..bytes_read].to_vec()).unwrap_or_else(|e| {
                    log::error!("{:?}", e);
                });
            }
        } else {
            tx.send(
                Error::new(JsonElem::String(
                    StaticReplies::ClientConnectionError.to_string(),
                ))
                .serialize()
                .unwrap(),
            )
            .unwrap_or_else(|e| {
                log::error!("{:?}", e);
            });
        }
    }
}

fn insert_or_update(map: &mut HashMap<String, Vec<SocketHolder>>, key: &str, value: SocketHolder) {
    let entry = map.entry(key.to_string());

    match entry {
        std::collections::hash_map::Entry::Occupied(mut e) => {
            if e.get().iter().all(|x| x.name != value.name) {
                e.get_mut().push(value);
            }
        }
        std::collections::hash_map::Entry::Vacant(e) => {
            e.insert(vec![value]);
        }
    }
}

fn remove_socket(map: &mut HashMap<String, Vec<SocketHolder>>, ip_address: &str) {
    for (key, _value) in map.clone() {
        if let Some(values) = map.get_mut(&key) {
            values.retain(|x| x.name.as_str() != ip_address);

            if values.is_empty() {
                map.remove(&key);
            }
        }
    }
}
