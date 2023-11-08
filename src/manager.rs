use std::collections::HashMap;

use tokio::sync::{mpsc::UnboundedReceiver, oneshot::Sender};

use ipc_client::client::{
    message::{CallObjectRequest, ListObjects, StaticReplies, Success},
    socket::Socket,
};

use crate::{
    error::Error,
    message::{IpcMessage, Message},
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
                                let ipaddress = session.socket.ip_address();

                                match session.msg {
                                    IpcMessage::Register(data) => {

                                        log::trace!("[{}]: {:?}", ipaddress, data);
                                        list_session.insert(data.reg_object, session.socket.clone());
                                        log::trace!("[{}]: Shared objects: {:?}", ipaddress, list_session);

                                        tx.send(Success::new(StaticReplies::Ok.as_ref()).serialize().unwrap())
                                            .unwrap_or_else(|e| {
                                                log::error!("{:?}", e);
                                            });
                                    }
                                    IpcMessage::Call(request) => {
                                        log::trace!("[{}]: {:?}", ipaddress, request);
                                        if let Some(s) = list_session.get(request.object.as_str()) {
                                            TaskManager::handle_call_request(s.clone(), request, tx, &mut list_session).await;
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
                                            log::trace!("[{}]: {:?} object are available.", ipaddress, request);
                                            serde_json::to_vec(&request).unwrap()
                                        } else {
                                            log::trace!("[{}]: {:?} object not yet available.", ipaddress, request);
                                            let list = ListObjects::new(Vec::new());
                                            serde_json::to_vec(&list).unwrap()
                                        };

                                        tx.send(response).unwrap_or_else(|e| {
                                            log::error!("{:?}", e);
                                        });
                                    }
                                    IpcMessage::AddToEventList(add_to_event) => {
                                        insert_or_update(&mut list_subscriber_for_event, add_to_event.event_name.as_str(), session.socket.clone());

                                        log::trace!("{} has subscribe for events.", ipaddress);
                                        log::trace!("Subscriber List: {:?}", list_subscriber_for_event);
                                        tx.send(Success::new(StaticReplies::Ok.as_ref()).serialize().unwrap())
                                        .unwrap_or_else(|e| {
                                            log::error!("{:?}", e);
                                        });
                                    }

                                    IpcMessage::BroadCastEvent(event) => {
                                        if let Some(list_socket_holder) = list_subscriber_for_event.get(&event.event) {
                                            for socket in list_socket_holder {
                                                log::trace!("Broadcasting this event to -> {}", socket.ip_address());

                                                socket.write_all(event.clone().serialize().unwrap().as_slice()).await.unwrap_or_else(|e|{
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
                                        log::trace!("Unhandled Message: {:?}", session.msg);
                                    }
                                }
                            },
                            Message::RemoveRegistered(session) => {
                                log::trace!("{:?}", session);
                                let ip_address = session.socket.ip_address();
                                list_session.retain(|_, v| v.ip_address() != ip_address);
                                log::trace!("[{}]: Shared objects: {:?}", ip_address, list_session);

                                remove_socket(&mut list_subscriber_for_event, ip_address);
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
        socket: Socket,
        request: CallObjectRequest,
        tx: Sender<Vec<u8>>,
        list_session: &mut HashMap<String, Socket>,
    ) {
        let ip_address = socket.ip_address();
        match request.serialize() {
            Ok(request) => {
                // Forward this call request to the destination process
                if let Err(e) = socket.write_all(&request).await {
                    // If Destination process cannot be reached, better remove it from the list.
                    list_session.retain(|_, v| v.ip_address() != ip_address);
                    log::trace!("[{}]: Shared objects: {:?}", ip_address, list_session);

                    tx.send(Error::new(e.to_string().as_str()).serialize().unwrap())
                        .unwrap_or_else(|e| {
                            log::error!("{:?}", e);
                        });
                    return;
                }
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
        if let Ok(bytes_read) = socket.read(&mut buffer).await {
            // Forward the response of the call object back to the calling process
            tx.send(buffer[0..bytes_read].to_vec()).unwrap_or_else(|e| {
                log::error!("{:?}", e);
            });
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

fn insert_or_update(map: &mut HashMap<String, Vec<Socket>>, key: &str, value: Socket) {
    let entry = map.entry(key.to_string());

    match entry {
        std::collections::hash_map::Entry::Occupied(mut e) => {
            if e.get().iter().all(|x| x.ip_address() != value.ip_address()) {
                e.get_mut().push(value);
            }
        }
        std::collections::hash_map::Entry::Vacant(e) => {
            e.insert(vec![value]);
        }
    }
}

fn remove_socket(map: &mut HashMap<String, Vec<Socket>>, ip_address: &str) {
    for (key, _value) in map.clone() {
        if let Some(values) = map.get_mut(&key) {
            values.retain(|x| x.ip_address() != ip_address);

            if values.is_empty() {
                map.remove(&key);
            }
        }
    }
}
